use crate::fields::FieldInfo;
use quote::quote;

#[derive(Debug, PartialEq, Eq)]
pub enum Modifier {
    Trim,
    Uppercase,
    Lowercase,
    Capitalize,
    Custom { function: syn::Path },
    Nested,
}

impl Modifier {
    /// Returns direct modification tokens as the first element and any nested validify tokens as the second element.
    /// Necessary because we need both in case a nested validify occurs. In that case, the first element will have the
    /// necessary modification tokens for nested elements in the `Modify` impl while the second will have the tokens
    /// for the `Validify` impl.
    pub fn to_validify_tokens(
        &self,
        field_info: &FieldInfo,
    ) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
        let param = field_info.quote_modifier_param();
        match self {
            Modifier::Trim => {
                let tokens = if field_info.is_option() {
                    quote!(
                        *#param = #param.trim().to_string();
                    )
                } else {
                    quote!(
                        #param = #param.trim().to_string();
                    )
                };
                (
                    field_info.wrap_modifier_if_option(
                        field_info.wrap_modifier_if_collection(param, tokens, self),
                        false,
                    ),
                    None,
                )
            }
            Modifier::Uppercase => {
                let tokens = if field_info.is_option() {
                    quote!(
                        *#param = #param.to_uppercase();
                    )
                } else {
                    quote!(
                        #param = #param.to_uppercase();
                    )
                };
                (
                    field_info.wrap_modifier_if_option(
                        field_info.wrap_modifier_if_collection(param, tokens, self),
                        false,
                    ),
                    None,
                )
            }
            Modifier::Lowercase => {
                let tokens = if field_info.is_option() {
                    quote!(
                        *#param = #param.to_lowercase();
                    )
                } else {
                    quote!(
                        #param = #param.to_lowercase();
                    )
                };
                (
                    field_info.wrap_modifier_if_option(
                        field_info.wrap_modifier_if_collection(param, tokens, self),
                        false,
                    ),
                    None,
                )
            }
            Modifier::Capitalize => {
                let tokens = if field_info.is_option() {
                    quote!(
                      *#param = ::std::format!("{}{}", &#param[0..1].to_uppercase(), &#param[1..]);
                    )
                } else {
                    quote!(
                      #param = ::std::format!("{}{}", &#param[0..1].to_uppercase(), &#param[1..]);
                    )
                };
                (
                    field_info.wrap_modifier_if_option(
                        field_info.wrap_modifier_if_collection(param, tokens, self),
                        false,
                    ),
                    None,
                )
            }
            Modifier::Custom { function } => {
                let tokens = if field_info.is_option() {
                    quote!(
                        #function(#param);
                    )
                } else {
                    quote!(
                        #function(&mut #param);
                    )
                };
                (field_info.wrap_modifier_if_option(tokens, false), None)
            }
            Modifier::Nested => {
                let par = param.to_string();
                let field = par.split('.').last().unwrap();

                let modifications = if field_info.is_list() {
                    quote!(
                        for el in #param.iter_mut() {
                            el.modify();
                        }
                    )
                } else {
                    quote!(#param.modify();)
                };

                let field_ident: proc_macro2::TokenStream =
                    format!("this.{field}").parse().unwrap();

                let param = if field_info.is_option() {
                    let field: proc_macro2::TokenStream = field.parse().unwrap();
                    field
                } else {
                    field_ident
                };

                let nested_validifies = if field_info.is_list() {
                    quote!(
                        for (i, el) in #param.iter_mut().enumerate() {
                            if let Err(mut errs) = el.validify_self() {
                                errs.errors_mut().iter_mut().for_each(|err|err.set_location_idx(i, #field));
                                errors.merge(errs);
                            }
                        }
                    )
                } else {
                    quote!(
                        if let Err(mut err) = #param.validify_self() {
                            err.errors_mut().iter_mut().for_each(|e| e.set_location(#field));
                            errors.merge(err);
                        }
                    )
                };
                (
                    field_info.wrap_modifier_if_option(modifications, false),
                    Some(field_info.wrap_modifier_if_option(nested_validifies, true)),
                )
            }
        }
    }
}
