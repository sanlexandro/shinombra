use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet;
use std::fs;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, File, Item, LitStr, Token};

#[proc_macro] // Обрати внимание: это просто proc_macro, не attribute
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

                    #[derive(::config_gen::__private::serde::Deserialize, Default)]
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
        if let syn::Type::Path(ref mut tp) = new_ty {
            if let Some(last_segment) = tp.path.segments.last_mut() {
                let ident_str = last_segment.ident.to_string();
                if all_struct_names.contains(&ident_str) {
                    last_segment.ident = quote::format_ident!("{}Shadow", last_segment.ident);
                }
            }
        }
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
                    let name_str = name.to_string();
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

                    // Генерируем From с .into() для рекурсии
                    let from_body = match &s.fields {
                        syn::Fields::Named(f) => {
                            let mapping = f.named.iter().map(|field| {
                                let f_name = &field.ident;
                                quote! { #f_name: shadow.#f_name.into() }
                            });
                            quote! { Self { #( #mapping, )* } }
                        }
                        syn::Fields::Unnamed(f) => {
                            let mapping = (0..f.unnamed.len()).map(|i| {
                                let idx = syn::Index::from(i);
                                quote! { shadow.#idx.into() }
                            });
                            quote! { Self ( #( #mapping ),* )
                            }
                        }
                        syn::Fields::Unit => quote! { Self },
                    };

                    final_tokens.push(quote! {
                        #s // Оригинал (чистый)

                        #[derive(::config_gen::__private::serde::Deserialize, Default)]
                        #[serde(crate = "::config_gen::__private::serde", rename = #name_str)]
                        #vis struct #shadow_name #shadow_fields

                        impl From<#shadow_name> for #name {
                            fn from(shadow: #shadow_name) -> Self {
                                #from_body
                            }
                        }
                    });
                }
                syn::Item::Enum(ref e) => {
                    let name = &e.ident;
                    let shadow_name = quote::format_ident!("{}Shadow", name);
                    let name_str = name.to_string();
                    let vis = &e.vis;

                    // Получаем список идентификаторов
                    let variant_idents: Vec<_> = e.variants.iter().map(|v| &v.ident).collect();

                    // Разделяем на первый и остальные
                    if let Some((first_variant, rest_variants)) = variant_idents.split_first() {
                        final_tokens.push(quote! {
                            #e // Оригинал

                            #[derive(::config_gen::__private::serde::Deserialize, Default)]
                            #[serde(crate = "::config_gen::__private::serde", rename = #name_str)]
                            #vis enum #shadow_name {
                                #[default]
                                #first_variant,
                                #( #rest_variants ),*
                            }

                            impl From<#shadow_name> for #name {
                                fn from(shadow: #shadow_name) -> Self {
                                    match shadow {
                                        #( #shadow_name::#variant_idents => #name::#variant_idents ),*
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
        let field_name = quote::format_ident!("{}", name.to_lowercase()); // Простой snake_case
        let ty = quote::format_ident!("{}Shadow", name);
        quote! { pub #field_name: Option<#ty> }
    });

    final_tokens.push(quote! {
        #[derive(::config_gen::__private::serde::Deserialize, Default)]
        #[serde(crate = "::config_gen::__private::serde")]
        pub struct FullConfigShadow {
            #( #fields, )*
        }
    });

    TokenStream::from(quote! { #( #final_tokens )* })
}
