use super::quoting::quote_field_modifiers;
use crate::{
    fields::{collect_field_info, collect_fields, map_field_types},
    types::Modifier,
    validate::r#impl::impl_validate,
};
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::quote;
use syn::parenthesized;

const TRIM_MODIFIER: &str = "trim";
const CUSTOM_MODIFIER: &str = "custom";
const UPPERCASE_MODIFIER: &str = "uppercase";
const LOWERCASE_MODIFIER: &str = "lowercase";
const CAPITALIZE_MODIFIER: &str = "capitalize";
const VALIDIFY: &str = "validify";

/// Impl entry point
pub fn impl_validify(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let ident = &input.ident;
    let field_info = collect_field_info(input, false).unwrap();

    let (modifiers, nested_validifies) = quote_field_modifiers(field_info);

    let validate_impl = impl_validate(input);

    let (payload_impl, payload_ident) = generate_payload_type(input);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote!(

    #payload_impl

    #validate_impl

    impl #impl_generics ::validify::Modify for #ident #ty_generics #where_clause {
        fn modify(&mut self) {
            #(#modifiers)*
        }
    }

    impl #impl_generics ::validify::Validify for #ident #ty_generics #where_clause {

        type Payload = #payload_ident;

        /// Apply the provided modifiers to self and run validations
        fn validify(payload: Self::Payload) -> Result<Self, ::validify::ValidationErrors> {
            <Self::Payload as ::validify::Validate>::validate(&payload)?;
            let mut this = Self::from(payload);
            let mut errors = ::validify::ValidationErrors::new();
            #(#nested_validifies)*
            <Self as ::validify::Modify>::modify(&mut this);
            if let Err(errs) = <Self as ::validify::Validate>::validate(&this) {
                errors.merge(errs);
            }
            if !errors.is_empty() {
                errors.sort();
                return Err(errors);
            }
            Ok(this)
        }
    })
}

fn generate_payload_type(
    input: &syn::DeriveInput,
) -> (proc_macro2::TokenStream, proc_macro2::Ident) {
    let ident = &input.ident;
    let visibility = &input.vis;
    let payload_ident = syn::Ident::new(
        &format!("{}Payload", &input.ident.to_string()),
        Span::call_site(),
    );

    let fields = collect_fields(input);
    let types = map_field_types(&fields, false);

    let payload_fields = fields
        .iter()
        .map(|f| {
            let ident = f.ident.as_ref().unwrap();
            let typ = types.get(&ident.to_string()).unwrap();
            let ty = &f.ty;
            if typ.starts_with("Option") {
                quote!(
                    #ident: #ty,
                )
            } else {
                quote!(
                    #[validate(required)]
                    #ident: Option<#ty>,
                )
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let into_fields = fields
        .iter()
        .map(|f| {
            let ident = f.ident.as_ref().unwrap();
            let typ = types.get(&ident.to_string()).unwrap();
            if typ.starts_with("Option") {
                quote!(
                    #ident: original.#ident,
                )
            } else {
                quote!(#ident: Some(original.#ident),)
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let from_fields = fields
        .iter()
        .map(|f| {
            let ident = f.ident.as_ref().unwrap();
            let typ = types.get(&ident.to_string()).unwrap();
            if typ.starts_with("Option") {
                quote!(
                    #ident: payload.#ident,
                )
            } else {
                quote!(#ident: payload.#ident.unwrap(),)
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let quoted = quote!(
        #[derive(Debug, Clone, ::validify::Validate, serde::Deserialize)]
        #visibility struct #payload_ident #ty_generics #where_clause {
            #(#payload_fields)*
        }

         impl #impl_generics From<#ident> for #payload_ident {
            fn from(original: #ident) -> Self {
                Self {
                    #(#into_fields)*
                }
            }
        }

        impl #impl_generics From<#payload_ident> for #ident {
            fn from(payload: #payload_ident) -> Self {
                Self {
                    #(#from_fields)*
                }
            }
        }
    );

    (quoted, payload_ident)
}

pub fn collect_modifiers(modifiers: &mut Vec<Modifier>, field: &syn::Field) {
    for attr in &field.attrs {
        // Nest validified fields
        if attr.path().is_ident(VALIDIFY) {
            modifiers.push(Modifier::Nested);
            continue;
        }

        if !attr.path().is_ident("modify") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident(CUSTOM_MODIFIER) {
                let content;
                parenthesized!(content in meta.input);
                let path: syn::Path = content.parse()?;
                modifiers.push(Modifier::Custom { function: path });
                return Ok(());
            }

            if meta.path.is_ident(TRIM_MODIFIER) {
                modifiers.push(Modifier::Trim);
                return Ok(());
            }

            if meta.path.is_ident(LOWERCASE_MODIFIER) {
                modifiers.push(Modifier::Lowercase);
                return Ok(());
            }

            if meta.path.is_ident(UPPERCASE_MODIFIER) {
                modifiers.push(Modifier::Uppercase);
                return Ok(());
            }

            if meta.path.is_ident(CAPITALIZE_MODIFIER) {
                modifiers.push(Modifier::Capitalize);
                return Ok(());
            }

            Err(meta.error("Unrecognized modify parameter"))
        })
        .unwrap_or_else(|e| abort!(e.span(), e));
    }
}
