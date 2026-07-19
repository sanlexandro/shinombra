//! Поток, необходимый для отправки данных на устройство
//!
//! Работает по принципу почтового ящика (mailbox)
//! Копирует предоставленный массив в локальный буфер через atomic операцию, а
//! затем отправляет буфер на устройство

use algorithms::color::{types::RGBPixel, Color};
use common::core::{controller::CoreController, event_handlers::EventHandler};
use hardware_output::HardwareOutput;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex,
};
use std::thread;

use crate::hardware_output::handler::{shutdown_error, HardwareHandler};

/// Поток отправки данных на устройство
///
/// Полностью инкапсулирует логику работы с потоком отправки
///
/// **Поля:**
/// - `mailbox`: [Arc]<([Mutex]<([Vec]<[RGBPixel]>, [bool])>, [Condvar])> -
///   почтовый ящик
/// - `keep_running`: [Arc]<[AtomicBool]> - собственный флаг на продолжение работы
/// - `handle`: [Option]<thread::JoinHandle<()>> - поток
/// - `controller`: [Arc]<[CoreController]> - контроллер ядра
pub struct HardwareOutputThread {
    pub(super) mailbox: Arc<(Mutex<(Vec<RGBPixel>, bool)>, Condvar)>,
    pub(super) keep_running: Arc<AtomicBool>,
    pub(super) handle: Option<thread::JoinHandle<()>>,
}

/// Реализация методов [HardwareOutputThread]
impl HardwareOutputThread {
    /// Конструктор
    ///
    /// Создаёт поток и контроллер потока
    ///
    /// **Поля:**
    /// - `output`: [HardwareOutput] - готовый метод отправки данных
    /// - `buffer_size`: [usize]     - размер буфера (обычно - количество светодиодов)
    /// - `core_controller`: [Arc]<[CoreController]> - контроллер ядра
    ///
    /// **Выходные поля:**
    /// - [HardwareOutputThread] - готовый контроллер потока
    pub fn new<Output>(
        output: Output,
        buffer_size: usize,
        core_controller: Arc<CoreController>,
    ) -> HardwareOutputThread
    where
        Output: HardwareOutput + Send + 'static,
    {
        // Создаем связку (Данные, Флаг Обновления) + Condvar
        let mailbox = Arc::new((
            Mutex::new((vec![RGBPixel::black(); buffer_size], false)),
            Condvar::new(),
        ));
        let keep_running = Arc::new(AtomicBool::new(true));

        let worker = HardwareOutputWorker {
            output,
            buffer: vec![RGBPixel::black(); buffer_size],
            mailbox: Arc::clone(&mailbox),
            keep_running: Arc::clone(&keep_running),
            core_controller,
        };

        // Отрываем от главного потока
        let handle = thread::spawn(move || {
            worker.run();
        });

        // Возвращаем наружу красивый Контроллер
        HardwareOutputThread {
            mailbox,
            keep_running,
            handle: Some(handle),
        }
    }

    /// Обновить цвета в буфере
    ///
    /// Данный метод пробует перехватить `lock`, чтобы записать данные в
    /// локальный буфер работника. При неудаче отпускает, а при успехе "будит" поток-работник.
    /// 
    /// Автоматически преобразует принятый тип в [RGBPixel] за счёт ограничений
    /// трейта [Color]
    ///
    /// **Поля:**
    /// -  `colors`: &[Color] - указатель на массив цветов для отправки в любом формате
    pub fn update_colors<C: Copy + Into<RGBPixel>>(&self, colors: &[C]) {
        let (lock, cvar) = &*self.mailbox;
        if let Ok(mut state) = lock.try_lock() {
            for (dest, &src) in state.0.iter_mut().zip(colors.iter()) {
                *dest = src.into();
            }

            state.1 = true; // Указываем, что появились новые данные
            cvar.notify_one(); // Будим поток отправки
        }
    }

    /// Остановка потока
    ///
    /// Данный метод мягко останавливает поток, позволяя работнику отправить
    /// последнее сообщение
    pub fn stop(&mut self) {
        self.keep_running.store(false, Ordering::Relaxed);
        let (_, cvar) = &*self.mailbox;
        cvar.notify_all(); // Будим поток-работник, если он спал, чтобы он мог выйти

        // Ждём пока работник всё дошлёт и мягко завершаем
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// "Работник" потока отправки
///
/// **Поля:**
/// - `output`: [HardwareOutput] - способ отправки данных
/// - `buffer`: [Vec]<[RGBPixel]> - собственный буфер
/// - `mailbox`: [Arc]<([Mutex]<([Vec]<[RGBPixel]>, [bool])>, [Condvar])> -
///   почтовый ящик
/// - `keep_running`: [Arc]<[AtomicBool]> - флаг продолжения работы
/// - `core_controller`: [Arc]<[CoreController]> - контроллер ядра
struct HardwareOutputWorker<Output: HardwareOutput> {
    output: Output,
    buffer: Vec<RGBPixel>,
    mailbox: Arc<(Mutex<(Vec<RGBPixel>, bool)>, Condvar)>,
    keep_running: Arc<AtomicBool>,
    core_controller: Arc<CoreController>,
}

impl<Output: HardwareOutput + Send + 'static> HardwareOutputWorker<Output> {
    /// Основная ф-я отправки данных
    ///
    /// Работает по следующей инструкции пока `keep_running = true` :
    /// 1) заснуть до появления новых данных
    /// 2) захватить `lock` и скопировать буфер
    /// 3) сбросить флаг и отпустить мьютекс
    /// 4) отправить данные на устройство
    ///
    /// Когда `keep_running = false` на устройство отправляется сигнал завершения
    fn run(mut self) {
        while self.keep_running.load(Ordering::Relaxed) {
            {
                let (lock, cvar) = &*self.mailbox;
                let mut state = lock.lock().unwrap();

                // Спим в ожидании момента, когда появятся новые данные (flag == true)
                // ИЛИ когда придет команда на остановку потока
                while !state.1 && self.keep_running.load(Ordering::Relaxed) {
                    state = cvar.wait(state).unwrap();
                }

                // Проверяем, не разбудили ли нас ради остановки программы
                if !self.keep_running.load(Ordering::Relaxed) {
                    break;
                }

                self.buffer.clone_from_slice(&state.0);
                state.1 = false; // Сбрасываем флаг, говоря "я забрал эти данные"
            }
            // Отправляем данные, при этом мьютекс уже отпущен
            match self.output.send_colors(&self.buffer) {
                Ok(_) => (),
                Err(event) => HardwareHandler::handle(event, &self.core_controller),
            }
        }

        // После работы отправляем сигнал завершения
        if let Err(error) = self.output.send_shutdown_signal() {
            // Ругаемся на ошибку, но всё равно просто завершаемся
            shutdown_error(error);
        }
    }
}
