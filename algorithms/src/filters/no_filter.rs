//! Отсутствие фильтра

use crate::{color::types::RGBPixel, filters::types::NoFilter};

impl NoFilter {
    pub fn new(amount: usize) -> Self {
        return Self {
            states: vec![RGBPixel::black(); amount],
        };
    }
}
