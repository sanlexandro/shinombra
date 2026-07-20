//! Баланс белого
//!
//! Регулирует RGB спектр согласно заданной температуре

use crate::{
    color::types::RGBPixel,
    filters::{configs::WhiteBalanceConfig, types::WhiteBalance, ColorFilter},
};

impl WhiteBalance {
    /// Конструктор
    ///
    /// Предрасчитывает коэффициенты
    ///
    /// LUT в данном случае будет менее эффективен, т.к. операция умножения
    /// дешевле подглядывания в массив
    pub fn new(config: WhiteBalanceConfig) -> Self {
        let t = config.kelvins as f32 / 100.0;

        Self {
            k_red: Self::calculate_red_k(t),
            k_green: Self::calculate_green_k(t),
            k_blue: Self::calculate_blue_k(t),
        }
    }

    /// Расчёт коэффициента красного канала алгоритмом Таннера Хеллэнда
    pub(super) fn calculate_red_k(t: f32) -> f32 {
        if t <= 66.0 {
            return 1.0;
        }
        (1.292936 * (t - 60.0).powf(-0.13320476)).clamp(0.0, 1.0)
    }

    /// Расчёт коэффициента зелёного канала алгоритмом Таннера Хеллэнда
    pub(super) fn calculate_green_k(t: f32) -> f32 {
        if t <= 66.0 {
            return (0.39008158 * t.ln() - 0.63184144).clamp(0.0, 1.0);
        }
        (1.12989086 * (t - 60.0).powf(-0.07551485)).clamp(0.0, 1.0)
    }

    /// Расчёт коэффициента синего канала алгоритмом Таннера Хеллэнда
    pub(super) fn calculate_blue_k(t: f32) -> f32 {
        if t <= 19.0 {
            return 0.0;
        } else if t < 66.0 {
            return (0.54320678 * (t - 10.0).ln() - 1.19625409).clamp(0.0, 1.0);
        }
        1.0
    }
}

impl ColorFilter for WhiteBalance {
    fn apply(&mut self, raw_colors: &mut crate::color::types::ColorBuffer) {
        for raw in AsMut::<[RGBPixel]>::as_mut(raw_colors).iter_mut() {
            raw.red = (raw.red as f32 * self.k_red) as u8;
            raw.green = (raw.green as f32 * self.k_green) as u8;
            raw.blue = (raw.blue as f32 * self.k_blue) as u8;
        }
    }
}
