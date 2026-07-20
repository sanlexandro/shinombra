//! Алгоритм запуска нескольких фильтров подряд

use crate::{
    color::types::ColorBuffer,
    filters::{
        types::{FilterChain, FilterInstance},
        ColorFilter,
    },
};

impl FilterChain {
    /// Конструктор
    ///
    /// Создаёт пустую цепь
    pub fn new() -> Self {
        return Self {
            filters: Vec::new(),
        };
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
    /// - `raw_colors`: & mut [ColorBuffer] - массив новых цветов для подсчёта
    pub fn run_chain(&mut self, raw_colors: &mut ColorBuffer) {
        // Последовательно пропускаем данные через каждый фильтр в цепочке
        for filter in self.filters.iter_mut() {
            filter.apply(raw_colors);
        }
    }
}

impl ColorFilter for FilterChain {
    fn apply(&mut self, raw_colors: &mut ColorBuffer) {
        for filter in self.filters.iter_mut() {
            filter.apply(raw_colors);
        }
    }
}

impl ColorFilter for FilterInstance {
    fn apply(&mut self, raw_colors: &mut ColorBuffer) {
        match self {
            Self::Ema(f) => f.apply(raw_colors),
            Self::Gamma(f) => f.apply(raw_colors),
            Self::BlackThreshold(f) => f.apply(raw_colors),
            Self::SaturationBoost(f) => f.apply(raw_colors),
            Self::WhiteBalance(f) => f.apply(raw_colors),
            Self::ChannelGain(f) => f.apply(raw_colors),
        }
    }
}
