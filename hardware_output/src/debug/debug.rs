//! Реализация отладочного вывода

use algorithms::color::types::RGBPixel;

use crate::{HardwareEvents, HardwareOutput};

use super::types::Debug;

/// Ф-я вывода цвета в консоль для отладки
fn print_debug_color(rgb: RGBPixel) {
    // \x1b[48;2;R;G;Bm — цвет фона (TrueColor)
    // \x1b[0m — сброс
    print!(
        "\x1b[48;2;{};{};{}m({:<3}, {:<3}, {:<3})\x1b[0m", // 6 + 9 = 15 символов
        rgb.red, rgb.green, rgb.blue, rgb.red, rgb.green, rgb.blue
    );
}

/// Реализация методов Debug
impl Debug {
    /// Конструктор отладочного вывода
    ///
    /// **Аргументы:**
    /// - `led_width`: [usize]  - количество блоков светодиодов, помещающиеся на
    ///   экране в ширину
    /// - `led_height`: [usize] - количество блоков светодиодов, помещающиеся на
    ///   экране в высоту
    pub fn new(led_width: usize, led_height: usize) -> Self {
        return Self {
            led_width,
            led_height,
        };
    }
}

impl HardwareOutput for Debug {
    /// Отладочный вывод на экран
    ///
    /// Выводит рамку из цветов так, как эти цвета выводились бы на экран с
    /// помощью ленты
    ///
    /// **Аргументы:**
    /// - `colors`: &[RGBPixel] - массив цветов для вывода
    fn send_colors(&mut self, colors: &[RGBPixel]) -> Result<(), HardwareEvents> {
        // Верхняя строка
        print!(" {:15}", ""); // 15+1 пробелов
        for idx in 0..self.led_width {
            print_debug_color(colors[idx as usize]);
            print!(" ");
        }

        // Количество фрагментов
        let colors_amount = self.led_width * 2 + self.led_height * 2;

        // Боковые стенки
        for idx in 0..self.led_height {
            println!("");
            println!("");
            // Вывели кусок левой стенки
            print_debug_color(colors[(colors_amount - idx - 1) as usize]);
            print!(" ");

            for _ in 0..self.led_width {
                print!(" {:15}", ""); // 15+1 пробелов
            }

            // Вывели кусочек правой стенки
            print_debug_color(colors[(self.led_width + idx) as usize]);
        }

        // Нижняя строчка
        println!("");
        print!(" {:15}", ""); // 15+1 пробелов
        for idx in 0..self.led_width {
            print_debug_color(colors[(colors_amount - self.led_height - idx) as usize]);
            print!(" ");
        }

        println!("");
        println!("");

        Ok(())
    }

    /// Отправка завершающего сигнала
    fn send_shutdown_signal(&mut self) -> Result<(), String> {
        println!("Got shutdown signal!");
        Ok(())
    }
}
