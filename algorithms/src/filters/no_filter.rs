//! Отсутствие фильтра

use crate::filters::{types::NoFilter, ColorFilter};

impl NoFilter {
    pub fn new() -> Self {
        return Self {};
    }
}

impl ColorFilter for NoFilter {
    fn apply(&mut self, _: &mut crate::color::types::ColorBuffer) {}
}
