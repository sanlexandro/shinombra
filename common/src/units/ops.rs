//! Операции над мерами

use crate::units::*;

macro_rules! impl_unit_ops {
    ($name:ident, $type:ty) => {
        // Сложение: Unit + Unit
        impl std::ops::Add for $name {
            type Output = Self;
            fn add(self, other: Self) -> Self { $name(self.0 + other.0) }
        }

        // Вычитание: Unit - Unit
        impl std::ops::Sub for $name {
            type Output = Self;
            fn sub(self, other: Self) -> Self { $name(self.0 - other.0) }
        }

        // Умножение на число (скаляр): Unit * u32
        impl std::ops::Mul<$type> for $name {
            type Output = Self;
            fn mul(self, rhs: $type) -> Self { $name(self.0 * rhs) }
        }

        // Умножение друг на друга: Unit * Unit
        impl std::ops::Mul<$name> for $name {
            type Output = Self;
            fn mul(self, other: $name) -> Self { $name(self.0 * other.0) }
        }

        // Деление на число: Unit / u32
        impl std::ops::Div<$type> for $name {
            type Output = Self;
            fn div(self, rhs: $type) -> Self { $name(self.0 / rhs) }
        }

        // Деление друг на друга: Unit / Unit
        impl std::ops::Div<$name> for $name {
            type Output = Self;
            fn div(self, other: $name) -> Self { $name(self.0 / other.0) }
        }
        
        // Позволяет делать +=
        impl std::ops::AddAssign for $name {
            fn add_assign(&mut self, other: Self) { self.0 += other.0; }
        }
    };
}

impl_unit_ops!(Millimeters, u32);
impl_unit_ops!(Pixels, usize);