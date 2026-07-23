//! Реализация связи по DRGB

use std::{
    format,
    net::{ToSocketAddrs, UdpSocket},
    time::{Duration, Instant},
};

use algorithms::color::types::RGBPixel;

use crate::{
    drgb::{config::config::DRGBConfig, types::*},
    HardwareEvents, HardwareOutput,
};

impl DRGB {
    pub fn new(config: DRGBConfig) -> Result<Self, String> {
        // Пробуем открыть сокет на любом порту
        let socket = match UdpSocket::bind("0.0.0.0:0") {
            Ok(s) => s,
            Err(err) => {
                return Err(err.to_string());
            }
        };
        // Собираем адрес
        // Можно развернуть без проверки, т.к. при валидации конфига проверка
        // уже прошла, но как фолбек оставлен вариант с данными из конфига
        let address = format!("{}:{}", config.ip, config.port.unwrap_or(DEFAULT_PORT))
            .to_socket_addrs()
            .ok()
            .and_then(|mut addrs| addrs.next())
            .unwrap();

        // Вычисляем время ожидания
        let max_fps = config.max_fps.unwrap_or(DEFAULT_MAX_FPS) as u64;
        let wait_duration = Duration::from_nanos(1_000_000_000 / max_fps.max(1));

        Ok(Self {
            timeout: config.timeout.unwrap_or(DEFAULT_TIMEOUT),
            socket,
            address,
            wait_duration,
            last_frame: Instant::now(),
        })
    }
}

impl HardwareOutput for DRGB {
    /// Отправка на устройство по wifi
    ///
    /// Отправляет пакет вида: [0x02 + rgb_1 + rgb_2]
    fn send_colors(&mut self, colors: &[RGBPixel]) -> Result<(), HardwareEvents> {
        // Проверка для соблюдения fps
        if self.last_frame.elapsed() < self.wait_duration {
            return Ok(());
        }
        // Обновляем время отправки последнего кадра
        self.last_frame = Instant::now();

        // Формируем пакет
        let mut packet = Vec::<u8>::with_capacity(2 + colors.len() * 3);
        packet.push(0x02); // ID протокола
        packet.push(self.timeout); // Установленный таймаут

        // Собираем всё в пакет
        for color in colors.iter() {
            packet.push(color.red);
            packet.push(color.green);
            packet.push(color.blue);
        }

        // И отправляем
        match self.socket.send_to(&packet, &self.address) {
            Ok(_) => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    /// Отсутствует в протоколе
    fn send_shutdown_signal(&mut self) -> Result<(), String> {
        Ok(())
    }
}
