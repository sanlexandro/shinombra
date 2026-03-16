//! Структуры хранения цвета

/// Пиксель в формате HSV *(Hue Saturation Value)*
#[derive(Debug, Default, Copy, Clone)]
pub struct HSVPixel {
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
}

/// Пиксель в формате RBG *(Red Green Blue)*
#[derive(Debug, Default, Copy, Clone)]
pub struct RGBPixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}