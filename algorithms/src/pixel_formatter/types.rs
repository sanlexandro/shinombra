//! Поддерживаемые для конвертации типы
//!
//! См. документацию pipewire https://gstreamer.freedesktop.org/documentation/additional/design/mediatype-video-raw.html?gi-language=c

#![allow(non_camel_case_types)]

pub struct RGBx;
pub struct BGRx;
pub struct xRGB;
pub struct xBGR;
pub struct RGBA;
pub struct BGRA;
pub struct ARGB;
pub struct ABGR;
pub struct RGB; // TODO: Требует выравнивания по строке
pub struct BGR; // TODO: Требует выравнивания по строке
