//! Реализация отладочного вывода

use algorithms::color::types::RGBPixel;

use crate::HardwareEvents;

use super::types::DebugDriver;

/// Ф-я вывода цвета в консоль для отладки
fn print_debug_color(rgb: RGBPixel) {
    // \x1b[48;2;R;G;Bm — цвет фона (TrueColor)
    // \x1b[0m — сброс
    print!(
        "\x1b[48;2;{};{};{}m      \x1b[0m", // 6 пробелов
        rgb.red, rgb.green, rgb.blue
    );
}

/// Реализация методов DebugDriver
impl DebugDriver {
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

    /// Отладочный вывод на экран
    ///
    /// Выводит рамку из цветов так, как эти цвета выводились бы на экран с
    /// помощью ленты
    ///
    /// **Аргументы:**
    /// - `colors`: &[RGBPixel] - массив цветов для вывода
    pub fn print_debug_frame(&self, colors: &[RGBPixel]) -> Result<(), HardwareEvents> {
        // Верхняя строка
        print!("       "); // 7 пробелов
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
                print!("       "); // 7 пробелов
            }

            // Вывели кусочек правой стенки
            print_debug_color(colors[(self.led_width + idx) as usize]);
        }

        // Нижняя строчка
        println!("");
        print!("       "); // 7 пробелов
        for idx in 0..self.led_width {
            print_debug_color(colors[(colors_amount - self.led_height - idx) as usize]);
            print!(" ");
        }

        println!("");
        println!("");

        Ok(())
    }
}
