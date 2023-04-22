use crate::asserts::{is_list, is_map, COW_TYPE, NUMBER_TYPES};
use proc_macro2::{self};
use proc_macro_error::abort;
use quote::quote;

/// Pass around all the information needed for creating a validation
#[derive(Debug)]
pub struct FieldQuoter {
    pub ident: syn::Ident,
    pub name: String,
    pub original_name: Option<String>,
    pub _type: String,
}

impl FieldQuoter {
    pub fn new(
        ident: syn::Ident,
        name: String,
        original_name: Option<String>,
        _type: String,
    ) -> FieldQuoter {
        FieldQuoter {
            ident,
            name,
            original_name,
            _type,
        }
    }

    pub fn name(&self) -> &str {
        self.original_name.as_deref().unwrap_or(&self.name)
    }

    /// Quotes the field as necessary for passing the resulting tokens into a validation
    /// function.
    ///
    /// If the field is an `Option`, we simply quote the field as we always
    /// wrap optional fields in an `if let Some`.
    ///
    /// If the field is a reference the returned tokens are `self.field`.
    ///
    /// If the field is owned, the tokens are `&self.field`.
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

    /// If the field is an `Option` the passed tokens get wrapped in an `if let Some`
    /// statement, otherwise just return the tokens.
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

    /// Returns `ident` or `ref ident` to account for the
    /// `if let Some(ident) = self.field`.
    fn get_optional_validator_param(&self) -> proc_macro2::TokenStream {
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

    fn get_optional_modifier_param(&self) -> proc_macro2::TokenStream {
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
    /// the field type is a collection (used solely for nested validation)
    pub fn wrap_if_collection(
        &self,
        validator_field: proc_macro2::TokenStream,
        tokens: proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        let field_name = &self.name;

        // When we're using an option, we'll have the field unwrapped, so we should not access it
        // through `self`.
        let prefix = (!self._type.starts_with("Option<")).then(|| quote! { self. });

        // When iterating over a list, the iterator has Item=T, while a map yields Item=(K, V), and
        // we're only interested in V.
        if is_list(&self._type) {
            quote!(
                for (i, item) in #prefix #validator_field.iter().enumerate() {
                    if let Err(mut errs) = item.validate() {
                        errs.errors_mut().iter_mut().for_each(|err| err.set_location_idx(i, #field_name));
                        errors.merge(errs);
                    }
                }
            )
        } else if is_map(&self._type) {
            quote!(
                for (key, item) in #prefix #validator_field.iter() {
                    if let Err(mut errs) = item.validate() {
                        errs.errors_mut().iter_mut().for_each(|err| err.set_location_idx(key, #field_name));
                        errors.merge(errs);
                    }
                }
            )
        } else {
            tokens
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
