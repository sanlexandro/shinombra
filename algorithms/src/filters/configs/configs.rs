//! Конфигурация, необходимая для работы фильтров

/// Настройки gamma-фильтра
///
/// **Поля:**
/// - `gamma`: [f32] - коэффициент для gamma (обычно ~2.2 для светодиодных лент)
pub struct GammaConfig {
    pub gamma: f32,
}

/// Настройки EMA-фильтра
///
/// **Поля:**
/// - `alpha`: [f32] - коэффициент "сглаживания" (чем меньше, тем резче меняется
///   цвет)
/// - `amount`: [usize] - количество светодиодов
pub struct EmaConfig {
    pub alpha: f32,
    pub amount: usize,
}

/// Настройки отсекателя тёмного
///
/// **Поля:**
/// - `threshold`: [f32]        - минимальная полностью отсекаемая яркость %
/// - `fade_range`: [f32]       - ширина зоны плавного отсечения %
/// - `falloff_exponent`: [f32] - крутизна кривой
#[derive(Clone)]
pub struct BlackThresholdConfig {
    pub threshold: f32,
    pub fade_range: f32,    
    pub falloff_exponent: f32,
}
