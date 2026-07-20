//! Конфигурация, необходимая для работы различных анализаторов цвета

/// Настройки для [super::types::Histogram]
///
/// **Поля:**
/// - `precision_level`: [usize] - множитель дискретизации. Число, задающее
///   количество разбиений цветового круга по 6. Т.е. в итоге цветовой круг
///   будет разбит на `precision_level * 6` корзин
#[derive(Default, Debug)]
pub struct HistogramConfig {
    pub precision_level: usize,
}

/// Настройка отладочного аналитика
/// 
/// Принимает RGB
pub struct DebugRGBConfig {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}