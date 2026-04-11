use crate::{
    processing::{configs::*, measures::calculate_mm_to_px_k},
    units::*,
};

// Реализация методов GeometryConfig
impl GeometryConfig {
    /// Расчёт конфигурации фрагмента
    ///
    /// Данный метод необходим для определения конфигурации фрагмента, исходя из
    /// данных, переданных пользователю
    ///
    /// **Аргументы:**
    /// - `screen_config`: [ScreenConfig] - информация об экране
    pub fn calculate_chunk_config(&self, screen_config: ScreenConfig) -> ChunkConfig {
        // Коэффициент по вертикали
        let k_y =
            calculate_mm_to_px_k(screen_config.frame_height_mm, screen_config.frame_height_px);
        // Коэффициент по горизонтали (на случай нестандартных экранов)
        let k_x = calculate_mm_to_px_k(screen_config.frame_width_mm, screen_config.frame_width_px);

        ChunkConfig {
            // Ширина — это длина блока диодов (в px)
            width: self.led_pos.led_length.as_pixels(k_x),
            // Высота — это суммарная глубина захвата (в px)
            height: Pixels::new(
                ((self.reading.deep_in.0 + self.reading.deep_out.0) as f64 * k_y).round() as usize,
            ),
        }
    }

    /// Расчёт количества светодиодов в ленте
    pub fn calculate_leds_amount(&self) -> usize {
        return (self.led_pos.horizontal_led_amount + self.led_pos.vertical_led_amount) * 2;
    }
}

// TODO! Комментарии к коду!

impl ScreenConfig {
    pub fn load_px(& mut self,frame_width_px: u32, frame_height_px: u32) {
        self.frame_width_px = Pixels(frame_width_px as usize);
        self.frame_height_px = Pixels(frame_height_px as usize);
    }
}