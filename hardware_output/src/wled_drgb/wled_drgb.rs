//! Реализация связи по WledDrgb

use std::{
    format,
    net::{ToSocketAddrs, UdpSocket},
};

use algorithms::color::types::RGBPixel;

use crate::{
    wled_drgb::{config::config::WledDrgbConfig, types::*},
    HardwareEvents, HardwareOutput,
};

impl WledDrgb {
    pub fn new(config: WledDrgbConfig, led_amount: usize) -> Result<Self, String> {
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

        Ok(Self {
            timeout: config.timeout.unwrap_or(DEFAULT_TIMEOUT),
            socket,
            address,
            payload: Vec::with_capacity(2 + led_amount * size_of::<RGBPixel>()),
        })
    }
}

impl HardwareOutput for WledDrgb {
    /// Отправка на устройство по wifi
    ///
    /// Отправляет пакет вида: [0x02 + rgb_1 + rgb_2]
    fn send_colors(&mut self, colors: &[RGBPixel]) -> Result<(), HardwareEvents> {
        // Формируем пакет
        self.payload.clear();
        self.payload = Vec::<u8>::with_capacity(2 + colors.len() * size_of::<RGBPixel>());
        self.payload.push(0x02); // ID протокола
        self.payload.push(self.timeout); // Установленный таймаут

        // Собираем всё в пакет
        for color in colors.iter() {
            self.payload
                .extend_from_slice(&[color.red, color.green, color.blue]);
        }

        // И отправляем
        match self.socket.send_to(&self.payload, &self.address) {
            Ok(_) => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    /// Отсутствует в протоколе
    fn send_shutdown_signal(&mut self) -> Result<(), String> {
        Ok(())
    }
}
