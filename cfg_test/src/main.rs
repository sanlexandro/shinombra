use config_gen::{*, __private::*};

include_shadow_all!("./src/unit.rs", "./src/config.rs");

fn main() {
    // 1. Имитируем чтение файла
    let toml_str = std::fs::read_to_string("cfg.toml")
        .expect("Не удалось прочитать cfg.toml");

    // 2. Десериализуем в " Мега-обертку"
    let shadow_root: FullConfigShadow = toml::from_str(&toml_str)
        .expect("Ошибка парсинга TOML");

    // 3. Достаем GeometryShadow и превращаем в реальную Geometry
    if let Some(geo_shadow) = shadow_root.geometry {
        let mut geo: Geometry = geo_shadow.into(); // Работает наш From!
        
        // 4. Проверяем логику методов
        geo.calculate();
        geo.print(); // Должно вывести: unset = 30
    } else {
        println!("Секция [Geometry] не найдена в конфиге");
    }
}