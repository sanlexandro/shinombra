//! Процесс генерации теневых структур для типа `struct`
//! 
//! Весь процесс генерации завязан на уникальном синтаксисе одного из видов структур:
//! - Именованные ([syn::Fields::Named])
//! - Кортежные / безымянные ([syn::Fields::Unnamed])
//! - Пустые ([syn::Fields::Unit])
//! 
//! В зависимости от типа макрос самостоятельно выбирает синтаксис
//! 

use std::collections::HashSet;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{FieldsNamed, FieldsUnnamed, ItemStruct};

/// Преобразование типа
/// 
/// Рекурсивно ищет типы для замены на теневые копии
fn walk_type(type_: &mut syn::Type, names: &HashSet<String>) {
    match type_ {
        syn::Type::Path(ref mut tp) => {
            // Проверяем сам тип (например, ColorFilterType -> ColorFilterTypeShadow)
            if let Some(last_segment) = tp.path.segments.last_mut() {
                let ident_str = last_segment.ident.to_string();
                if names.contains(&ident_str) {
                    last_segment.ident = quote::format_ident!("{}Shadow", last_segment.ident);
                }

                // Рекурсия: Проверяем generic-аргументы (например, Vec<ColorFilterType>)
                if let syn::PathArguments::AngleBracketed(ref mut args) = last_segment.arguments {
                    for arg in args.args.iter_mut() {
                        if let syn::GenericArgument::Type(ref mut inner_ty) = arg {
                            walk_type(inner_ty, names);
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

/// Обёртка над рекурсивной функцией преобразования типов
fn transform_type(type_: &syn::Type, all_struct_names: &HashSet<String>) -> syn::Type {
    let mut new_type = type_.clone();

    walk_type(&mut new_type, all_struct_names);
    return new_type;
}

/// Вспомогательная ф-я для обработки сложных типов (вроде [Vec])
fn gen_conversion(
    field_tokens: proc_macro2::TokenStream,
    ty: &syn::Type,
) -> proc_macro2::TokenStream {
    if let syn::Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            if seg.ident == "Vec" {
                // Если это Vec, конвертируем каждый элемент
                return quote! { #field_tokens.into_iter().map(|v| v.into()).collect() };
            }
            if seg.ident == "Option" {
                // Если это Option, трансформируем внутреннее значение через .map
                return quote! { #field_tokens.map(|v| v.into()) };
            }
        }
    }
    // Для обычных типов оставляем как было
    quote! { #field_tokens.into() }
}

pub struct GeneratedStruct {
    pub generated_fields: Vec<TokenStream>, // Место для хранения сгенерированных полей структуры
    pub conversions_shadow_to_original: Vec<TokenStream>, // Место для хранения сгенерированных преобразований Shadow -> Original
    pub conversions_ptr_shadow_to_original: Vec<TokenStream>, // Место для хранения сгенерированных преобразований Shadow -> Original
}

/// Создание реализаций именованной структуры
pub fn gen_named_struct(
    fields: &FieldsNamed,
    all_struct_names: &HashSet<String>,
) -> GeneratedStruct {
    let mut generated_fields = Vec::new();
    let mut conversions_shadow_to_original = Vec::new();
    let mut conversions_ptr_shadow_to_original = Vec::new();

    for field in fields.named.iter() {
        // Из каждого поля вытягиваем имя, видимость и тип поля
        let field_name = &field.ident;
        let field_visibility = &field.vis;
        let field_type = transform_type(&field.ty, all_struct_names);

        // Создаём поля новой структуры
        generated_fields.push(quote! {
            #[serde(default)]   // Для удобства навешиваем флаг сериализации
            #field_visibility #field_name: #field_type
        });

        // Создаём преобразование Shadow -> Original
        let conv_sh_to_orig = gen_conversion(quote!(shadow.#field_name), &field.ty);
        conversions_shadow_to_original.push(quote! { #field_name: #conv_sh_to_orig }); // Сохраняем

        // Создаём преобразование &Shadow -> Original
        let conv_ptr_sh_to_orig = gen_conversion(quote!(shadow.#field_name.clone()), &field.ty);
        conversions_ptr_shadow_to_original.push(quote! { #field_name: #conv_ptr_sh_to_orig});

        // МОЖЕТ Создаём преобразование Original -> Shadow
    }

    // Возвращаем готовое
    return GeneratedStruct {
        generated_fields,
        conversions_shadow_to_original,
        conversions_ptr_shadow_to_original,
    };
}

/// Создание реализаций кортежных структур
pub fn gen_unnamed_struct(
    fields: &FieldsUnnamed,
    all_struct_names: &HashSet<String>,
) -> GeneratedStruct {
    let mut generated_fields = Vec::new();
    let mut conversions_shadow_to_original = Vec::new();
    let mut conversions_ptr_shadow_to_original = Vec::new();

    for (index, field) in fields.unnamed.iter().enumerate() {
        // Вытягиваем видимость и тип поля
        let field_visibility = &field.vis;
        let field_type = transform_type(&field.ty, all_struct_names);

        // Создаём поля новой структуры
        generated_fields.push(quote! {
            #[serde(default)]
            #field_visibility #field_type
        });

        // Используем syn::Index для правильной работы с tuple indices в quote!
        let idx = syn::Index::from(index);

        // Создаём преобразование Shadow -> Original
        let conv_sh_to_orig = gen_conversion(quote!(shadow.#idx), &field.ty);
        conversions_shadow_to_original.push(conv_sh_to_orig);

        // Создаём преобразование &Shadow -> Original
        let conv_ptr_sh_to_orig = gen_conversion(quote!(shadow.#idx.clone()), &field.ty);
        conversions_ptr_shadow_to_original.push(conv_ptr_sh_to_orig);
    }

    return GeneratedStruct {
        generated_fields,
        conversions_shadow_to_original,
        conversions_ptr_shadow_to_original,
    };
}

/// Генерация кода теневой структуры
pub fn gen_struct(
    struct_: &ItemStruct,
    all_struct_names: &HashSet<String>,
) -> proc_macro2::TokenStream {
    // Получаем имя и видимость
    let name_ident = &struct_.ident;
    let shadow_name_ident = quote::format_ident!("{}Shadow", name_ident);
    let visibility = &struct_.vis;

    // Создаём основной код под каждую структуру
    let generated: GeneratedStruct = match &struct_.fields {
        syn::Fields::Named(fields) => gen_named_struct(&fields, all_struct_names),
        syn::Fields::Unnamed(fields) => gen_unnamed_struct(&fields, all_struct_names),
        syn::Fields::Unit => GeneratedStruct {
            generated_fields: Vec::new(),
            conversions_shadow_to_original: Vec::new(),
            conversions_ptr_shadow_to_original: Vec::new(),
        },
    };

    let fields_tokens = &generated.generated_fields;
    let conversions = &generated.conversions_shadow_to_original;
    let conversions_ptr = &generated.conversions_ptr_shadow_to_original;

    // Выбираем обёртку
    let (from_conversions, from_conversions_ptr, struct_def) = match &struct_.fields {
        // Именованные структуры требуют `{}`
        syn::Fields::Named(_) => (
            quote! {Self{ #(#conversions),*}},
            quote! {Self{ #(#conversions_ptr),*}},
            quote! {{
                #(#fields_tokens),*
            }},
        ),
        // Кортежные - `()`
        syn::Fields::Unnamed(_) => (
            quote! {Self(#(#conversions),*)},
            quote! {Self(#(#conversions_ptr),*)},
            quote! {(#(#fields_tokens),*);},
        ),
        // Пустая вообще ничего не требует
        syn::Fields::Unit => (quote! {Self}, quote! {Self}, quote! {;}),
    };

    // Собираем структуру, навешивая необходимые трейты, а также пишем
    // реализацию трейта From
    return TokenStream::from(quote! {
        #[derive(::config_gen::__private::serde::Deserialize, ::config_gen::__private::serde::Serialize, Default, Clone, Debug, PartialEq)]
        #[serde(crate = "::config_gen::__private::serde")]
        #visibility struct #shadow_name_ident #struct_def

        // Для отображения на экране и отладки
        impl std::fmt::Display for #shadow_name_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }

        // From Shadow -> to Original
        impl From<#shadow_name_ident> for #name_ident {
            fn from(shadow: #shadow_name_ident) -> Self {
                #from_conversions
            }
        }

        // From &Shadow -> to Original
        impl From<&#shadow_name_ident> for #name_ident {
            fn from(shadow: &#shadow_name_ident) -> Self {
                #from_conversions_ptr
            }
        }
    });
}
