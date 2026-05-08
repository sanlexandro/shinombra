//! Реализация преобразования [BGRA] -> [RGBPixel]

use crate::color::types::RGBPixel;
use crate::pixel_mapper::{types::BGRA, PixelFormatter};

impl PixelFormatter for BGRA {
    const SIZE: usize = 4;

    #[inline(always)]
    fn to_rgb(data: &[u8]) -> RGBPixel {
        RGBPixel {
            red: data[2],
            green: data[1],
            blue: data[0],
        }
    }
}
