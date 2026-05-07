// Процесс генерации теневых структур для типа `enum`

use crate::utils::name_to_snake_case;
use proc_macro2;
use quote::quote;
use syn::ItemEnum;

pub fn gen_enum(enum_: &ItemEnum) -> proc_macro2::TokenStream {
    let name_ident = &enum_.ident;
    let shadow_name_ident = quote::format_ident!("{}Shadow", name_ident);
    // преобразуем имя к snake_case
    let name_str = name_to_snake_case(name_ident.to_string());
    let visibility = &enum_.vis;

    // Получаем список идентификаторов
    let variant_idents: Vec<_> = enum_.variants.iter().map(|v| &v.ident).collect();

    // Разделяем на первый и остальные
    if let Some((first_variant, rest_variants)) = variant_idents.split_first() {
        return proc_macro2::TokenStream::from(quote! {
            #[derive(::config_gen::__private::serde::Deserialize, ::config_gen::__private::serde::Serialize, Default, Clone, Copy, Debug, PartialEq)]
            #[serde(crate = "::config_gen::__private::serde", rename = #name_str)]
            #visibility enum #shadow_name_ident {
                #[default]
                #first_variant,
                #( #rest_variants ),*
            }

            // Для отображения на экране и отладки
            impl std::fmt::Display for #shadow_name_ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #( #shadow_name_ident::#variant_idents => write!(f, "{}", stringify!(#variant_idents)) ),*
                    }
                }
            }

            // From Shadow -> to Original
            impl From<#shadow_name_ident> for #name_ident {
                fn from(shadow: #shadow_name_ident) -> Self {
                    match shadow {
                        #( #shadow_name_ident::#variant_idents => #name_ident::#variant_idents ),*
                    }
                }
            }
        });
    } // else будет только при enum с 0 вариантов, что не возможно

    // Но на всякий случай вернём пустой токен
    return proc_macro2::TokenStream::new();
}
