//! Функции и трейты для генерации HTML-кода
//!
//! Отвечает за преобразование разобранных настроек (AST) в готовые
//! статические и динамические HTML-элементы для веб-интерфейса.

use quote::quote;
use std::fs;

use proc_macro2;
use syn::File;

use crate::{
    types::{BoolFieldSettings, NumericFieldSettings, RegistrySettings, TextFieldSettings},
    FieldSetting, SliderFieldSettings, WidgetType,
};

/// Общий трейт для виджетов, умеющих генерировать HTML представление
trait GenerateInputHTML {
    fn gen_html(
        &self,
        struct_name: String,
        field_name: String,
        access_path: proc_macro2::TokenStream,
        array_index: Option<proc_macro2::TokenStream>,
    ) -> (proc_macro2::TokenStream, String);
}

/// Генерирует уникальный ID элемента для использования в атрибуте `id`
fn gen_field_id(
    struct_name: &String,
    field_name: &String,
    is_array: bool,
    is_static: bool,
) -> String {
    if is_array {
        if is_static {
            format!("{}_{}___INDEX__", struct_name, field_name)
        } else {
            format!("{}_{}_{{}}", struct_name, field_name)
        }
    } else {
        format!("{}_{}", struct_name, field_name)
    }
}

/// Генерирует имя поля для отправки формы (атрибут `name`)
fn gen_field_name(
    struct_name: &String,
    field_name: &String,
    is_array: bool,
    is_static: bool,
) -> String {
    if is_array {
        if is_static {
            format!("{}[{}][__INDEX__]", struct_name, field_name)
        } else {
            format!("{}[{}][{{}}]", struct_name, field_name)
        }
    } else {
        format!("{}[{}]", struct_name, field_name)
    }
}

impl GenerateInputHTML for RegistrySettings {
    fn gen_html(
        &self,
        struct_name: String,
        field_name: String,
        access_path: proc_macro2::TokenStream,
        array_index: Option<proc_macro2::TokenStream>,
    ) -> (proc_macro2::TokenStream, String) {
        // Читаем файл
        let content = fs::read_to_string(&self.file_path).expect("File not found");

        // Парсим файл как AST Rust
        let file = syn::parse_str::<File>(&content).expect("Parse error");

        // Ищем enum с конкретным именем
        let target_enum = file
            .items
            .iter()
            .find_map(|item| {
                if let syn::Item::Enum(e) = item {
                    if e.ident == self.registry_enum {
                        return Some(e);
                    }
                }
                None
            })
            .expect(&format!("Can`t find enum {}", self.registry_enum));

        let mut static_string = Vec::new();
        let mut logic_token = Vec::new();

        // Вытягиваем всё поля как строки
        for variant in target_enum.variants.iter() {
            let id = variant.ident.to_string();
            let selected = format!(r#"<option value="{}" selected>{}</option>"#, id, id);
            let not_selected = format!(r#"<option value="{}">{}</option>"#, id, id);

            let shadow_registry_ident = quote::format_ident!("{}Shadow", self.registry_enum);
            let id_ident = quote::format_ident!("{}", id);

            static_string.push(not_selected.clone());
            logic_token.push(quote! {
                if #access_path == #shadow_registry_ident::#id_ident {
                    html.push(String::from(#selected));
                } else {
                    html.push(String::from(#not_selected));
                }
            });
        }

        let first_string = format!(
            r#"<select id="{}" name="{}" required>"#,
            gen_field_id(&struct_name, &field_name, array_index.is_some(), false),
            gen_field_name(&struct_name, &field_name, array_index.is_some(), false)
        );

        let static_first_string = format!(
            r#"<select id="{}" name="{}" required>"#,
            gen_field_id(&struct_name, &field_name, array_index.is_some(), true),
            gen_field_name(&struct_name, &field_name, array_index.is_some(), true)
        );

        let last_string = "</select>".to_string();

        let create_string =
            static_first_string.to_string() + &static_string.join("\n") + &last_string.to_string();

        // Возвращаем обёртку в select
        let ts = if let Some(idx) = &array_index {
            proc_macro2::TokenStream::from(quote! {
                html.push(format!(#first_string, #idx, #idx)); // Вставляем первую строку
                #( #logic_token )*
                html.push(String::from(#last_string)); // Вставляем последнюю строку
            })
        } else {
            proc_macro2::TokenStream::from(quote! {
                html.push(String::from(#first_string)); // Вставляем первую строку
                #( #logic_token )*
                html.push(String::from(#last_string)); // Вставляем последнюю строку
            })
        };

        return (ts, create_string);
    }
}

impl GenerateInputHTML for NumericFieldSettings {
    fn gen_html(
        &self,
        struct_name: String,
        field_name: String,
        access_path: proc_macro2::TokenStream,
        array_index: Option<proc_macro2::TokenStream>,
    ) -> (proc_macro2::TokenStream, String) {
        // Собираем атрибуты только если значения существуют
        let mut attrs: Vec<String> = Vec::new();

        if let Some(min) = self.min {
            attrs.push(format!(r#"min="{}""#, min));
        }
        if let Some(max) = self.max {
            attrs.push(format!(r#"max="{}""#, max));
        }

        let attrs_str = if attrs.is_empty() {
            String::new()
        } else {
            format!(" {}", attrs.join(" "))
        };

        // Статическая строка
        let static_string = format!(
            r#"<input type="number" id="{}" name="{}" {} required />"#,
            gen_field_id(&struct_name, &field_name, array_index.is_some(), true),
            gen_field_name(&struct_name, &field_name, array_index.is_some(), true),
            attrs_str
        );
        // Строка, готовая для динамической обработки
        let logic_token = format!(
            r#"<input type="number" id="{}" name="{}" {} value="{{}}" required />"#,
            gen_field_id(&struct_name, &field_name, array_index.is_some(), false),
            gen_field_name(&struct_name, &field_name, array_index.is_some(), false),
            attrs_str
        );

        let ts = if let Some(idx) = &array_index {
            proc_macro2::TokenStream::from(
                quote! { html.push(format!(#logic_token, #idx, #idx, #access_path)); },
            )
        } else {
            proc_macro2::TokenStream::from(
                quote! { html.push(format!(#logic_token, #access_path)); },
            )
        };

        return (ts, static_string);
    }
}

impl GenerateInputHTML for TextFieldSettings {
    fn gen_html(
        &self,
        struct_name: String,
        field_name: String,
        access_path: proc_macro2::TokenStream,
        array_index: Option<proc_macro2::TokenStream>,
    ) -> (proc_macro2::TokenStream, String) {
        // Статическая строка
        let static_string = format!(
            r#"<input type="text" id="{}" name="{}" placeholder="{}" required />"#,
            gen_field_id(&struct_name, &field_name, array_index.is_some(), true),
            gen_field_name(&struct_name, &field_name, array_index.is_some(), true),
            self.placeholder
        );
        // Строка для динамической обработки
        let logic_token = format!(
            r#"<input type="text" id="{}" name="{}" placeholder="{}" value="{{}}" required />"#,
            gen_field_id(&struct_name, &field_name, array_index.is_some(), false),
            gen_field_name(&struct_name, &field_name, array_index.is_some(), false),
            self.placeholder
        );

        let ts = if let Some(idx) = &array_index {
            proc_macro2::TokenStream::from(
                quote! { html.push(format!(#logic_token, #idx, #idx, #access_path)); },
            )
        } else {
            proc_macro2::TokenStream::from(
                quote! { html.push(format!(#logic_token, #access_path)); },
            )
        };

        return (ts, static_string);
    }
}

impl GenerateInputHTML for BoolFieldSettings {
    fn gen_html(
        &self,
        struct_name: String,
        field_name: String,
        access_path: proc_macro2::TokenStream,
        array_index: Option<proc_macro2::TokenStream>,
    ) -> (proc_macro2::TokenStream, String) {
        // Статическая строка для режима Create (без checked)
        let static_string = format!(
            r#"<input type="checkbox" id="{}" name="{}" />"#,
            gen_field_id(&struct_name, &field_name, array_index.is_some(), true),
            gen_field_name(&struct_name, &field_name, array_index.is_some(), true)
        );

        // Строка для динамической обработки с условным checked атрибутом
        let logic_token_template = format!(
            r#"<input type="checkbox" id="{}" name="{}" {{}}/>"#,
            gen_field_id(&struct_name, &field_name, array_index.is_some(), false),
            gen_field_name(&struct_name, &field_name, array_index.is_some(), false)
        );

        let ts = if let Some(idx) = &array_index {
            proc_macro2::TokenStream::from(quote! {
                let checked_attr = if #access_path { "checked " } else { "" };
                html.push(format!(#logic_token_template, #idx, #idx, checked_attr));
            })
        } else {
            proc_macro2::TokenStream::from(quote! {
                let checked_attr = if #access_path { "checked " } else { "" };
                html.push(format!(#logic_token_template, checked_attr));
            })
        };

        return (ts, static_string);
    }
}

impl GenerateInputHTML for SliderFieldSettings {
    fn gen_html(
        &self,
        struct_name: String,
        field_name: String,
        access_path: proc_macro2::TokenStream,
        array_index: Option<proc_macro2::TokenStream>,
    ) -> (proc_macro2::TokenStream, String) {
        let is_arr = array_index.is_some();
        let id_pattern = gen_field_id(&struct_name, &field_name, is_arr, false);
        let name_pattern = gen_field_name(&struct_name, &field_name, is_arr, false);

        let min = self.min;
        let max = self.max;
        let step = self.step;

        // Статический HTML
        let html_template = format!(
            r#"<div class="slider-group">
        <input type="number" name="{}" class="slider-num" min="{}" max="{}" step="{}" id="{}"
            oninput="this.parentElement.querySelector('.slider-range').value = this.value">
        <div class="slider-wrapper">
            <span class="slider-limit">{}</span>
            <input type="range" class="slider-range" min="{}" max="{}" step="{}" 
                oninput="this.parentElement.previousElementSibling.value = this.value">
            <span class="slider-limit">{}</span>
        </div>
    </div>"#,
            name_pattern, min, max, step, id_pattern, min, min, max, step, max
        );

        // Динамическая логика
        let dynamic_logic = quote! {
            let val = #access_path;
            let current_id = format!(#id_pattern, #array_index);
            let current_name = format!(#name_pattern, #array_index);

            html.push(format!(
                r#"<div class="slider-group">
                    <input type="number" name="{}" class="slider-num" min="{}" max="{}" step="{}" value="{}" id="{}"
                        oninput="this.parentElement.querySelector('.slider-range').value = this.value">
                    <div class="slider-wrapper">
                        <span class="slider-limit">{}</span>
                        <input type="range" class="slider-range" min="{}" max="{}" step="{}" value="{}" 
                            oninput="this.parentElement.previousElementSibling.value = this.value">
                        <span class="slider-limit">{}</span>
                    </div>
                </div>"#,
                current_name, #min, #max, #step, val, current_id, #min, #min, #max, #step, val, #max
            ));
        };

        (dynamic_logic, html_template)
    }
}

impl GenerateInputHTML for WidgetType {
    fn gen_html(
        &self,
        struct_name: String,
        field_name: String,
        access_path: proc_macro2::TokenStream,
        array_index: Option<proc_macro2::TokenStream>,
    ) -> (proc_macro2::TokenStream, String) {
        match self {
            WidgetType::Registry(c) => {
                c.gen_html(struct_name, field_name, access_path, array_index)
            }
            WidgetType::NumericField(c) => {
                c.gen_html(struct_name, field_name, access_path, array_index)
            }
            WidgetType::TextField(c) => {
                c.gen_html(struct_name, field_name, access_path, array_index)
            }
            WidgetType::BoolField(c) => {
                c.gen_html(struct_name, field_name, access_path, array_index)
            }
            WidgetType::SliderField(c) => {
                c.gen_html(struct_name, field_name, access_path, array_index)
            }
            WidgetType::Wrapper(_, inner) => inner.gen_html(
                struct_name,
                field_name,
                quote! { (#access_path).0 },
                array_index,
            ),
            WidgetType::WrapperVec(_) => {
                panic!("WrapperVec cannot generate standard HTML input directly")
            }
            WidgetType::OptionField(_) => {
                panic!("OptionField cannot generate standard HTML input directly")
            }
        }
    }
}

/// Генерирует HTML/логику для поля `Option<T>`.
///
/// Идея: рендерим чекбокс "enabled" плюс сам вложенный виджет, обёрнутый
/// в `<fieldset disabled>` когда значение отсутствует.
///
/// Данные на клиенте отправляются ДВУМЯ плоскими полями формы:
/// - `Struct[field__enabled]` (bool, из самого чекбокса)
/// - `Struct[field]` (значение T в обычном для вложенного виджета формате)
///
/// На бекенде (`apply_json_patch`) это восстанавливается в `Option<T>`:
/// `enabled == false` => `None`, `enabled == true` => `Some(<распарсенное field>)`.
///
/// Вызывается напрямую из `FieldSetting::gen_html`, а не через
/// `GenerateInputHTML for WidgetType`, т.к. этому виджету нужен доступ
/// к самому полю `Option<T>` (для матчинга `Some`/`None`), а не просто
/// к его "распакованному" значению, как остальным виджетам.
///
/// Ограничение: `Option(WrapperVec(...))` не поддерживается (как и вложенный
/// `WrapperVec` внутри `Wrapper`) - `WrapperVec` не реализует `GenerateInputHTML`
/// напрямую и требует отдельной обработки на уровне `FieldSetting`.
pub(crate) fn gen_option_html(
    inner: &WidgetType,
    struct_name: String,
    field_name: String,
    access_path: proc_macro2::TokenStream,
) -> (proc_macro2::TokenStream, String) {
    // ВАЖНО: используем "плоские" имена полей ("field__enabled" / "field"),
    // а НЕ вложенный объект вида "field[enabled]"/"field[value]".
    // Причина: клиентская getFormData() в script.js жёстко трактует имя вида
    // "Struct[field][subkey]" (3 части через [ ]) как элемент МАССИВА (это
    // нужно для WrapperVec), а не как вложенный объект - так что схема
    // {enabled, value} там просто не соберётся правильно. Вместо этого чекбокс
    // получает отдельное имя "Struct[field__enabled]", а сам вложенный виджет
    // использует своё обычное имя "Struct[field]", как будто Option тут ни при
    // чём - тогда getFormData() отработает как для обычного поля.
    let enabled_id = format!("{}_{}_enabled", struct_name, field_name);
    let enabled_name = format!("{}[{}__enabled]", struct_name, field_name);
    let toggle_target = format!("{}_{}_value_fieldset", struct_name, field_name);

    let (inner_logic_some, inner_static) = inner.gen_html(
        struct_name.clone(),
        field_name.clone(),
        quote! { inner_val },
        None,
    );

    // <fieldset disabled> отключает ВСЕ вложенные input/select одним махом,
    // независимо от того, какой конкретно виджет внутри (в т.ч. вложенные
    // Wrapper/несколько input-ов у SliderField). Не нужно вручную обходить
    // потомков, как для WrapperVec.
    //
    // Обёртка ".option-field" - блочный flex-контейнер (по аналогии с
    // .slider-group), а не голый inline <fieldset>: без него checkbox и
    // вложенное поле схлопывались бы в узкую inline-полоску вместо того,
    // чтобы поле растягивалось на всю ширину grid-колонки значения.
    // grid-column: 2 фиксирует, что элемент идёт именно во вторую колонку
    // сетки формы (в отличие от WrapperVec, который растягивается на всю
    // карточку через grid-column: 1 / -1).

    // Статический HTML (режим Create): чекбокс выключен, вложенный fieldset disabled
    let static_string = format!(
        r#"<div class="option-field"><input type="checkbox" id="{enabled_id}" name="{enabled_name}" class="option-toggle" onchange="this.nextElementSibling.disabled = !this.checked" /><fieldset id="{toggle_target}" disabled>{inner_static}</fieldset></div>"#,
    );

    // Динамическая логика (режим Edit): в зависимости от Some/None по-разному
    // рендерим чекбокс (checked/не checked) и disabled-статус fieldset-а
    let dynamic_logic = quote! {
        html.push(String::from(r#"<div class="option-field">"#));
        match &(#access_path) {
            Some(inner_val) => {
                html.push(format!(
                    r#"<input type="checkbox" id="{}" name="{}" class="option-toggle" checked onchange="this.nextElementSibling.disabled = !this.checked" /><fieldset id="{}">"#,
                    #enabled_id, #enabled_name, #toggle_target
                ));
                #inner_logic_some
                html.push(String::from("</fieldset>"));
            }
            None => {
                html.push(format!(
                    r#"<input type="checkbox" id="{}" name="{}" class="option-toggle" onchange="this.nextElementSibling.disabled = !this.checked" /><fieldset id="{}" disabled>"#,
                    #enabled_id, #enabled_name, #toggle_target
                ));
                html.push(String::from(#inner_static));
                html.push(String::from("</fieldset>"));
            }
        }
        html.push(String::from("</div>"));
    };

    (
        proc_macro2::TokenStream::from(dynamic_logic),
        static_string,
    )
}

impl FieldSetting {
    pub(crate) fn gen_html(&self, struct_name: String) -> (proc_macro2::TokenStream, String) {
        let label = format!(
            r#"<label for="{}">{} </label>"#,
            gen_field_id(&struct_name, &self.field_name.to_string(), false, false),
            self.field_name.to_string()
        );

        let field_ident = &self.field_name;

        // В зависимости от типа поля вставляем необходимый html
        let (field_logic, field_static) = match &self.widget_type {
            WidgetType::OptionField(inner) => gen_option_html(
                inner,
                struct_name.clone(),
                self.field_name.to_string(),
                quote! { data.#field_ident },
            ),
            WidgetType::WrapperVec(inner) => {
                let (inner_field_logic, inner_field_static) = inner.gen_html(
                    struct_name.clone(),
                    self.field_name.to_string(),
                    quote! { *item },
                    Some(quote! { _i }),
                );

                let container_id =
                    format!("{}_{}_container", struct_name, self.field_name.to_string());
                let template_html = format!(
                    r#"<div class="wrapper-vec-item">{}<button type="button" onclick="this.parentElement.remove()">Remove</button></div>"#,
                    inner_field_static
                );

                let full_static = format!(
                    r#"<div class="wrapper-vec-container" id="{}">
                        <template id="{}_template">{}</template>
                        <div class="wrapper-vec-items"></div>
                        <button type="button" onclick="addWrapperVecItem('{}')">Add Item</button>
                    </div>"#,
                    container_id, container_id, template_html, container_id
                );

                let full_logic = quote! {
                    html.push(format!("<div class=\"wrapper-vec-container\" id=\"{}\">", #container_id));
                    html.push(format!("<template id=\"{}_template\">{}</template>", #container_id, #template_html));
                    html.push(String::from("<div class=\"wrapper-vec-items\">"));
                    for (_i, item) in data.#field_ident.iter().enumerate() {
                        html.push(String::from("<div class=\"wrapper-vec-item\">"));
                        #inner_field_logic
                        html.push(String::from("<button type=\"button\" onclick=\"this.parentElement.remove()\">Remove</button></div>"));
                    }
                    html.push(String::from("</div>"));
                    html.push(format!("<button type=\"button\" onclick=\"addWrapperVecItem('{}')\">Add Item</button></div>", #container_id));
                };

                return (
                    proc_macro2::TokenStream::from(quote! {
                        html.push(String::from(#label));
                        #full_logic
                    }),
                    label.to_string() + &full_static + "<br/>",
                );
            }
            other => other.gen_html(
                struct_name.clone(),
                self.field_name.to_string(),
                quote! { data.#field_ident },
                None,
            ),
        };

        return (
            proc_macro2::TokenStream::from(quote! {
                html.push(String::from(#label));

                #field_logic
            }),
            label + &field_static,
        );
    }
}
