mod quoting;

use modify::{FieldInformation, ModType};
use proc_macro2::Span;
use proc_macro_error::{abort, proc_macro_error};
use quote::{quote, ToTokens};
use std::collections::HashMap;
use syn::{parse_quote, spanned::Spanned};

use crate::quoting::quote_field_modifiers;

#[proc_macro_derive(Modify, attributes(modifier))]
#[proc_macro_error]
pub fn derive_validation(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_modify(&ast).into()
}

fn impl_modify(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let ident = &ast.ident;
    let fields = collect_field_modifiers(ast);
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let (modifiers, _) = quote_field_modifiers(fields);
    quote!(
        impl #impl_generics ::modify::Modify for #ident #ty_generics #where_clause {
            fn modify(&mut self) {
                #(#modifiers)*
            }
    })
}

fn collect_field_modifiers(ast: &syn::DeriveInput) -> Vec<FieldInformation> {
    let mut fields = collect_fields(ast);

    let field_types = map_field_types(&fields);
    fields.drain(..).fold(vec![], |mut acc, field| {
        let key = field.ident.clone().unwrap().to_string();
        let (name, modifiers) = find_modifiers_for_field(&field, &field_types);
        acc.push(FieldInformation::new(
            field,
            field_types.get(&key).unwrap().clone(),
            name,
            modifiers,
        ));
        acc
    })
}

fn collect_fields(ast: &syn::DeriveInput) -> Vec<syn::Field> {
    match ast.data {
        syn::Data::Struct(syn::DataStruct { ref fields, .. }) => {
            if fields.iter().any(|field| field.ident.is_none()) {
                abort!(
                    fields.span(),
                    "Struct has unnamed fields";
                    help = "#[derive(Modify)] can only be used on structs with named fields";
                );
            }
            fields.iter().cloned().collect()
        }
        _ => abort!(
            ast.span(),
            "#[derive(Modify)] can only be used with structs"
        ),
    }
}

/// Find everything we need to know about a field: its real name if it's changed from the serialization
/// and the list of modifiers to run on it
fn find_modifiers_for_field(
    field: &syn::Field,
    field_types: &HashMap<String, String>,
) -> (String, Vec<ModType>) {
    let rust_ident = field.ident.clone().unwrap().to_string();
    let field_ident = field.ident.clone().unwrap().to_string();

    let error = |span: Span, msg: &str| -> ! {
        abort!(
            span,
            "Invalid attribute #[modifier] on field `{}`: {}",
            field.ident.clone().unwrap().to_string(),
            msg
        );
    };

    let field_type = field_types.get(&field_ident).unwrap();

    let mut modifiers = vec![];
    let mut has_modifiers = false;

    for attr in &field.attrs {
        if attr.path != parse_quote!(modifier) && attr.path != parse_quote!(serde) {
            continue;
        }

        if attr.path == parse_quote!(modifier) {
            has_modifiers = true;
        }

        match attr.parse_meta() {
            Ok(syn::Meta::List(syn::MetaList { ref nested, .. })) => {
                let meta_items = nested.iter().collect::<Vec<_>>();
                // original name before serde rename
                /*                 if attr.path == parse_quote!(serde) {
                    if let Some(s) = find_original_field_name(&meta_items) {
                        field_ident = s;
                    }
                    continue;
                } */

                // Only modifiers from here on
                for meta_item in meta_items {
                    match *meta_item {
                        syn::NestedMeta::Meta(ref item) => match *item {
                            // These contain single word modifiers: trim, upper/lowercase, capitalize
                            // #[modifier(trim)]
                            syn::Meta::Path(ref name) => {
                                match name.get_ident().unwrap().to_string().as_ref() {
                                    "trim" => {
                                        assert_string_type("trim", field_type, &field.ty);
                                        modifiers.push(ModType::Trim);
                                    }
                                    "uppercase" => {
                                        assert_string_type("uppercase", field_type, &field.ty);
                                        modifiers.push(ModType::Uppercase);
                                    }
                                    "lowercase" => {
                                        assert_string_type("lowercase", field_type, &field.ty);
                                        modifiers.push(ModType::Lowercase);
                                    }
                                    "capitalize" => {
                                        assert_string_type("capitalize", field_type, &field.ty);
                                        modifiers.push(ModType::Capitalize);
                                    }
                                    _ => {
                                        let mut ident = proc_macro2::TokenStream::new();
                                        name.to_tokens(&mut ident);
                                        abort!(name.span(), "Unexpected modifier: {}", ident)
                                    }
                                }
                            }
                            // #[modifier(custom = "custom_fn")]
                            syn::Meta::NameValue(syn::MetaNameValue {
                                ref path, ref lit, ..
                            }) => {
                                let ident = path.get_ident().unwrap();
                                match ident.to_string().as_ref() {
                                    "custom" => {
                                        match lit_to_string(lit) {
                                            Some(s) => modifiers.push(ModType::Custom{
                                                function: s,

                                            }),
                                            None => error(lit.span(), "Invalid argument for `custom` modifier, only strings are allowed"),
                                        };
                                    }
                                    v => abort!(
                                        path.span(),
                                        "Unexpected name value modifier: {:?}",
                                        v
                                    ),
                                };
                            } // Validators with several args
                            syn::Meta::List(syn::MetaList {
                                ref path,
                                ref nested,
                                ..
                            }) => {
                                let meta_items = nested.iter().cloned().collect::<Vec<_>>();
                                let ident = path.get_ident().unwrap();
                                match ident.to_string().as_ref() {
                                    /* "length" => {
                                        assert_has_len(rust_ident.clone(), field_type, &field.ty);
                                        modifiers.push(extract_length_validation(
                                            rust_ident.clone(),
                                            attr,
                                            &meta_items,
                                        ));
                                    }
                                    "range" => {
                                        assert_has_range(rust_ident.clone(), field_type, &field.ty);
                                        modifiers.push(extract_range_validation(
                                            rust_ident.clone(),
                                            attr,
                                            &meta_items,
                                        ));
                                    } */
                                    "custom" => {
                                        modifiers.push(extract_custom_validation(
                                            rust_ident.clone(),
                                            attr,
                                            &meta_items,
                                        ));
                                    }
                                    v => abort!(path.span(), "Unexpected list modifier: {:?}", v),
                                }
                            }
                        },
                        _ => unreachable!("Found a non Meta while looking for modifiers"),
                    };
                }
            }
            Ok(syn::Meta::Path(_)) => {}
            Ok(syn::Meta::NameValue(_)) => abort!(attr.span(), "Unexpected name=value argument"),
            Err(e) => {
                abort!(
                    attr.span(),
                    "Unable to parse this attribute for the field `{}` with the error: {:?}",
                    field_ident,
                    e
                );
            }
        }

        if has_modifiers && modifiers.is_empty() {
            error(attr.span(), "Needs at least one modifier");
        }
    }

    (field_ident, modifiers)
}

fn assert_string_type(name: &str, type_name: &str, field_type: &syn::Type) {
    if !type_name.contains("String") {
        abort!(
            field_type.span(),
            "`{}` modifier can only be used on `Option<String>` or `String`",
            name
        );
    }
}

fn lit_to_string(lit: &syn::Lit) -> Option<String> {
    match *lit {
        syn::Lit::Str(ref s) => Some(s.value()),
        _ => None,
    }
}

/// Find the types (as string) for each field of the struct
/// Needed for the `must_match` filter
fn map_field_types(fields: &[syn::Field]) -> HashMap<String, String> {
    let mut types = HashMap::new();

    for field in fields {
        let field_ident = field.ident.clone().unwrap().to_string();
        let field_type = match field.ty {
            syn::Type::Path(syn::TypePath { ref path, .. }) => {
                let mut tokens = proc_macro2::TokenStream::new();
                path.to_tokens(&mut tokens);
                tokens.to_string().replace(' ', "")
            }
            syn::Type::Reference(syn::TypeReference {
                ref elem,
                ref lifetime,
                ..
            }) => {
                let mut tokens = proc_macro2::TokenStream::new();
                elem.to_tokens(&mut tokens);
                let mut name = tokens.to_string().replace(' ', "");
                if lifetime.is_some() {
                    name.insert(0, '&')
                }
                name
            }
            syn::Type::Group(syn::TypeGroup { ref elem, .. }) => {
                let mut tokens = proc_macro2::TokenStream::new();
                elem.to_tokens(&mut tokens);
                tokens.to_string().replace(' ', "")
            }
            _ => {
                let mut field_type = proc_macro2::TokenStream::new();
                field.ty.to_tokens(&mut field_type);
                field_type.to_string().replace(' ', "")
            }
        };
        types.insert(field_ident, field_type);
    }

    types
}

fn extract_custom_validation(
    field: String,
    attr: &syn::Attribute,
    meta_items: &[syn::NestedMeta],
) -> ModType {
    let mut function = None;

    let error = |span: Span, msg: &str| -> ! {
        abort!(
            span,
            "Invalid attribute #[modifier] on field `{}`: {}",
            field,
            msg
        );
    };

    for meta_item in meta_items {
        match *meta_item {
            syn::NestedMeta::Meta(ref item) => match *item {
                syn::Meta::NameValue(syn::MetaNameValue {
                    ref path, ref lit, ..
                }) => {
                    let ident = path.get_ident().unwrap();
                    match ident.to_string().as_ref() {
                        "function" => {
                            function = match lit_to_string(lit) {
                                Some(s) => Some(s),
                                None => error(lit.span(), "Invalid argument type for `function` of `custom` validator: expected a string")
                            };
                        }
                        v => error(path.span(), &format!(
                            "Invalid argument `{}` for `custom` modifier. A function identifier should be used.",
                            v
                        )),
                    }
                }
                _ => abort!(
                    item.span(),
                    "Unexpected item {:?} while parsing `custom` modifier",
                    item
                ),
            },
            _ => unreachable!(),
        }
    }

    if function.is_none() {
        error(
            attr.span(),
            "The `custom` modifier requires a `function` parameter.",
        );
    }

    ModType::Custom {
        function: function.unwrap(),
    }
}
