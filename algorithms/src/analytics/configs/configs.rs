//! Конфигурация, необходимая для работы различных анализаторов цвета

/// Настройки для [super::types::ColorHistogram]
///
/// **Поля:**
/// - `precision_level`: [usize] - множитель дискретизации. Число, задающее
///   количество разбиений цветового круга по 6. Т.е. в итоге цветовой круг
///   будет разбит на `precision_level * 6` корзин
#[derive(Default, Debug)]
pub struct ColorHistogramConfig {
    pub precision_level: usize,
}
