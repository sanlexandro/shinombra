//! Ручная настройка каналов

use crate::{
    color::types::RGBPixel,
    filters::{configs::ChannelGainConfig, types::ChannelGain, ColorFilter},
};

impl ChannelGain {
    /// Конструктор
    pub fn new(config: ChannelGainConfig) -> Self {
        Self {
            k_red: config.k_red,
            k_green: config.k_green,
            k_blue: config.k_blue,
        }
    }
}

impl ColorFilter for ChannelGain {
    fn apply(&mut self, raw_colors: &mut crate::color::types::ColorBuffer) {
        for raw in AsMut::<[RGBPixel]>::as_mut(raw_colors).iter_mut() {
            raw.red = (raw.red as f32 * self.k_red) as u8;
            raw.green = (raw.green as f32 * self.k_green) as u8;
            raw.blue = (raw.blue as f32 * self.k_blue) as u8;
        }
    }
}
