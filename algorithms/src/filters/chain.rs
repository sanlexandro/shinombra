//! Алгоритм запуска нескольких фильтров подряд

use crate::{
    color::types::RGBPixel,
    filters::{ColorFilter, types::{FilterChain, FilterInstance}},
};

/// Реализация методов [FilterChain]
impl FilterChain {
    /// Конструктор
    /// 
    /// Создаёт пустую цепь
    pub fn new() -> Self {
        return Self { filters: Vec::new() };
    }

    /// Добавление фильтра
    /// 
    /// Добавляет в конец цепи новый фильтр
    /// 
    /// **Поля:**
    /// - `filter`: [FilterInstance] - хранилище с фильтром
    pub fn add_filter(&mut self, filter: FilterInstance) {
        self.filters.push(filter);
    }


    /// Запуск цепи фильтров
    /// 
    /// **Поля:**
    /// - `raw_colors`: &[[RGBPixel]] - массив новых цветов для подсчёта
    ///
    /// **Выходные данные:**
    /// - ?[RGBPixel] - массив фильтрованных цветов в формате RGB
    pub fn run_chain<'a>(&'a mut self, raw_colors: &'a [RGBPixel]) -> &'a [RGBPixel] {
        let mut current_colors = raw_colors;

        // Последовательно пропускаем данные через каждый фильтр в цепочке
        for filter in self.filters.iter_mut() {
            current_colors = filter.apply(current_colors);
        }

        return current_colors;
    }
}
