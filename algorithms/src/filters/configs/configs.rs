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
pub struct BlackThresholdConfig {
    pub threshold: f32,
    pub fade_range: f32,
    pub falloff_exponent: f32,
}

/// Настройки усилителя насыщенности
///
/// **Поля:**
/// - `floor`: [f32]          - нижний порог, до которого цвета не изменяются %
/// - `boost_exponent`: [f32] - сила "выгибания" кривой (>1 - вверх)
pub struct SaturationBoostConfig {
    pub floor: f32,
    pub boost_exponent: f32,
}

/// Настройка баланса белого
///
/// **Поля:**
/// - `kelvins`: [u32] - температура в кельвинах
pub struct WhiteBalanceConfig {
    pub kelvins: u32,
}

/// Настройка каналов
///
/// **Поля:**
/// - `k_red`: [f32]   - коэффициент для красного
/// - `k_green`: [f32] - зелёного
/// - `k_blue`: [f32]  - синего
pub struct ChannelGainConfig {
    pub k_red: f32,
    pub k_green: f32,
    pub k_blue: f32,
}

/// Настройки защиты от вспышек
///
/// **Поля:**
/// - `alpha`: [f32]       - коэффициент сглаживания встроенного EMA
/// - `sensitivity`: [f32] - порог чувствительности к вспышкам
pub struct FlashGuardConfig {
    pub alpha: f32,
    pub sensitivity: f32,
}
