//! Макрос генерации методов, необходимых для отрисовки UI
//!
//! Данный макрос зависит от макроса генерации теневых структур. Для них он
//! реализует методы трейта [Renderable], в зависимости от настроек, переданных
//! для отображения данной структуры. Также собираются методы для парсинга и
//! отправки данных

use proc_macro::TokenStream;
use quote::quote;
use std::fs;
use syn::{parse_macro_input, Error, File, Item};

mod html;
mod parse;
mod types;

use types::*;

/// Вспомогательная структура для преобразования имени к snake_case
fn name_to_snake_case(name_ident: &String) -> String {
    let mut name_str = String::new();
    for (i, ch) in name_ident.chars().enumerate() {
        if ch.is_uppercase() && i != 0 {
            name_str.push('_');
        }
        name_str.push(ch.to_ascii_lowercase());
    }
    return name_str;
}

/// Генерация методов для отображения UI
///
/// Данный макрос принимает настройки в виде `"path/to/file.rs" => {...}`
/// Для каждого файла должны быть описаны структуры в виде `Struct => {...}`.
/// Для полей структуры можно описывать отображение в виде `field: WidgetType
/// {...}`, если для поля не указать ничего (`{}`), то оно не будет отображено
///
/// **Существующие типы полей [WidgetType]:**
/// - [WidgetType::Registry] - поле, отображающее варианты enum-а. Принимает
///   настройки в виде `{"paht/to/registry.rs" => RegType}`, где `RegType` - это
///   конкретный тип enum, указанный в данном файле
/// - [WidgetType::NumericField] - поле, отображающее численные значения.
///   Принимает настройки в виде `{ min, max }`, где `min` и `max` указываются в
///   виде числа. Они не являются обязательными, возможны указания: `{ }` или `{
///   min }`. Отдельно `max` указать нельзя
/// - [WidgetType::TextField] - поле, отображающее текстовые значения. Принимает
///   настройки в виде `{ "placeholder" }`, где `placeholder` - это подсказка
/// - [WidgetType::BoolField] - поле, отображающее булевы значения как checkbox.
///   Не требует параметров. Синтаксис: `{ }`. Генерирует checkbox input с
///   автоматической установкой атрибута `checked` на основе значения bool
/// - [WidgetType::Wrapper] - виджет-обёртка для поддержки tuple struct
///   (например, Millimeters). Обеспечивает корректный доступ к внутреннему
///   значению обёртки для отрисовки и инициализацию обёртки при парсинге JSON.
///   Принимает настройки в виде `( TupleStruct WidgetType {...} )`, где в
///   качестве `WidgetType` может быть любой виджет, а TupleStruct - это тип
///   самой обёртки
/// - [WidgetType::WrapperVec] - виджет-коллекция для полей с типом [Vec]<T>.
///   Генерирует контейнер с возможностью добавления и удаления элементов.
///   Не имеет собственных настроек и является просто обёрткой. Синтаксис вида
///   `WrapperVec( WidgetType )`, где в качестве `WidgetType` может быть любой
///   виджет
///
/// **Реализуемые методы функции:**
/// - `render_full_html(shadow_root: &FullShadowConfig) -> String` - функция для
///   генерации полного html-текста всех структур
/// - `from_json(json: ::ui_gen::__private::serde_json::Value) -> Self` - метод `FullShadowConfig`,
///   позволяющий распарсить данную структуру из Json
/// - `apply_patch(&mut self, patch: Self)` - метод `FullShadowConfig`,
///   позволяющий применить "патч", дополняющий поля в состоянии [None]
#[proc_macro]
pub fn generate_ui(input_raw: TokenStream) -> TokenStream {
    let inputs = parse_macro_input!(input_raw as PathList);

    let mut generated_functions = Vec::new(); // Место, для сохранения сгенерированных функций
    let mut struct_types = Vec::new();
    let mut generated_json_parsers = Vec::new(); // Хранилище для сгенерированных Json парсеров

    // Генерация методов по всем файлам
    for input in inputs.inputs {
        // Читаем файл
        let content = fs::read_to_string(input.file_path.value()).expect("File not found");

        // Парсим файл как AST Rust
        let file = syn::parse_str::<File>(&content).expect("Parse error");

        // Генерация HTML
        for item in file.items {
            if let Item::Struct(struct_) = item {
                let struct_name = struct_.ident;

                // Сохраняем все имена для реализации специального метода в будущем
                struct_types.push(struct_name.to_string());

                // Ищем настройки для структуры по её имени
                if let Some(config) = input.configs.iter().find(|c| c.struct_name == struct_name) {
                    let mut logic_fields = Vec::new(); //
                    let mut static_fields = Vec::new(); //

                    // Проходим по всем полям
                    for field in config.fields.iter() {
                        //
                        let (field_logic, field_static) = field.gen_html(struct_name.to_string());

                        logic_fields.push(field_logic);
                        static_fields.push(field_static);
                    }

                    // Генерируем строку с условием
                    let condition_string = if let Some(expr) = &config.condition {
                        format!(
                            r#"data-show-if="{}""#,
                            quote!(#expr).to_string().replace("\"", "'")
                        )
                    } else {
                        String::new()
                    };

                    // Формируем итоговый html
                    let div_start = format!(r#"<div id="{}" {}> "#, struct_name, condition_string);
                    let div_end = "</div>";

                    let static_html =
                        div_start.to_string() + &static_fields.join("\n") + &div_end.to_string();

                    // Имя теневой структуры
                    let shadow_struct_name = quote::format_ident!("{}Shadow", struct_name);
                    let name_snake_ident =
                        quote::format_ident!("{}", name_to_snake_case(&struct_name.to_string()));

                    let mut field_parsers = Vec::new();
                    for field in config.fields.iter() {
                        let field_ident = &field.field_name;
                        let field_name_str = field_ident.to_string();

                        match &field.widget_type {
                            WidgetType::Wrapper(wrapper_ident, _) => {
                                let wrapper_shadow_ident =
                                    quote::format_ident!("{}Shadow", wrapper_ident);
                                field_parsers.push(quote! {
                                    if let Some(val) = struct_obj.get(#field_name_str) {
                                        if let Ok(parsed_val) = ::ui_gen::__private::serde_json::from_value::<_>(val.clone()) {
                                            target.#field_ident = #wrapper_shadow_ident(parsed_val);
                                        }
                                    }
                                });
                            }
                            _ => {
                                field_parsers.push(quote! {
                                    if let Some(val) = struct_obj.get(#field_name_str) {
                                        if let Ok(parsed_val) = ::ui_gen::__private::serde_json::from_value(val.clone()) {
                                            target.#field_ident = parsed_val;
                                        }
                                    }
                                });
                            }
                        }
                    }

                    generated_json_parsers.push(quote! {
                        if let Some(data) = obj.get(stringify!(#struct_name)) {
                            if let Some(struct_obj) = data.as_object() {
                                if self.#name_snake_ident.is_none() {
                                    self.#name_snake_ident = Some(#shadow_struct_name::default());
                                }
                                if let Some(ref mut target) = self.#name_snake_ident {
                                    #(#field_parsers)*
                                }
                            }
                        }
                    });

                    generated_functions.push(quote! {
                        impl ::ui_gen::Renderable for #shadow_struct_name {
                            fn render_ui(mode: &::ui_gen::RenderMode<Self>) -> String {
                                use ::ui_gen::RenderMode;
                                // В зависимости от режима генерируем
                                match mode {
                                    // Статическую строку
                                    RenderMode::Create => #static_html.to_string(),
                                    // Динамический код
                                    RenderMode::Edit(data) => {
                                        let mut html: Vec<String> = Vec::new();

                                        html.push(String::from(#div_start));

                                        #( #logic_fields )*

                                        html.push(String::from(#div_end));

                                        html.join("\n")
                                        // String::new()
                                    }
                                }
                            }
                        }
                    });
                } else {
                    // Если не нашли структуру, возвращаем ошибку
                    return Error::new(
                        struct_name.span(),
                        format!("Can`t find config for struct {}", struct_name),
                    )
                    .to_compile_error()
                    .into();
                }
            }
        }
    }

    // Генерация функции для логики генерации html
    let mut generated_render_function = Vec::new(); // Хранилище сгенерированных методов для отрисовки UI

    for name in struct_types {
        let shadow_name_ident = quote::format_ident!("{}Shadow", name);
        let name_snake_ident = quote::format_ident!("{}", name_to_snake_case(&name));

        generated_render_function.push(quote! {
            full_html.push(#shadow_name_ident::render_ui(
                &match shadow_root.#name_snake_ident.as_ref() {
                    Some(s) => ::ui_gen::RenderMode::Edit(&s),
                    None => ::ui_gen::RenderMode::Create,
            }));
        });
    }

    // Сборка функции для сборки целой html строки из всех структур
    generated_functions.push(quote! {
        fn render_full_html(shadow_root: &FullConfigShadow) -> String {
            let mut full_html = Vec::new();

            #( #generated_render_function )*

            full_html.join("\n")
        }
    });

    // Сборка парсера и применения патчей для FullConfigShadow
    generated_functions.push(quote! {
        impl FullConfigShadow {
            /// Применяет новые данные к существующей структуре на уровне полей, не затирая отсутствующие поля
            pub fn apply_json_patch(&mut self, json: ::ui_gen::__private::serde_json::Value) {
                if let Some(obj) = json.as_object() {
                    #( #generated_json_parsers )*
                }
            }
        }
    });

    // Отправляем
    TokenStream::from(quote! {#(#generated_functions)*})
}
