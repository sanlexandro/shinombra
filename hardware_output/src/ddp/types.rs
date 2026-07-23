//! Структуры, необходимые для работы с DDP

use std::{
    net::{SocketAddr, UdpSocket},
    time::{Duration, Instant},
};

/// Тип передаваемых данных в пакете
#[repr(u8)]
pub enum DataType {
    Default = 0x00,
    RGB = 0x01,
    RGBW = 0x02,
    HSV = 0x03,
}

/// Пакет DDP
pub struct PackageHeader {
    pub flags: u8,
    pub sequence_number: u8,
    pub data_type: u8,
    pub device_id: u8,
    pub offset: u32,
    pub data_length: u16,
}

pub const DEFAULT_MTU: u16 = 1500;
pub const APPROX_NETWORK_STACK_COSTS: u16 = 50;
pub const DEFAULT_PORT: u16 = 4048;
pub const DEFAULT_MAX_FPS: u16 = 30;

/// Необходимые данные для работы DDP
///
/// **Поля:**
/// - `socket`: [UdpSocket]             - собственно сокет, в который отправляются
///   данные
/// - `max_pixels_per_package`: [usize] - максимальное число пикселей в пакете
/// - `address`: [SocketAddr]           - адрес ленты
/// - `wait_duration`: [Duration]       - задержка, для соблюдения fps (чтобы не
///   положить сетевой стек)
/// - `last_frame`: [Instant]           - время последней отправки кадра
/// - `header`: [PackageHeader]         - заголовок пакета
/// - `payload`: [Vec]<[u8]>            - буфер для отправки (необходим во
///   избежание аллокаций памяти в runtime)
pub struct Ddp {
    pub(crate) socket: UdpSocket,
    pub(crate) max_pixels_per_package: usize,
    pub(crate) address: SocketAddr,
    pub(crate) wait_duration: Duration,
    pub(crate) last_frame: Instant,
    pub(crate) header: PackageHeader,
    pub(crate) payload: Vec<u8>,
}
