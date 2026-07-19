//! Парсинг синтаксических структур
//!
//! Здесь реализован трейт [Parse] для всех моделей из модуля `types`.
//! Логика позволяет корректно извлекать настройки виджетов и полей из потока токенов.

use super::types::*;
use syn::{
    parse::{Parse, ParseStream},
    Ident, LitStr, Result, Token,
};

impl Parse for RegistrySettings {
    fn parse(input: ParseStream) -> Result<Self> {
        let path_lit: LitStr = input.parse()?;
        let _fat_arrow: Token![=>] = input.parse()?;
        let enum_name: syn::Ident = input.parse()?;
        Ok(RegistrySettings {
            file_path: path_lit.value(),
            registry_enum: enum_name,
        })
    }
}

impl Parse for TextFieldSettings {
    fn parse(input: ParseStream) -> Result<Self> {
        // Парсим строку в кавычках: "Введите значение"
        let placeholder_lit: LitStr = input.parse()?;

        Ok(TextFieldSettings {
            placeholder: placeholder_lit.value(), // Метод .value() возвращает обычный String
        })
    }
}

fn parse_f32(input: ParseStream) -> Result<f32> {
    let lit: syn::Lit = input.parse()?;
    match lit {
        syn::Lit::Float(f) => f.base10_parse(),
        syn::Lit::Int(i) => i.base10_parse(),
        _ => Err(syn::Error::new_spanned(lit, "Ожидалось число")),
    }
}

impl Parse for NumericFieldSettings {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut min = None;
        let mut max = None;

        // Если внутри скобок что-то есть
        if !input.is_empty() {
            min = Some(parse_f32(input)?);

            // Если после первого числа есть запятая
            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;

                // Если после запятой есть еще токены — это max
                if !input.is_empty() {
                    max = Some(parse_f32(input)?);
                }
            }
        }

        Ok(NumericFieldSettings { min, max })
    }
}

impl Parse for BoolFieldSettings {
    fn parse(_input: ParseStream) -> Result<Self> {
        // BoolField не требует параметров, это просто флаг
        Ok(BoolFieldSettings)
    }
}

impl Parse for SliderFieldSettings {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        syn::braced!(content in input);

        // Считываем min (обязательно)
        let min = parse_f32(&content)?;

        // Ожидаем запятую
        content.parse::<Token![,]>()?;

        // Считываем max (обязательно)
        let max = parse_f32(&content)?;

        // Проверяем, есть ли шаг (необязательно, по умолчанию 1.0)
        let step = if content.peek(Token![,]) {
            content.parse::<Token![,]>()?;
            if !content.is_empty() {
                parse_f32(&content)?
            } else {
                1.0
            }
        } else {
            1.0
        };

        Ok(SliderFieldSettings { min, max, step })
    }
}

impl Parse for WidgetType {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;

        match name.to_string().as_str() {
            "Registry" => {
                if input.peek(syn::token::Brace) {
                    let content;
                    syn::braced!(content in input);
                    // Парсим настройки из скобок
                    let settings: RegistrySettings = content.parse()?;
                    Ok(WidgetType::Registry(settings))
                } else {
                    Err(input.error("Can`t find config"))
                }
            }
            "NumericField" => {
                if input.peek(syn::token::Brace) {
                    let content;
                    syn::braced!(content in input);
                    let settings: NumericFieldSettings = content.parse()?;
                    Ok(WidgetType::NumericField(settings))
                } else {
                    Err(input.error("Can`t find config"))
                }
            }
            "TextField" => {
                if input.peek(syn::token::Brace) {
                    let content;
                    syn::braced!(content in input);
                    let settings: TextFieldSettings = content.parse()?;
                    Ok(WidgetType::TextField(settings))
                } else {
                    Err(input.error("Can`t find config"))
                }
            }
            "BoolField" => {
                if input.peek(syn::token::Brace) {
                    let content;
                    syn::braced!(content in input);
                    let settings: BoolFieldSettings = content.parse()?;
                    Ok(WidgetType::BoolField(settings))
                } else {
                    Err(input.error("Can`t find config"))
                }
            }
            "SliderField" => Ok(WidgetType::SliderField(input.parse()?)),
            "Wrapper" => {
                if input.peek(syn::token::Paren) {
                    let content;
                    syn::parenthesized!(content in input);
                    let wrapper_ident: Ident = content.parse()?;
                    let _: Token![,] = content.parse()?;
                    let inner_widget: WidgetType = content.parse()?;
                    Ok(WidgetType::Wrapper(wrapper_ident, Box::new(inner_widget)))
                } else {
                    Err(input.error(
                        "Can`t find wrapper params. Usage: Wrapper(WrapperType, InnerWidget)",
                    ))
                }
            }
            "WrapperVec" => {
                if input.peek(syn::token::Paren) {
                    let content;
                    syn::parenthesized!(content in input);
                    let inner_widget: WidgetType = content.parse()?;
                    Ok(WidgetType::WrapperVec(Box::new(inner_widget)))
                } else {
                    Err(input
                        .error("Can`t find wrapper vec params. Usage: WrapperVec(InnerWidget)"))
                }
            }
            _ => Err(input.error(format!("Unknown widget_type type: {}", name))),
        }
    }
}

impl Parse for FieldSetting {
    fn parse(input: ParseStream) -> Result<Self> {
        let field_name: Ident = input.parse()?;

        // Читаем двоеточие. Если его там нет, syn сам выкинет красивую ошибку.
        let _colon_token: Token![:] = input.parse()?;

        let widget_type: WidgetType = input.parse()?;

        Ok(FieldSetting {
            field_name,
            widget_type,
        })
    }
}

impl Parse for StructConfig {
    fn parse(input: ParseStream) -> Result<Self> {
        // Сначала проверяем условие
        let mut condition = None;
        if input.peek(Token![@]) {
            let _: Token![@] = input.parse()?;
            let _name: Ident = input.parse()?; // show_if
            let content;
            syn::parenthesized!(content in input);
            condition = Some(content.parse()?);
        }

        // Имя структуры и стрелка
        let struct_name: Ident = input.parse()?;
        let _fat_arrow_token: Token![=>] = input.parse()?; // Четко забираем =>

        // Тело в фигурных скобках
        let content;
        syn::braced!(content in input);

        // Используем встроенный метод парсинга разделенного списка
        let fields = content.parse_terminated(FieldSetting::parse, Token![,])?;

        Ok(StructConfig {
            struct_name,
            fields,
            condition,
        })
    }
}

impl Parse for UiGenInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let file_path = input.parse()?;
        let _fat_arrow_token: Token![=>] = input.parse()?;
        syn::braced!(content in input);
        Ok(UiGenInput {
            file_path,
            configs: content.parse_terminated(StructConfig::parse, Token![,])?,
        })
    }
}

impl Parse for PathList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut inputs = Vec::new();
        while !input.is_empty() {
            inputs.push(input.parse()?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(PathList { inputs })
    }
}
