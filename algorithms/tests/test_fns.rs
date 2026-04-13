/// Функции, необходимые для тестирования

/// Проверка того, что первое значение меньше
///
/// **Поля:**
/// - `a` - меньшее число
/// - `b` - большее число
#[track_caller]
pub fn assert_smaller<T: PartialOrd + std::fmt::Display>(a: T, b: T) {
    assert!(a < b, "Expected: {} < {}; Actually: {} >= {}", a, b, a, b);
}

/// Проверка того, что первое значение больше
///
/// **Поля:**
/// - `a` - большее число
/// - `b` - меньшее число
#[track_caller]
pub fn assert_bigger<T: PartialOrd + std::fmt::Display>(a: T, b: T) {
    assert!(a > b, "Expected: {} > {}; Actually: {} <= {}", a, b, a, b);
}

/// Проверка того, что числа приблизительно равны с точностью
///
/// **Поля:**
/// - `a` - число для сравнения
/// - `b` - число для сравнения
/// - `e` - погрешность (меньше `a` и `b`)
#[track_caller]
pub fn assert_approximately_equal<T>(a: T, b: T, e: T)
where
    T: PartialOrd + std::fmt::Display + std::ops::Sub<Output = T> + Copy,
{
    // Либо если вы хотели проверить, что разница (ошибка) меньше заданного `e`, это делается в конце:
    let diff = if a > b { a - b } else { b - a };
    assert!(
        diff < e,
        "Expected max diff: {}; Actually diff is: {}",
        e,
        diff
    );
}