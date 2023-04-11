use crate::validate::asserts::{is_list, is_map, COW_TYPE, NUMBER_TYPES};
use proc_macro2::{self};
use proc_macro_error::abort;
use quote::quote;

/// Pass around all the information needed for creating a validation
#[derive(Debug)]
pub struct FieldQuoter {
    pub ident: syn::Ident,
    pub name: String,
    pub _type: String,
}

impl FieldQuoter {
    pub fn new(ident: syn::Ident, name: String, _type: String) -> FieldQuoter {
        FieldQuoter { ident, name, _type }
    }

    /// Don't put a & in front a pointer since we are going to pass
    /// a reference to the validator
    /// Also just use the ident without `if` as it's optional and will go through
    /// an if let first
    pub fn quote_validator_param(&self) -> proc_macro2::TokenStream {
        let ident = &self.ident;

        if self._type.starts_with("Option<") {
            quote!(#ident)
        } else if COW_TYPE.is_match(self._type.as_ref()) {
            quote!(self.#ident.as_ref())
        } else if self._type.starts_with('&') || NUMBER_TYPES.contains(&self._type.as_ref()) {
            quote!(self.#ident)
        } else {
            quote!(&self.#ident)
        }
    }

    /// Returns `self.#ident`, unless the field is an option in which case it just
    /// returns an `#ident` as we always do a `if let` check on Option fields
    pub fn quote_modifier_param(&self) -> proc_macro2::TokenStream {
        let ident = &self.ident;

        if self._type.starts_with('&') {
            abort!(
                ident.span(),
                "Fields containing modifiers must contain owned data"
            )
        }

        if self._type.starts_with("Option<") {
            quote!(#ident)
        } else {
            quote!(self.#ident)
        }
    }

    pub fn quote_validator_field(&self) -> proc_macro2::TokenStream {
        let ident = &self.ident;

        if self._type.starts_with("Option<") || is_list(&self._type) || is_map(&self._type) {
            quote!(#ident)
        } else if COW_TYPE.is_match(self._type.as_ref()) {
            quote!(self.#ident.as_ref())
        } else {
            quote!(self.#ident)
        }
    }

    /// Wrap the quoted output of a validation with a if let Some if
    /// the field type is an option
    pub fn wrap_validator_if_option(
        &self,
        tokens: proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        let field_ident = &self.ident;
        let optional_pattern_matched = self.get_optional_validator_param();
        if self._type.starts_with("Option<Option<") {
            return quote!(
                if let Some(Some(#optional_pattern_matched)) = self.#field_ident {
                    #tokens
                }
            );
        } else if self._type.starts_with("Option<") {
            return quote!(
                if let Some(#optional_pattern_matched) = self.#field_ident {
                    #tokens
                }
            );
        }
        tokens
    }

    pub fn get_optional_validator_param(&self) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        if self._type.starts_with("Option<&")
            || self._type.starts_with("Option<Option<&")
            || NUMBER_TYPES.contains(&self._type.as_ref())
        {
            quote!(#ident)
        } else {
            quote!(ref #ident)
        }
    }

    /// Wrap the quoted output of a modification with a if let Some if
    /// the field type is an option
    pub fn wrap_modifier_if_option(
        &self,
        tokens: proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        let field_ident = &self.ident;
        let optional_pattern_matched = self.get_optional_modifier_param();
        if self._type.starts_with("Option<Option<") {
            return quote!(
                if let Some(Some(#optional_pattern_matched)) = self.#field_ident.as_mut() {
                    #tokens
                }
            );
        } else if self._type.starts_with("Option<") {
            return quote!(
                if let Some(#optional_pattern_matched) = self.#field_ident.as_mut() {
                    #tokens
                }
            );
        }

        tokens
    }

    pub fn get_optional_modifier_param(&self) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        if self._type.starts_with("Option<&") || self._type.starts_with("Option<Option<&") {
            abort!(
                ident.span(),
                "Fields containing modifiers must contain owned data"
            )
        } else {
            quote!(#ident)
        }
    }

    /// Wrap the quoted output of a validation with a for loop if
    /// the field type is a vector
    pub fn wrap_if_collection(&self, tokens: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        let field_ident = &self.ident;

        // When we're using an option, we'll have the field unwrapped, so we should not access it
        // through `self`.
        let prefix = (!self._type.starts_with("Option<")).then(|| quote! { self. });

        // When iterating over a list, the iterator has Item=T, while a map yields Item=(K, V), and
        // we're only interested in V.
        let args = if is_list(&self._type) {
            quote! { #field_ident }
        } else if is_map(&self._type) {
            quote! { (_, #field_ident) }
        } else {
            return tokens;
        };

        quote! {
            for #args in #prefix #field_ident.iter() {
                #tokens
            }
        }
    }

    /// Check if this field's type is an Option
    pub fn check_option(&self) -> bool {
        self._type.starts_with("Option")
    }

    /// Check if this field's type is an Option
    pub fn check_vec(&self) -> bool {
        self._type.starts_with("Vec") || self._type.starts_with("Option<Vec")
    }
}
