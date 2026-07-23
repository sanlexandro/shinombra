//! Структуры, необходимые для работы WledDrgb

use std::{
    net::{SocketAddr, UdpSocket}, time::{Duration, Instant},
};

pub const DEFAULT_PORT: u16 = 21324;
pub const DEFAULT_TIMEOUT: u8 = 2;
pub const DEFAULT_MAX_FPS: u16 = 30;

/// Необходимые данные для работы WledDrgb
/// 
/// **Поля:**
/// - `timeout`: [u8]             - время ожидания ленты
/// - `socket`: [UdpSocket]       - собственно сокет, в который отправляются данные
/// - `address`: [SocketAddr]     - адрес ленты
/// - `wait_duration`: [Duration] - задержка, для соблюдения fps (чтобы не
///   положить сетевой стек)
/// - `last_frame`: [Instant]     - время последней отправки кадра
/// - `payload`: [Vec]<[u8]>      - буфер для отправки (необходим во избежание
///   аллокаций памяти в runtime)
pub struct WledDrgb {
    pub(crate) timeout: u8,
    pub(crate) socket: UdpSocket,
    pub(crate) address: SocketAddr,
    pub(crate) wait_duration: Duration,
    pub(crate) last_frame: Instant,
    pub(crate) payload: Vec<u8>,
}
