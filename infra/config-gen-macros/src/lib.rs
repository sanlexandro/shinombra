//! Макрос генерации теневых структур
//!
//! Генерация позволяет создать теневые копии каждой структуры. Причём, если
//! сама структура является типом поля другой структуры, то в новой структуре
//! оригинальный тип будет заменён своим клоном. Это позволяет, соблюдать
//! SRP (принцип единственной ответственности), а также SST(принцип
//! единственного места правды)
//!
//! В зависимости от типа макрос самостоятельно выбирает синтаксис
//!
//! Также для возможности использования макрос автоматически создаёт реализацию
//! трейта [From]

use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet;
use std::fs;
use syn::{parse_macro_input, File, Item};

mod item_enum;
mod item_struct;
mod utils;

use item_enum::gen_enum;
use item_struct::gen_struct;
use utils::{name_to_snake_case, PathList};

/// Генерация теневых копии всех структур и enum'ов из множества файлов конфигурации
///
/// Этот макрос является точкой входа для процесса генерации. Он загружает все указанные
/// файлы конфигурации, парсит их как Rust код, и для каждой найденной структуры или enum'а
/// создаёт "теневую" копию с суффиксом `Shadow`. Теневые структуры используют типы,
/// подходящие для десериализации (с автоматическим преобразованием типов полей через трейты).
/// Затем создаётся корневая структура `FullConfigShadow`, которая содержит Optional поля
/// для всех найденных типов.
///
/// **Входные данные:**
/// - `input`: `TokenStream` - список путей вида `"path/to/file1.rs", "path/to/file2.rs", ...`
///
/// **Процесс выполнения:**
/// 1. Парсит входной `TokenStream` в список путей (`PathList`)
/// 2. Загружает и парсит каждый файл конфигурации
/// 3. Собирает все имена структур и enum'ов во множество (`all_struct_names`)
/// 4. Проходит по всем найденным элементам и генерирует их теневые версии
/// 5. Создаёт реализации трейта `From` для конвертации между оригиналом и тенью
/// 6. Генерирует корневую структуру `FullConfigShadow` с Optional полями для каждого типа
///
/// **Возвращает:**
/// - `TokenStream` - полный генерированный код (теневые структуры, enum'ы, реализации трейтов и корневая структура)
#[proc_macro]
pub fn include_shadow_all(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as PathList);
    let mut all_struct_names = HashSet::new(); // Хранилище для имён всех изменённых структур
    let mut all_items = Vec::new(); // Хранилище для найденных объектов

    // Собираем все имена структур из всех файлов
    for path_lit in &input.paths {
        let path = path_lit.value();
        let content = fs::read_to_string(&path).expect("File not found");
        let file = syn::parse_str::<File>(&content).expect("Parse error");

        for item in &file.items {
            if let Item::Struct(struct_) = item {
                all_struct_names.insert(struct_.ident.to_string());
            } else if let Item::Enum(struct_) = item {
                all_struct_names.insert(struct_.ident.to_string());
            }
        }
        all_items.push(file.items);
    }

    let mut final_tokens = Vec::new(); // Хранилище токенов

    // Генерируем код в зависимости от типа
    for items in all_items {
        for item in items {
            match item {
                Item::Struct(struct_) => {
                    final_tokens.push(gen_struct(&struct_, &all_struct_names));
                }
                syn::Item::Enum(ref enum_) => {
                    final_tokens.push(gen_enum(&enum_));
                }
                Item::Use(_) => continue, // Пропускаем импорты
                _ => final_tokens.push(quote! { #item }),
            }
        }
    }

    // Внутри макроса, когда собрали все all_struct_names создаём корень,
    // включающий в себя все структуры
    let fields = all_struct_names.iter().map(|name| {
        let field_name_ident = quote::format_ident!("{}", name_to_snake_case(name.to_string())); // Простой snake_case
        let type_ = quote::format_ident!("{}Shadow", name);
        quote! { pub #field_name_ident: Option<#type_> }
    });

    final_tokens.push(quote! {
        #[derive(::config_gen::__private::serde::Deserialize, ::config_gen::__private::serde::Serialize, Default, Debug)]
        #[serde(crate = "::config_gen::__private::serde")]
        pub struct FullConfigShadow {
            #( #fields, )*
        }

        // Для отображения на экране и отладки
        impl std::fmt::Display for FullConfigShadow {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    });

    TokenStream::from(quote! { #( #final_tokens )* })
}
