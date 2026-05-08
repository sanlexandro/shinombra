//! Реализация преобразования [RGBA] -> [RGBPixel]

use crate::{
    color::types::RGBPixel,
    pixel_formatter::{types::RGBA, PixelFormatter},
};

impl PixelFormatter for RGBA {
    const SIZE: usize = 4;

    fn to_rgb(data: &[u8]) -> crate::color::types::RGBPixel {
        RGBPixel {
            red: data[0],
            green: data[1],
            blue: data[2],
        }
    }
}
