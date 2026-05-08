//! Реализация преобразования [xBGR] -> [RGBPixel]

use crate::color::types::RGBPixel;
use crate::pixel_formatter::{types::xBGR, PixelFormatter};

impl PixelFormatter for xBGR {
    const SIZE: usize = 4;

    #[inline(always)]
    fn to_rgb(data: &[u8]) -> RGBPixel {
        RGBPixel {
            red: data[3],
            green: data[2],
            blue: data[1],
        }
    }
}
