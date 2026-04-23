use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet;
use std::fs;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, File, Item, LitStr, Token};

// TODO! Комментарии к коду!

#[proc_macro]
pub fn include_shadow(input: TokenStream) -> TokenStream {
    // 1. Получаем путь из макроса include_shadow!("path/to/file.rs")
    let path_lit = parse_macro_input!(input as LitStr);
    let path = path_lit.value();

    // 2. Читаем файл (путь относительно Cargo.toml проекта, где запускается макрос)
    let content =
        fs::read_to_string(&path).expect(&format!("Не удалось прочитать файл по пути: {}", path));

    // 3. Парсим содержимое как файл Rust
    let syntax_tree = syn::parse_str::<File>(&content).expect("Ошибка парсинга файла конфига");

    let mut expanded_items = Vec::new();

    // 4. Проходим по всем элементам файла
    for item in syntax_tree.items {
        match item {
            Item::Struct(ref s) => {
                let name = &s.ident;
                let shadow_name = quote::format_ident!("{}Shadow", name);
                let vis = &s.vis;
                let fields = &s.fields;

                let expanded = match fields {
                    // 1. Обычные структуры: struct Geo { x: u32 }
                    syn::Fields::Named(named) => {
                        let idents = named.named.iter().map(|f| &f.ident);
                        quote! {
                            #vis struct #shadow_name #fields
                            impl From<#shadow_name> for #name {
                                fn from(shadow: #shadow_name) -> Self {
                                    Self { #( #idents: shadow.#idents ),* }
                                }
                            }
                        }
                    }
                    // 2. Кортежные структуры: struct Millimeters(pub u32)
                    syn::Fields::Unnamed(unnamed) => {
                        let indices = (0..unnamed.unnamed.len()).map(syn::Index::from);
                        quote! {
                            #vis struct #shadow_name #fields;
                            impl From<#shadow_name> for #name {
                                fn from(shadow: #shadow_name) -> Self {
                                    Self ( #( shadow.#indices ),* )
                                }
                            }
                        }
                    }
                    // 3. Unit-структуры: struct Empty;
                    syn::Fields::Unit => {
                        quote! {
                            #vis struct #shadow_name;
                            impl From<#shadow_name> for #name {
                                fn from(_: #shadow_name) -> Self { Self }
                            }
                        }
                    }
                };

                expanded_items.push(quote! {
                    #item // Оригинал

                    #[derive(::config_gen::__private::serde::Deserialize, ::config_gen::__private::serde::Serialize, Default)]
                    #[serde(crate = "::config_gen::__private::serde")]
                    #expanded
                });
            }
            Item::Use(_) => {
                // Просто игнорируем импорты из файла,
                // так как в main.rs они могут быть невалидны
                continue;
            }
            _ => expanded_items.push(quote! { #item }),
        }
    }

    TokenStream::from(quote! {
        #( #expanded_items )*
    })
}

// Вспомогательная структура для парсинга нескольких путей: include_shadow_all!("a.rs", "b.rs")
struct PathList {
    paths: Vec<LitStr>,
}

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

#[proc_macro]
pub fn include_shadow_all(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as PathList);
    let mut all_struct_names = HashSet::new();
    let mut all_items = Vec::new();

    // Шаг 1: Собираем все имена структур из всех файлов
    for path_lit in &input.paths {
        let path = path_lit.value();
        let content = fs::read_to_string(&path).expect("File not found");
        let file = syn::parse_str::<File>(&content).expect("Parse error");

        for item in &file.items {
            if let Item::Struct(s) = item {
                all_struct_names.insert(s.ident.to_string());
            } else if let Item::Enum(s) = item {
                all_struct_names.insert(s.ident.to_string());
            }
        }
        all_items.push(file.items);
    }

    // Вспомогательная функция для трансформации типов на лету
    let transform_ty = |ty: &syn::Type| -> syn::Type {
        let mut new_ty = ty.clone();

        fn walk_type(ty: &mut syn::Type, names: &HashSet<String>) {
            match ty {
                syn::Type::Path(ref mut tp) => {
                    // 1. Проверяем сам тип (например, ColorFilterType -> ColorFilterTypeShadow)
                    if let Some(last_segment) = tp.path.segments.last_mut() {
                        let ident_str = last_segment.ident.to_string();
                        if names.contains(&ident_str) {
                            last_segment.ident =
                                quote::format_ident!("{}Shadow", last_segment.ident);
                        }

                        // 2. РЕКУРСИЯ: Проверяем generic-аргументы (например, Vec<ColorFilterType>)
                        if let syn::PathArguments::AngleBracketed(ref mut args) =
                            last_segment.arguments
                        {
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

        walk_type(&mut new_ty, &all_struct_names);
        new_ty
    };

    let mut final_tokens = Vec::new();

    // Шаг 2: Генерируем код
    for items in all_items {
        for item in items {
            match item {
                Item::Struct(s) => {
                    let name = &s.ident;
                    let shadow_name = quote::format_ident!("{}Shadow", name);
                    // преобразуем имя к snake_case
                    let mut name_str = name.to_string();
                    for (i, ch) in name.to_string().chars().enumerate() {
                        if ch.is_uppercase() && i != 0 {
                            name_str.push('_');
                        }
                        name_str.push(ch.to_ascii_lowercase());
                    }
                    let vis = &s.vis;

                    // Генерируем поля для Shadow, трансформируя типы
                    let shadow_fields = match &s.fields {
                        syn::Fields::Named(f) => {
                            let fields = f.named.iter().map(|field| {
                                let f_name = &field.ident;
                                let f_vis = &field.vis;
                                let f_ty = transform_ty(&field.ty);
                                quote! {
                                    #[serde(default)]
                                    #f_vis #f_name: #f_ty
                                }
                            });
                            quote! { { #( #fields, )* } }
                        }
                        syn::Fields::Unnamed(f) => {
                            let fields = f.unnamed.iter().map(|field| {
                                let f_vis = &field.vis;
                                let f_ty = transform_ty(&field.ty);
                                quote! {
                                    #[serde(default)]
                                    #f_vis #f_ty
                                }
                            });
                            quote! { ( #( #fields ),* ); }
                        }
                        syn::Fields::Unit => quote! { ; },
                    };

                    // Вспомогательная функция для генерации конвертации поля
                    let gen_conversion = |field_tokens: proc_macro2::TokenStream,
                                          ty: &syn::Type| {
                        if let syn::Type::Path(tp) = ty {
                            if let Some(seg) = tp.path.segments.last() {
                                if seg.ident == "Vec" {
                                    // Если это Vec, конвертируем каждый элемент
                                    return quote! { #field_tokens.into_iter().map(|v| v.into()).collect() };
                                }
                            }
                        }
                        // Для обычных типов оставляем как было
                        quote! { #field_tokens.into() }
                    };

                    // Генерируем From с .into() для рекурсии
                    let from_body = match &s.fields {
                        syn::Fields::Named(f) => {
                            let mapping = f.named.iter().map(|field| {
                                let f_name = &field.ident;
                                let conv = gen_conversion(quote!(shadow.#f_name), &field.ty);
                                quote! { #f_name: #conv }
                            });
                            quote! { Self { #( #mapping, )* } }
                        }
                        syn::Fields::Unnamed(f) => {
                            let mapping = f.unnamed.iter().enumerate().map(|(i, field)| {
                                let idx = syn::Index::from(i);
                                let conv = gen_conversion(quote!(shadow.#idx), &field.ty);
                                quote! { #conv }
                            });
                            quote! { Self ( #( #mapping ),* ) }
                        }
                        syn::Fields::Unit => quote! { Self },
                    };

                    // Генерируем From для ссылки &Shadow -> Original
                    let from_ref_body = match &s.fields {
                        syn::Fields::Named(f) => {
                            let mapping = f.named.iter().map(|field| {
                                let f_name = &field.ident;
                                let conv = gen_conversion(quote!(shadow.#f_name.clone()), &field.ty);
                                quote! { #f_name: #conv }
                            });
                            quote! { Self { #( #mapping, )* } }
                        }
                        syn::Fields::Unnamed(f) => {
                            let mapping = f.unnamed.iter().enumerate().map(|(i, field)| {
                                let idx = syn::Index::from(i);
                                let conv = gen_conversion(quote!(shadow.#idx.clone()), &field.ty);
                                quote! { #conv }
                            });
                            quote! { Self ( #( #mapping ),* ) }
                        }
                        syn::Fields::Unit => quote! { Self },
                    };

                    // Подготавливаем тело для From<Original> for Shadow
                    let to_shadow_body = match &s.fields {
                        syn::Fields::Named(f) => {
                            let mapping = f.named.iter().map(|field| {
                                let f_name = &field.ident;
                                let conv = gen_conversion(quote!(orig.#f_name), &field.ty);
                                quote! { #f_name: #conv }
                            });
                            quote! { Self { #( #mapping, )* } }
                        }
                        syn::Fields::Unnamed(f) => {
                            let mapping = f.unnamed.iter().enumerate().map(|(i, field)| {
                                let idx = syn::Index::from(i);
                                let conv = gen_conversion(quote!(orig.#idx), &field.ty);
                                quote! { #conv }
                            });
                            quote! { Self ( #( #mapping ),* ) }
                        }
                        syn::Fields::Unit => quote! { Self },
                    };

                    final_tokens.push(quote! {
                        #[derive(::config_gen::__private::serde::Deserialize, ::config_gen::__private::serde::Serialize, Default, Clone, Debug, PartialEq)]
                        #[serde(crate = "::config_gen::__private::serde", rename = #name_str)]
                        #vis struct #shadow_name #shadow_fields

                        // Impl Display для использования в шаблонах
                        impl std::fmt::Display for #shadow_name {
                            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                                write!(f, "{:?}", self)
                            }
                        }

                        // Конвертация по значению (потребляет Shadow)
                        impl From<#shadow_name> for #name {
                            fn from(shadow: #shadow_name) -> Self {
                                #from_body
                            }
                        }

                        // Конвертация по ссылке (удобно для твоего "let")
                        impl From<&#shadow_name> for #name {
                            fn from(shadow: &#shadow_name) -> Self {
                                #from_ref_body
                            }
                        }

                        // И в обратную сторону
                        impl From<#name> for #shadow_name {
                            fn from(orig: #name) -> Self {
                                #to_shadow_body
                            }
                        }
                    });
                }
                syn::Item::Enum(ref e) => {
                    let name = &e.ident;
                    let shadow_name = quote::format_ident!("{}Shadow", name);
                    // преобразуем имя к snake_case
                    let mut name_str = name.to_string();
                    for (i, ch) in name.to_string().chars().enumerate() {
                        if ch.is_uppercase() && i != 0 {
                            name_str.push('_');
                        }
                        name_str.push(ch.to_ascii_lowercase());
                    }
                    let vis = &e.vis;

                    // Получаем список идентификаторов
                    let variant_idents: Vec<_> = e.variants.iter().map(|v| &v.ident).collect();

                    // Разделяем на первый и остальные
                    if let Some((first_variant, rest_variants)) = variant_idents.split_first() {
                        final_tokens.push(quote! {
                            #[derive(::config_gen::__private::serde::Deserialize, ::config_gen::__private::serde::Serialize, Default, Clone, Copy, Debug, PartialEq)]
                            #[serde(crate = "::config_gen::__private::serde", rename = #name_str)]
                            #vis enum #shadow_name {
                                #[default]
                                #first_variant,
                                #( #rest_variants ),*
                            }

                            // Impl Display для использования в шаблонах
                            impl std::fmt::Display for #shadow_name {
                                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                                    match self {
                                        #( #shadow_name::#variant_idents => write!(f, "{}", stringify!(#variant_idents)) ),*
                                    }
                                }
                            }

                            impl From<#shadow_name> for #name {
                                fn from(shadow: #shadow_name) -> Self {
                                    match shadow {
                                        #( #shadow_name::#variant_idents => #name::#variant_idents ),*
                                    }
                                }
                            }

                            impl From<#name> for #shadow_name {
                                fn from(orig: #name) -> Self {
                                    match orig {
                                        #( #name::#variant_idents => #shadow_name::#variant_idents ),*
                                    }
                                }
                            }
                        });
                    }
                }
                Item::Use(_) => continue, // Пропускаем импорты
                _ => final_tokens.push(quote! { #item }),
            }
        }
    }

    // Внутри макроса, когда собрали все all_struct_names
    let fields = all_struct_names.iter().map(|name| {
        let field_name = quote::format_ident!("{}", {
            // преобразуем имя к snake_case
            let mut snake = String::new();
            for (i, ch) in name.chars().enumerate() {
                if ch.is_uppercase() && i != 0 {
                    snake.push('_');
                }
                snake.push(ch.to_ascii_lowercase());
            }
            snake
        }); // Простой snake_case
        let ty = quote::format_ident!("{}Shadow", name);
        quote! { pub #field_name: Option<#ty> }
    });

    final_tokens.push(quote! {
        #[derive(::config_gen::__private::serde::Deserialize, ::config_gen::__private::serde::Serialize, Default)]
        #[serde(crate = "::config_gen::__private::serde")]
        pub struct FullConfigShadow {
            #( #fields, )*
        }
    });

    TokenStream::from(quote! { #( #final_tokens )* })
}
