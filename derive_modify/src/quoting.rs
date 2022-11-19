use crate::fields::FieldInformation;
use modify::ModType;
use quote::quote;

/// Creates a token stream applying the modifiers based on the field annotations.
pub(super) fn quote_field_modifiers(
    mut fields: Vec<FieldInformation>,
) -> Vec<proc_macro2::TokenStream> {
    let mut modifications = vec![];

    fields.drain(..).for_each(|item| {
        let field_ident = item.field.ident.clone().unwrap();
        let field_quoter = FieldQuoter::new(field_ident, item.field_type);

        for modifier in &item.modifiers {
            modifications.push(quote_modifiers(&field_quoter, modifier))
        }
    });

    modifications
}

/// Returns a modification statement for the field.
fn quote_modifiers(fq: &FieldQuoter, mod_type: &ModType) -> proc_macro2::TokenStream {
    let modifier_param = fq.quote_modifier_param();
    let is_option = fq.check_option();

    let quoted = match mod_type {
        ModType::Trim => quote_trim_modifier(modifier_param, is_option),
        ModType::Uppercase => quote_uppercase_modifier(modifier_param, is_option),
        ModType::Lowercase => quote_lowercase_modifier(modifier_param, is_option),
        ModType::Capitalize => quote_capitalize_modifier(modifier_param, is_option),
        ModType::Custom { function } => quote_custom_modifier(modifier_param, function, is_option),
        ModType::Nested => quote_nested_modifier(modifier_param),
    };

    fq.wrap_if_option(quoted)
}

fn quote_nested_modifier(param: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    quote!(#param.modify())
}

fn quote_custom_modifier(
    param: proc_macro2::TokenStream,
    function: &str,
    is_option: bool,
) -> proc_macro2::TokenStream {
    let fn_ident: syn::Path = syn::parse_str(function).unwrap();
    if is_option {
        quote!(
            #fn_ident(#param);
        )
    } else {
        quote!(
            #fn_ident(&mut #param);
        )
    }
}

pub(super) fn quote_trim_modifier(
    param: proc_macro2::TokenStream,
    is_option: bool,
) -> proc_macro2::TokenStream {
    if is_option {
        quote!(
            *#param = #param.trim().to_string();
        )
    } else {
        quote!(
            #param = #param.trim().to_string();
        )
    }
}

pub(super) fn quote_uppercase_modifier(
    param: proc_macro2::TokenStream,
    is_option: bool,
) -> proc_macro2::TokenStream {
    if is_option {
        quote!(
            *#param = #param.to_uppercase();
        )
    } else {
        quote!(
            #param = #param.to_uppercase();
        )
    }
}

pub(super) fn quote_lowercase_modifier(
    param: proc_macro2::TokenStream,
    is_option: bool,
) -> proc_macro2::TokenStream {
    if is_option {
        quote!(
            *#param = #param.to_lowercase();
        )
    } else {
        quote!(
            #param = #param.to_lowercase();
        )
    }
}

pub(super) fn quote_capitalize_modifier(
    param: proc_macro2::TokenStream,
    is_option: bool,
) -> proc_macro2::TokenStream {
    if is_option {
        quote!(
          *#param = ::std::format!("{}{}", &#param[0..1].to_uppercase(), &#param[1..]);
        )
    } else {
        quote!(
          #param = ::std::format!("{}{}", &#param[0..1].to_uppercase(), &#param[1..]);
        )
    }
}

/// Contains the field ident and its type
#[derive(Debug)]
pub(super) struct FieldQuoter {
    ident: syn::Ident,
    /// The field type
    _type: String,
}

impl FieldQuoter {
    pub fn new(ident: syn::Ident, _type: String) -> FieldQuoter {
        FieldQuoter { ident, _type }
    }

    /// Check if this field's type is an Option
    pub fn check_option(&self) -> bool {
        self._type.starts_with("Option")
    }

    /// Returns `self.#ident`, unless the field is an option in which case it just
    /// returns an `#ident` as we always do a `if let` check on Option fields
    pub fn quote_modifier_param(&self) -> proc_macro2::TokenStream {
        let ident = &self.ident;

        if self._type.starts_with('&') {
            panic!("Fields containing modifiers must contain owned data")
        }

        if self._type.starts_with("Option<") {
            quote!(#ident)
        } else {
            quote!(self.#ident)
        }
    }

    pub fn get_optional_modifier_param(&self) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        if self._type.starts_with("Option<&") || self._type.starts_with("Option<Option<&") {
            panic!("Fields containing modifiers must contain owned data")
        } else {
            quote!(#ident)
        }
    }

    /// If `self._type` is an option, wrap the given tokens in an `if let Some()` statement
    pub fn wrap_if_option(&self, tokens: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
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
}
