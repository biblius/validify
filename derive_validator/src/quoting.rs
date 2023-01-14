use crate::asserts::{is_list, is_map, COW_TYPE, NUMBER_TYPES};
use crate::lit::{option_to_tokens, value_or_path_to_tokens};
use crate::validation::{FieldValidation, SchemaValidation};
use if_chain::if_chain;
use proc_macro2::{self, Span};
use quote::quote;
use types::Validator;

/// Pass around all the information needed for creating a validation
#[derive(Debug)]
pub struct FieldQuoter {
    ident: syn::Ident,
    /// The field name
    name: String,
    /// The field type
    _type: String,
}

impl FieldQuoter {
    pub fn new(ident: syn::Ident, name: String, _type: String) -> FieldQuoter {
        FieldQuoter { ident, name, _type }
    }

    /// Don't put a & in front a pointer since we are going to pass
    /// a reference to the validator
    /// Also just use the ident without if it's optional and will go through
    /// a if let first
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

    /// Wrap the quoted output of a validation with a if let Some if
    /// the field type is an option
    pub fn wrap_if_option(&self, tokens: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
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
}

/// Quote an actual end-user error creation automatically
fn quote_error(
    validation: &FieldValidation,
    field_name: Option<&String>,
) -> proc_macro2::TokenStream {
    let code = &validation.code;

    let field = field_name.map_or_else(|| quote!(None), |field| quote!(#field));

    println!("{:?}", validation.message);

    let add_message_quoted = if let Some(ref m) = validation.message {
        quote!(err.set_message(String::from(#m));)
    } else {
        quote!()
    };

    quote!(
        let mut err = ::validator::ValidationError::new_field(#code, #field);
        #add_message_quoted
    )
}

pub fn quote_length_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    if let Validator::Length { min, max, equal } = &validation.validator {
        let min_err_param_quoted = if let Some(v) = min {
            let v = value_or_path_to_tokens(v);
            quote!(err.add_param(::std::borrow::Cow::from("min"), &#v);)
        } else {
            quote!()
        };
        let max_err_param_quoted = if let Some(v) = max {
            let v = value_or_path_to_tokens(v);
            quote!(err.add_param(::std::borrow::Cow::from("max"), &#v);)
        } else {
            quote!()
        };
        let equal_err_param_quoted = if let Some(v) = equal {
            let v = value_or_path_to_tokens(v);
            quote!(err.add_param(::std::borrow::Cow::from("equal"), &#v);)
        } else {
            quote!()
        };

        let min_tokens = option_to_tokens(
            &min.clone()
                .as_ref()
                .map(value_or_path_to_tokens)
                .map(|x| quote!(#x as u64)),
        );
        let max_tokens = option_to_tokens(
            &max.clone()
                .as_ref()
                .map(value_or_path_to_tokens)
                .map(|x| quote!(#x as u64)),
        );
        let equal_tokens = option_to_tokens(
            &equal
                .clone()
                .as_ref()
                .map(value_or_path_to_tokens)
                .map(|x| quote!(#x as u64)),
        );

        let quoted_error = quote_error(validation, Some(field_name));
        let quoted = quote!(
            if !::validator::validate_length(
                #validator_param,
                #min_tokens,
                #max_tokens,
                #equal_tokens
            ) {
                #quoted_error
                #min_err_param_quoted
                #max_err_param_quoted
                #equal_err_param_quoted
                err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
                errors.add(err);
            }
        );

        return field_quoter.wrap_if_option(quoted);
    }

    unreachable!()
}

pub fn quote_range_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let quoted_ident = field_quoter.quote_validator_param();

    if let Validator::Range { ref min, ref max } = validation.validator {
        let min_err_param_quoted = if let Some(v) = min {
            let v = value_or_path_to_tokens(v);
            quote!(err.add_param(::std::borrow::Cow::from("min"), &#v);)
        } else {
            quote!()
        };
        let max_err_param_quoted = if let Some(v) = max {
            let v = value_or_path_to_tokens(v);
            quote!(err.add_param(::std::borrow::Cow::from("max"), &#v);)
        } else {
            quote!()
        };

        // Can't interpolate None
        let min_tokens = min
            .clone()
            .map(|x| value_or_path_to_tokens(&x))
            .map(|x| quote!(#x as f64));
        let min_tokens = option_to_tokens(&min_tokens);

        let max_tokens = max
            .clone()
            .map(|x| value_or_path_to_tokens(&x))
            .map(|x| quote!(#x as f64));
        let max_tokens = option_to_tokens(&max_tokens);

        let quoted_error = quote_error(validation, Some(field_name));
        let quoted = quote!(
            if !::validator::validate_range(
                #quoted_ident as f64,
                #min_tokens,
                #max_tokens
            ) {
                #quoted_error
                #min_err_param_quoted
                #max_err_param_quoted
                err.add_param(::std::borrow::Cow::from("value"), &#quoted_ident);
                errors.add(err);
            }
        );

        return field_quoter.wrap_if_option(quoted);
    }

    unreachable!()
}

pub fn quote_credit_card_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    let quoted_error = quote_error(validation, Some(field_name));
    let quoted = quote!(
        if !::validator::validate_credit_card(#validator_param) {
            #quoted_error
            err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
            errors.add(err);
        }
    );

    field_quoter.wrap_if_option(quoted)
}

pub fn quote_phone_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    let quoted_error = quote_error(validation, Some(field_name));
    let quoted = quote!(
        if !::validator::validate_phone(#validator_param) {
            #quoted_error
            err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
            errors.add(err);
        }
    );

    field_quoter.wrap_if_option(quoted)
}

pub fn quote_non_control_character_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    let quoted_error = quote_error(validation, Some(field_name));
    let quoted = quote!(
        if !::validator::validate_non_control_character(#validator_param) {
            #quoted_error
            err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
            errors.add(err);
        }
    );

    field_quoter.wrap_if_option(quoted)
}

pub fn quote_url_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    let quoted_error = quote_error(validation, Some(field_name));
    let quoted = quote!(
        if !::validator::validate_url(#validator_param) {
            #quoted_error
            err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
            errors.add(err);
        }
    );

    field_quoter.wrap_if_option(quoted)
}

pub fn quote_email_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    let quoted_error = quote_error(validation, Some(field_name));
    let quoted = quote!(
        if !::validator::validate_email(#validator_param) {
            #quoted_error
            err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
            errors.add(err);
        }
    );

    field_quoter.wrap_if_option(quoted)
}

pub fn quote_must_match_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let ident = &field_quoter.ident;
    let field_name = &field_quoter.name;

    if let Validator::MustMatch(ref other) = validation.validator {
        let other_ident = syn::Ident::new(other, Span::call_site());
        let quoted_error = quote_error(validation, Some(field_name));
        let quoted = quote!(
            if !::validator::validate_must_match(&self.#ident, &self.#other_ident) {
                #quoted_error
                err.add_param(::std::borrow::Cow::from("value"), &self.#ident);
                err.add_param(::std::borrow::Cow::from("other"), &self.#other_ident);
                errors.add(err);
            }
        );

        return field_quoter.wrap_if_option(quoted);
    }

    unreachable!();
}

pub fn quote_custom_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let validator_param = field_quoter.quote_validator_param();

    if let Validator::Custom {
        function, argument, ..
    } = &validation.validator
    {
        let fn_ident: syn::Path = syn::parse_str(function).unwrap();

        let access = if_chain! {
            if let Some(argument) = &**argument;
            if let Some(access) = &argument.arg_access;
            then {
                quote!(, #access)
            } else {
                quote!()
            }
        };

        let add_message_quoted = if let Some(ref m) = validation.message {
            quote!(err.set_message(String::from(#m));)
        } else {
            quote!()
        };

        let quoted = quote!(
            match #fn_ident(#validator_param #access) {
                ::std::result::Result::Ok(()) => (),
                ::std::result::Result::Err(mut err) => {
                    #add_message_quoted
                    err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
                    errors.add(err);
                },
            };
        );

        return field_quoter.wrap_if_option(quoted);
    }

    unreachable!();
}

pub fn quote_contains_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    if let Validator::Contains(ref needle) = validation.validator {
        let quoted_error = quote_error(validation, Some(field_name));
        let quoted = quote!(
            if !::validator::validate_contains(#validator_param, &#needle) {
                #quoted_error
                err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
                err.add_param(::std::borrow::Cow::from("needle"), &#needle);
                errors.add(err);
            }
        );

        return field_quoter.wrap_if_option(quoted);
    }

    unreachable!();
}

pub fn quote_regex_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    if let Validator::Regex(ref re) = validation.validator {
        let re_ident: syn::Path = syn::parse_str(re).unwrap();
        let quoted_error = quote_error(validation, Some(field_name));
        let quoted = quote!(
            if !#re_ident.is_match(#validator_param) {
                #quoted_error
                err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
                errors.add(err);
            }
        );

        return field_quoter.wrap_if_option(quoted);
    }

    unreachable!();
}

pub fn quote_nested_validation(field_quoter: &FieldQuoter) -> proc_macro2::TokenStream {
    let validator_field = field_quoter.quote_validator_field();
    let quoted = quote!(
        if let Err(errs) = #validator_field.validate() {
            errors.merge(errs);
        }
    );
    field_quoter.wrap_if_option(field_quoter.wrap_if_collection(quoted))
}

pub fn quote_validator(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
    validations: &mut Vec<proc_macro2::TokenStream>,
    nested_validations: &mut Vec<proc_macro2::TokenStream>,
) {
    match validation.validator {
        Validator::Length { .. } => {
            validations.push(quote_length_validation(field_quoter, validation))
        }
        Validator::Range { .. } => {
            validations.push(quote_range_validation(field_quoter, validation))
        }
        Validator::Email => validations.push(quote_email_validation(field_quoter, validation)),
        Validator::Url => validations.push(quote_url_validation(field_quoter, validation)),
        Validator::MustMatch(_) => {
            validations.push(quote_must_match_validation(field_quoter, validation))
        }
        Validator::Custom { .. } => {
            validations.push(quote_custom_validation(field_quoter, validation))
        }
        Validator::Contains(_) => {
            validations.push(quote_contains_validation(field_quoter, validation))
        }
        Validator::Regex(_) => validations.push(quote_regex_validation(field_quoter, validation)),
        Validator::CreditCard => {
            validations.push(quote_credit_card_validation(field_quoter, validation))
        }
        Validator::Phone => validations.push(quote_phone_validation(field_quoter, validation)),
        Validator::Nested => nested_validations.push(quote_nested_validation(field_quoter)),
        Validator::NonControlCharacter => validations.push(quote_non_control_character_validation(
            field_quoter,
            validation,
        )),
        Validator::Required | Validator::RequiredNested => {
            validations.push(quote_required_validation(field_quoter, validation))
        }
        Validator::DoesNotContain(_) => {
            validations.push(quote_does_not_contain_validation(field_quoter, validation))
        }
        Validator::In(_) => validations.push(quote_is_in_validation(field_quoter, validation)),
    }
}

pub fn quote_is_in_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    if let Validator::In(haystack) = &validation.validator {
        let field_name = &field_quoter.name;
        let validator_param = field_quoter.quote_validator_param();

        let hs = syn::Ident::new(haystack, Span::call_site());
        let quoted_error = quote_error(validation, Some(field_name));
        let quoted = quote!(
            if !#hs.contains(#validator_param.as_str()) {
                #quoted_error
                err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
                err.add_param(::std::borrow::Cow::from("haystack"), &#hs);
                errors.add(err);
            }
        );

        return field_quoter.wrap_if_option(quoted);
    }
    unreachable!()
}

pub fn quote_schema_validation(v: &SchemaValidation) -> proc_macro2::TokenStream {
    let fn_ident: syn::Path = syn::parse_str(&v.function).unwrap();

    let arg_quoted = if let Some(ref args) = v.args {
        let arg_type = &args.arg_access;
        quote!(self, #arg_type)
    } else {
        quote!(self)
    };

    quote!(
        match #fn_ident(#arg_quoted) {
            ::std::result::Result::Ok(()) => (),
            ::std::result::Result::Err(mut errs) => {
                errors.merge(errs);
            },
        };
    )
}

pub fn quote_schema_validations(validation: &[SchemaValidation]) -> Vec<proc_macro2::TokenStream> {
    validation.iter().map(quote_schema_validation).collect()
}

pub fn quote_required_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let ident = &field_quoter.ident;
    let validator_param = quote!(&self.#ident);

    let quoted_error = quote_error(validation, Some(field_name));
    let quoted = quote!(
        if !::validator::validate_required(#validator_param) {
            #quoted_error
            err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
            errors.add(err);
        }
    );

    quoted
}

pub fn quote_does_not_contain_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    if let Validator::DoesNotContain(ref needle) = validation.validator {
        let quoted_error = quote_error(validation, Some(field_name));
        let quoted = quote!(
            if !::validator::validate_does_not_contain(#validator_param, &#needle) {
                #quoted_error
                err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
                err.add_param(::std::borrow::Cow::from("needle"), &#needle);
                errors.add(err);
            }
        );

        return field_quoter.wrap_if_option(quoted);
    }

    unreachable!();
}
