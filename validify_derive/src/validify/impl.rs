use super::quoting::quote_field_modifiers;
use crate::{fields::collect_field_info, types::Modifier, validate::r#impl::impl_validate};
use proc_macro_error::abort;
use quote::quote;
use syn::parenthesized;

const TRIM_MODIFIER: &str = "trim";
const CUSTOM_MODIFIER: &str = "custom";
const UPPERCASE_MODIFIER: &str = "uppercase";
const LOWERCASE_MODIFIER: &str = "lowercase";
const CAPITALIZE_MODIFIER: &str = "capitalize";
const VALIDIFY: &str = "validify";
const MODIFY: &str = "modify";

/// Impl entry point
pub fn impl_validify(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let ident = &input.ident;

    let field_info = collect_field_info(input, false).unwrap();

    let (modifiers, nested_validifies) = quote_field_modifiers(field_info);

    let validate_impl = impl_validate(input);

    let (payload_impl, payload_ident) = super::payload::generate(input);

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

            if let Err(errs) = this.validify_self() {
                errors.merge(errs);
            }

            if !errors.is_empty() {
                Err(errors)
            } else {
                Ok(this)
            }
        }
    })
}

pub fn collect_modifiers(modifiers: &mut Vec<Modifier>, field: &syn::Field) {
    for attr in &field.attrs {
        // Nest validified fields
        if attr.path().is_ident(VALIDIFY) {
            modifiers.push(Modifier::Nested);
            continue;
        }

        if !attr.path().is_ident(MODIFY) {
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
