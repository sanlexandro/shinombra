//! Вспомогательные функции и структуры

use syn::{
    parse::{Parse, ParseStream},
    LitStr, Token,
};

/// Вспомогательная ф-я для преобразования имени в snake_case
pub fn name_to_snake_case(name_ident: String) -> String {
    let mut name_str = String::new();
    for (i, ch) in name_ident.chars().enumerate() {
        if ch.is_uppercase() && i != 0 {
            name_str.push('_');
        }
        name_str.push(ch.to_ascii_lowercase());
    }
    return name_str;
}

// Вспомогательная структура для парсинга нескольких путей: include_shadow_all!("a.rs", "b.rs")
pub(crate) struct PathList {
    pub(crate) paths: Vec<LitStr>,
}
// Реализация парсинга путей
impl Parse for PathList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut paths = Vec::new();
        while !input.is_empty() {
            paths.push(input.parse()?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(PathList { paths })
    }
}
