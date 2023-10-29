use crate::{
    types::{Modifier, Validator},
    validate::r#impl::collect_validations,
    validify::r#impl::collect_modifiers,
};
use proc_macro_error::abort;
use quote::ToTokens;
use std::collections::HashMap;
use syn::{parenthesized, spanned::Spanned, Expr, Token};

type FieldAttrCollection = Result<(Vec<Validator>, Vec<Modifier>, Option<String>), syn::Error>;

/// Holds the combined validations and modifiers for one field
#[derive(Debug)]
pub struct FieldInfo {
    pub field: syn::Field,
    pub field_type: String,
    pub name: String,
    pub original_name: Option<String>,
    pub validations: Vec<Validator>,
    pub modifiers: Vec<Modifier>,
}

impl FieldInfo {
    pub fn new(
        field: syn::Field,
        field_type: String,
        name: String,
        original_name: Option<String>,
        validations: Vec<Validator>,
        modifiers: Vec<Modifier>,
    ) -> Self {
        FieldInfo {
            field,
            field_type,
            name,
            original_name,
            validations,
            modifiers,
        }
    }
}

/// Used by both the `Validate` and `Validify` implementations. Validate ignores the modifiers.
pub fn collect_field_info(
    input: &syn::DeriveInput,
    allow_refs: bool,
) -> Result<Vec<FieldInfo>, syn::Error> {
    let mut fields = collect_fields(input);

    let field_types = map_field_types(&fields, allow_refs);

    let mut field_info = vec![];

    for field in fields.drain(..) {
        let field_ident = field
            .ident
            .as_ref()
            .expect("Found unnamed field")
            .to_string();

        let (validations, modifiers, original_name) =
            collect_field_attributes(&field, &field_types)?;

        field_info.push(FieldInfo::new(
            field,
            field_types.get(&field_ident).unwrap().clone(),
            field_ident,
            original_name,
            validations,
            modifiers,
        ));
    }

    Ok(field_info)
}

/// Find the types (as string) for each field of the struct. The `allow_refs`, if false, will error if
/// the field is a reference. This is needed for modifiers as we do not allow references when deriving
/// `Validifty`. References in `Validate` are OK.
pub fn map_field_types(fields: &[syn::Field], allow_refs: bool) -> HashMap<String, String> {
    let mut types = HashMap::new();

    for field in fields {
        let field_ident = field
            .ident
            .clone()
            .expect("Found unnamed field")
            .to_string();

        let field_type = match field.ty {
            syn::Type::Path(syn::TypePath { ref path, .. }) => {
                let mut tokens = proc_macro2::TokenStream::new();
                path.to_tokens(&mut tokens);
                tokens.to_string().replace(' ', "")
            }
            syn::Type::Reference(syn::TypeReference {
                ref lifetime,
                ref elem,
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
            ref ty => {
                let mut field_type = proc_macro2::TokenStream::new();
                ty.to_tokens(&mut field_type);
                field_type.to_string().replace(' ', "")
            }
        };
        if field_type.contains('&') && !allow_refs {
            abort!(
                field.span(),
                "Validify must be implemented for structs with owned data, if you just need validation and not modification, use Validate instead"
            )
        }
        types.insert(field_ident, field_type);
    }

    types
}

pub fn collect_fields(input: &syn::DeriveInput) -> Vec<syn::Field> {
    match input.data {
        syn::Data::Struct(syn::DataStruct { ref fields, .. }) => {
            if fields.iter().any(|field| field.ident.is_none()) {
                abort!(
                    fields.span(),
                    "#[derive(Validate/Validify)] can only be used on structs with named fields"
                );
            }

            fields.iter().cloned().collect::<Vec<syn::Field>>()
        }
        _ => abort!(
            input.span(),
            "#[derive(Validate/Validify)] can only be used on structs with named fields"
        ),
    }
}

/// Find everything we need to know about a field: its real name if it's changed from the serialization
/// and the list of validators to run on it
pub fn collect_field_attributes(
    field: &syn::Field,
    field_types: &HashMap<String, String>,
) -> FieldAttrCollection {
    let field_ident = field.ident.clone().unwrap().to_string();
    let field_type = field_types.get(&field_ident).unwrap();

    let mut validators = vec![];
    let mut modifiers = vec![];

    collect_validations(&mut validators, field, field_type);
    collect_modifiers(&mut modifiers, field);

    // The original name refers to the field name set with serde rename.
    let original_name = find_original_field_name(field);

    Ok((validators, modifiers, original_name))
}

fn find_original_field_name(field: &syn::Field) -> Option<String> {
    let mut original_name = None;
    for attr in field.attrs.iter() {
        if !attr.path().is_ident("serde") {
            continue;
        }

        // serde field attributes are always lists
        let Ok(serde_meta) = attr.meta.require_list() else {
            continue;
        };

        let _ = serde_meta.parse_nested_meta(|meta| {
            if !meta.path.is_ident("rename") {
                return Ok(());
            }

            // Covers `rename = "something"`
            if meta.input.peek(Token!(=)) {
                let content = meta.value()?;
                original_name = Some(content.parse::<syn::LitStr>()?.value());
                return Ok(());
            }

            // Covers `rename(deserialize = "something")`
            if meta.input.peek(syn::token::Paren) {
                let content;
                parenthesized!(content in meta.input);
                let name_value = content.parse::<syn::MetaNameValue>()?;

                // We're only interested in the deserialize property as that is the
                // one related to the client payload
                if name_value.path.is_ident("deserialize") {
                    let Expr::Lit(expr_lit) = name_value.value else {
                        return Ok(());
                    };
                    if let syn::Lit::Str(str_lit) = expr_lit.lit {
                        original_name = Some(str_lit.value())
                    }
                }
                return Ok(());
            }

            Ok(())
        });
    }
    original_name
}
