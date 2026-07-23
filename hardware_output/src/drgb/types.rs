//! Структуры, необходимые для работы DRGB

use std::{
    net::{SocketAddr, UdpSocket}, time::{Duration, Instant},
};

pub const DEFAULT_PORT: u16 = 21324;
pub const DEFAULT_TIMEOUT: u8 = 2;
pub const DEFAULT_MAX_FPS: u16 = 30;

pub struct DRGB {
    pub timeout: u8,
    pub socket: UdpSocket,
    pub address: SocketAddr,
    pub wait_duration: Duration,
    pub last_frame: Instant,
}
