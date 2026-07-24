//! Реализация связи по протоколу DDP
//!
//! Реализация только под RGB, т.к. его принимает большинство микроконтроллеров
//!
//! TODO: добавить вариант для HSV

use std::{
    format,
    net::{ToSocketAddrs, UdpSocket},
};

use algorithms::color::types::RGBPixel;

use crate::{
    ddp::types::DEFAULT_PORT,
    ddp::{config::config::DdpConfig, types::*},
    HardwareOutput,
};

impl PackageHeader {
    /// Конструктор
    ///
    /// По умолчанию создаётся с push=0
    pub fn new() -> Self {
        Self {
            flags: 0b0100_0000,
            sequence_number: 1,
            data_type: DataType::RGB as u8,
            device_id: 0x01,
            offset: 0,
            data_length: 0,
        }
    }

    /// Установка значения флага push
    pub fn set_push_flag(&mut self, is_push: bool) {
        if is_push {
            self.flags |= 0x80;
        } else {
            self.flags &= !0x80;
        }
    }

    /// Вставка заголовка в буфер
    pub fn write_to_buffer(&self, buf: &mut Vec<u8>) {
        buf.push(self.flags);
        buf.push(self.sequence_number);
        buf.push(self.data_type);
        buf.push(self.device_id);

        // Преобразуем в Big-Endian (сетевой порядок)
        buf.extend_from_slice(&self.offset.to_be_bytes());
        buf.extend_from_slice(&self.data_length.to_be_bytes());
    }

    /// Сдвиг счётчика пакетов
    pub fn tick(&mut self) {
        self.sequence_number = (self.sequence_number % 15) + 1
    }
}

impl Ddp {
    /// Конструктор
    pub fn new(config: DdpConfig, led_amount: usize) -> Result<Self, String> {
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

        // Вычисляем максимальный размер пакета такой, чтобы в него помещалось
        // целое количество пикселей
        let max_pixels_per_package: usize = (config.mtu.unwrap_or(DEFAULT_MTU)
            - APPROX_NETWORK_STACK_COSTS) as usize
            / size_of::<RGBPixel>();

        // Рассчитываем размеры буфера заранее как минимум из максимального
        // буфера или буфера по числу светодиодов
        let payload_size: usize =
            (max_pixels_per_package as usize).min(led_amount) * size_of::<RGBPixel>();

        Ok(Self {
            socket,
            max_pixels_per_package,
            address,
            header: PackageHeader::new(),
            payload: Vec::with_capacity(size_of::<PackageHeader>() + payload_size),
        })
    }
}

impl HardwareOutput for Ddp {
    fn send_colors(&mut self, colors: &[RGBPixel]) -> Result<(), crate::HardwareEvents> {
        // Ставим флаг отправки в 0
        self.header.set_push_flag(false);
        // Сбрасываем сдвиг
        self.header.offset = 0;

        // Разбиваем весь массив на чанки, которые будут отправлены отдельными
        // пакетами
        let mut chunks = colors.chunks(self.max_pixels_per_package).peekable();

        // Начинаем отправлять пакеты
        while let Some(chunk) = chunks.next() {
            // Если это последний пакет устанавливаем флаг в 1
            if chunks.peek().is_none() {
                self.header.set_push_flag(true);
            }

            // Устанавливаем длину данных
            self.header.data_length = (chunk.len() * size_of::<RGBPixel>()) as u16;

            // Очищаем буфер
            self.payload.clear();

            // Записываем в него заголовок
            self.header.write_to_buffer(&mut self.payload);

            // Записываем текущие данные пакета
            for color in chunk.iter() {
                self.payload
                    .extend_from_slice(&[color.red, color.green, color.blue]);
            }

            // Отправляем пакет
            // В случае неудачи прерываем отправку
            if let Err(err) = self.socket.send_to(&self.payload, &self.address) {
                return Err(err.into());
            }

            // Сдвигаем offset
            self.header.offset += (chunk.len() * size_of::<RGBPixel>()) as u32;
        }

        // Обновляем номер кадра
        self.header.tick();

        Ok(())
    }

    fn send_shutdown_signal(&mut self) -> Result<(), String> {
        Ok(())
    }
}
