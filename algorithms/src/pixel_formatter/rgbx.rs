//! Реализация преобразования [RGBx] -> [RGBPixel]

use crate::{
    color::types::RGBPixel,
    pixel_formatter::{types::RGBx, PixelFormatter},
};

impl PixelFormatter for RGBx {
    const SIZE: usize = 4;

    fn to_rgb(data: &[u8]) -> crate::color::types::RGBPixel {
        RGBPixel {
            red: data[0],
            green: data[1],
            blue: data[2],
        }
    }
}
