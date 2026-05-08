//! Реализация преобразования [xRGB] -> [RGBPixel]

use crate::{
    color::types::RGBPixel,
    pixel_formatter::{types::xRGB, PixelFormatter},
};

impl PixelFormatter for xRGB {
    const SIZE: usize = 4;

    fn to_rgb(data: &[u8]) -> crate::color::types::RGBPixel {
        RGBPixel {
            red: data[1],
            green: data[2],
            blue: data[3],
        }
    }
}
