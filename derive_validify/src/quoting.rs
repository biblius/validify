use crate::fields::FieldInformation;
use proc_macro2::{Ident, Span};
use quote::quote;
use traits::ModType;

/// Creates a token stream applying the modifiers based on the field annotations.
pub(super) fn quote_field_modifiers(
    mut fields: Vec<FieldInformation>,
) -> (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>) {
    let mut modifications = vec![];
    let mut validations = vec![];

    fields.drain(..).for_each(|item| {
        let field_ident = item.field.ident.clone().unwrap();
        let field_quoter = FieldQuoter::new(field_ident, item.name, item.field_type);

        for modifier in item.modifiers.iter() {
            let (mods, valids) = quote_modifiers(&field_quoter, modifier);
            modifications.push(mods);
            if let Some(validation) = valids {
                validations.push(validation)
            }
        }
    });

    (modifications, validations)
}

/// Returns a modification and a validation (if it's nested) statement for the field.
fn quote_modifiers(
    fq: &FieldQuoter,
    mod_type: &ModType,
) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
    let (ty, span) = (fq._type.clone(), fq.ident.span());
    let modifier_param = fq.quote_modifier_param();
    let is_option = fq.check_option();
    let is_vec = fq.check_vec();

    let (mods, valids) = match mod_type {
        ModType::Trim => (quote_trim_modifier(modifier_param, is_option, is_vec), None),
        ModType::Uppercase => (
            quote_uppercase_modifier(modifier_param, is_option, is_vec),
            None,
        ),
        ModType::Lowercase => (
            quote_lowercase_modifier(modifier_param, is_option, is_vec),
            None,
        ),
        ModType::Capitalize => (
            quote_capitalize_modifier(modifier_param, is_option, is_vec),
            None,
        ),
        ModType::Custom { function } => (
            quote_custom_modifier(modifier_param, function, is_option),
            None,
        ),
        ModType::Nested => {
            let (modify, validate) =
                quote_nested_modifier(modifier_param, fq.name.clone(), ty, span, is_vec);
            (modify, Some(validate))
        }
    };

    (
        fq.wrap_if_option(mods),
        valids.map(|tokens| fq.wrap_if_option(tokens)),
    )
}

fn quote_nested_modifier(
    param: proc_macro2::TokenStream,
    name: String,
    ty: String,
    span: Span,
    is_vec: bool,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let ident = if is_vec {
        Ident::new(&strip_vec_prefix(&ty), span)
    } else {
        Ident::new(&ty, span)
    };

    let par = param.to_string();
    let field = par.split('.').last().unwrap();
    let field_ident: proc_macro2::TokenStream = format!("this.{}", field).parse().unwrap();

    let modifications = if is_vec {
        quote!(
            for el in #param.iter_mut() {
                el.modify();
            }
        )
    } else {
        quote!(#param.modify();)
    };

    let validations = if is_vec {
        quote!(
            for el in #field_ident.iter_mut() {
                match <#ident as ::validify::Validify>::validate(el.clone().into()) {
                    Ok(_) => {},
                    Err(errs) => {
                        errors = validator::ValidationErrors::merge(
                            errors,
                            #name,
                            Err(errs)
                        );
                    }
                }
            }
        )
    } else {
        quote!(
            match <#ident as ::validify::Validify>::validate(#field_ident.clone().into()) {
                Ok(_) => {},
                Err(errs) => {
                    errors = validator::ValidationErrors::merge(
                        errors,
                        #name,
                        Err(errs)
                    );
                }
            }
        )
    };
    (modifications, validations)
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
    is_vec: bool,
) -> proc_macro2::TokenStream {
    if is_vec {
        return quote_vec_modifier(param, is_option, ModType::Trim);
    }
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
    is_vec: bool,
) -> proc_macro2::TokenStream {
    if is_vec {
        return quote_vec_modifier(param, is_option, ModType::Uppercase);
    }
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

fn quote_vec_modifier(
    param: proc_macro2::TokenStream,
    is_option: bool,
    mod_type: ModType,
) -> proc_macro2::TokenStream {
    let modifier = match mod_type {
        ModType::Trim => quote!(trim().to_string()),
        ModType::Uppercase => quote!(to_uppercase()),
        ModType::Lowercase => quote!(to_lowercase()),
        ModType::Capitalize => quote!(),
        _ => unreachable!("Use of modifier that can be applied directly to vec forbidden"),
    };
    if is_option {
        if mod_type == ModType::Capitalize {
            quote!(
                for el in #param.iter_mut() {
                    *el = ::std::format!("{}{}", &el[0..1].to_uppercase(), &el[1..])
                }
            )
        } else {
            quote!(
                for el in #param.iter_mut() {
                    *el = el.#modifier
                }
            )
        }
    } else if mod_type == ModType::Capitalize {
        quote!(
            for el in #param.iter_mut() {
                *el = ::std::format!("{}{}", &el[0..1].to_uppercase(), &el[1..])
            }
        )
    } else {
        quote!(
            for el in #param.iter_mut() {
                *el = el.#modifier
            }
        )
    }
}

pub(super) fn quote_lowercase_modifier(
    param: proc_macro2::TokenStream,
    is_option: bool,
    is_vec: bool,
) -> proc_macro2::TokenStream {
    if is_vec {
        return quote_vec_modifier(param, is_option, ModType::Lowercase);
    }
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
    is_vec: bool,
) -> proc_macro2::TokenStream {
    if is_vec {
        return quote_vec_modifier(param, is_option, ModType::Capitalize);
    }
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
    name: String,
    /// The field type
    _type: String,
}

impl FieldQuoter {
    pub fn new(ident: syn::Ident, name: String, _type: String) -> FieldQuoter {
        FieldQuoter { ident, name, _type }
    }

    /// Check if this field's type is an Option
    pub fn check_option(&self) -> bool {
        self._type.starts_with("Option")
    }

    /// Check if this field's type is an Option
    pub fn check_vec(&self) -> bool {
        self._type.starts_with("Vec") || self._type.starts_with("Option<Vec")
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

fn strip_vec_prefix(s: &str) -> String {
    let s = s.replace("Vec<", "");
    s.replace('>', "")
}
