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
    /// - `raw_colors`: & mut [[RGBPixel]] - массив новых цветов для подсчёта
    pub fn run_chain(&mut self, raw_colors: & mut [RGBPixel]) {
        // Последовательно пропускаем данные через каждый фильтр в цепочке
        for filter in self.filters.iter_mut() {
            filter.apply(raw_colors);
        }
    }
}
