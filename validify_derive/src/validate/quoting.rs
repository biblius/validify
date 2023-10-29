use super::parsing::option_to_tokens;
use crate::fields::FieldInfo;
use crate::types::{
    Contains, CreditCard, Custom, Describe, Email, In, Ip, Length, MustMatch, NonControlChar,
    Phone, Range, Regex, Required, SchemaValidation, Time, TimeMultiplier, ToValidifyTokens, Url,
    ValidationType, ValueOrPath,
};
use proc_macro2::{self};
use quote::quote;

pub fn quote_field_validations(fields: Vec<FieldInfo>) -> Vec<proc_macro2::TokenStream> {
    let mut validations = vec![];

    for field_info in fields {
        let tokens = field_info.quote_validation();
        validations.extend(tokens);
    }

    validations

    /* fields.drain(..).for_each(|f_info| {
        for validator in f_info.validations.iter() {
            quote_validator(
                &f_info,
                validator,
                &mut quoted_validations,
                &mut nested_validations,
            );
        }
    }); */
}

pub fn quote_struct_validations(validation: &[SchemaValidation]) -> Vec<proc_macro2::TokenStream> {
    validation.iter().map(quote_struct_validation).collect()
}

fn quote_struct_validation(validation: &SchemaValidation) -> proc_macro2::TokenStream {
    let fn_ident = &validation.function;
    quote!(
        match #fn_ident(&self) {
            ::std::result::Result::Ok(()) => {},
            ::std::result::Result::Err(mut errs) => {
                errors.merge(errs);
            },
        };
    )
}

/* fn quote_validator(
    field_info: &FieldInfo,
    validator: &Validator,
) -> {
    match validator {
        Validator::Length(length) => length.to_validify_tokens(field_info),
        Validator::Range(validation) => {
            validations.push(quote_range_validation(field_info, validation))
        }
        Validator::Email(validation) => {
            validations.push(quote_email_validation(field_info, validation))
        }
        Validator::Url(validation) => {
            validations.push(quote_url_validation(field_info, validation))
        }
        Validator::MustMatch(validation) => {
            validations.push(quote_must_match_validation(field_info, validation))
        }
        Validator::Custom(validation) => {
            validations.push(quote_custom_validation(field_info, validation))
        }
        Validator::Contains(validation) => {
            validations.push(quote_contains_validation(field_info, validation))
        }
        Validator::Regex(validation) => {
            validations.push(quote_regex_validation(field_info, validation))
        }
        Validator::CreditCard(validation) => {
            validations.push(quote_credit_card_validation(field_info, validation))
        }
        Validator::Phone(validation) => {
            validations.push(quote_phone_validation(field_info, validation))
        }
        Validator::NonControlCharacter(validation) => validations.push(
            quote_non_control_character_validation(field_info, validation),
        ),
        Validator::Required(validation) => {
            validations.push(quote_required_validation(field_info, validation))
        }
        Validator::In(validation) => validations.push(quote_in_validation(field_info, validation)),
        Validator::Ip(validation) => validations.push(quote_ip_validation(field_info, validation)),
        Validator::Time(validation) => {
            validations.push(quote_time_validation(field_info, validation))
        }
        Validator::Nested => nested_validations.push(quote_nested_validation(field_info)),
    }
}
 */
/// Quote an error based on the validation's settings
fn quote_error(describe: &impl Describe, field_name: &str) -> proc_macro2::TokenStream {
    let add_message_quoted = if let Some(ref m) = describe.message() {
        quote!(err.set_message(String::from(#m));)
    } else {
        quote!()
    };

    let code = describe.code();

    quote!(
        let mut err = ::validify::ValidationError::new_field(#field_name, #code);
        #add_message_quoted
    )
}
impl ToValidifyTokens for Time {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationType {
        let field_name = field_info.name();
        let validator_param = field_info.quote_validator_param();
        let quoted_error = quote_error(self, field_name);

        let Time {
            op,
            inclusive,
            path_type,
            ref format,
            ref duration,
            ref target,
            ..
        } = self;

        let code = self.code();
        let quoted_parse_error = quote!(
            let mut err = ::validify::ValidationError::new_field(#field_name, #code);
            err.add_param("actual", &#validator_param);
            err.add_param("format", &#format);
            err.set_location(#field_name);
            errors.add(err);
        );

        let has_time = if let syn::Type::Path(ref p) = field_info.field.ty {
            if let Some(i) = p.path.get_ident() {
                i.to_string().contains("Time")
            } else if let Some(seg) = p.path.segments.last() {
                if seg.ident == "Option" {
                    if let syn::PathArguments::AngleBracketed(ref args) = seg.arguments {
                        args.args.last().is_some_and(|arg| match arg {
                            syn::GenericArgument::Type(syn::Type::Path(ty)) => ty
                                .path
                                .segments
                                .last()
                                .is_some_and(|seg| seg.ident.to_string().contains("Time")),
                            _ => false,
                        })
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };
        let duration = if let Some(duration) = duration {
            match duration {
                ValueOrPath::Value(val) => quote!(chrono::Duration::seconds(#val)),
                ValueOrPath::Path(path) => match path_type {
                    TimeMultiplier::Seconds => quote!(chrono::Duration::seconds(#path)),
                    TimeMultiplier::Minutes => quote!(chrono::Duration::minutes(#path)),
                    TimeMultiplier::Hours => quote!(chrono::Duration::hours(#path)),
                    TimeMultiplier::Days => quote!(chrono::Duration::days(#path)),
                    TimeMultiplier::Weeks => quote!(chrono::Duration::weeks(#path)),
                    TimeMultiplier::None => unreachable!(),
                },
            }
        } else {
            quote!()
        };

        use crate::types::TimeOp::*;
        let validation_fn = match op {
            BeforeNow => {
                if has_time {
                    quote!(::validify::time::before_now(#validator_param, #inclusive))
                } else {
                    quote!(::validify::time::before_today(#validator_param, #inclusive))
                }
            }
            AfterNow => {
                if has_time {
                    quote!(::validify::time::after_now(#validator_param, #inclusive))
                } else {
                    quote!(::validify::time::after_today(#validator_param, #inclusive))
                }
            }
            BeforeFromNow => {
                if has_time {
                    quote!(::validify::time::before_from_now(#validator_param, #duration))
                } else {
                    quote!(::validify::time::before_from_now_date(#validator_param, #duration))
                }
            }
            AfterFromNow => {
                if has_time {
                    quote!(::validify::time::after_from_now(#validator_param, #duration))
                } else {
                    quote!(::validify::time::after_from_now_date(#validator_param, #duration))
                }
            }
            Before => {
                let validation = {
                    let validation_fn = if has_time {
                        quote!(!::validify::time::before(#validator_param, target, #inclusive))
                    } else {
                        quote!(!::validify::time::before_date(#validator_param, target, #inclusive))
                    };
                    quote!(
                        if #validation_fn {
                            #quoted_error
                            err.add_param("actual", &#validator_param);
                            err.add_param("target", target);
                            err.set_location(#field_name);
                            errors.add(err);
                        }
                    )
                };
                let quoted = quote_time_with_target(
                    target.as_ref().unwrap(),
                    validation,
                    quoted_parse_error,
                    format.as_deref(),
                    has_time,
                );
                return ValidationType::Normal(field_info.wrap_tokens_if_option(quoted));
            }
            After => {
                let validation = {
                    let validation_fn = if has_time {
                        quote!(!::validify::time::after(#validator_param, target, #inclusive))
                    } else {
                        quote!(!::validify::time::after_date(#validator_param, target, #inclusive))
                    };
                    quote!(
                        if #validation_fn {
                            #quoted_error
                            err.add_param("actual", &#validator_param);
                            err.add_param("target", target);
                            err.set_location(#field_name);
                            errors.add(err);
                        }
                    )
                };
                let quoted = quote_time_with_target(
                    target.as_ref().unwrap(),
                    validation,
                    quoted_parse_error,
                    format.as_deref(),
                    has_time,
                );
                return ValidationType::Normal(field_info.wrap_tokens_if_option(quoted));
            }
            InPeriod => {
                let validation = {
                    let validation_fn = if has_time {
                        quote!(!::validify::time::in_period(#validator_param, target, #duration))
                    } else {
                        quote!(!::validify::time::in_period_date(#validator_param, target, #duration))
                    };

                    // We can safely unwrap since we do a check for overflow before quoting
                    quote!(
                        if #validation_fn {
                            #quoted_error
                            err.add_param("actual", &#validator_param);
                            let end = target.checked_add_signed(#duration).unwrap();
                            if end < *target {
                                err.add_param("from", &end);
                                err.add_param("to", target);
                            } else {
                                err.add_param("from", target);
                                err.add_param("to", &end);
                            }
                            err.set_location(#field_name);
                            errors.add(err);
                        }
                    )
                };
                let quoted = quote_time_with_target(
                    target.as_ref().unwrap(),
                    validation,
                    quoted_parse_error,
                    format.as_deref(),
                    has_time,
                );
                return ValidationType::Normal(field_info.wrap_tokens_if_option(quoted));
            }
            None => unreachable!(),
        };

        let quoted = quote!(
            if !#validation_fn {
                #quoted_error
                err.add_param("actual", &#validator_param);
                err.set_location(#field_name);
                errors.add(err);
            }
        );

        ValidationType::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

/// Quotes the tokens based on the target `value_or_path`. If the target is a string,
/// it will be parsed based on the format.
fn quote_time_with_target(
    value_or_path: &ValueOrPath<String>,
    quoted_validation: proc_macro2::TokenStream,
    quoted_parse_error: proc_macro2::TokenStream,
    format: Option<&str>,
    has_time: bool,
) -> proc_macro2::TokenStream {
    match value_or_path {
        ValueOrPath::Value(value) => {
            let format = format.unwrap();
            let parse_fn = if has_time {
                quote!(chrono::NaiveDateTime::parse_from_str(#value, #format))
            } else {
                quote!(chrono::NaiveDate::parse_from_str(#value, #format))
            };
            quote!(
                if let Ok(ref target) = #parse_fn {
                    #quoted_validation
                } else {
                    #quoted_parse_error
                }
            )
        }
        ValueOrPath::Path(target) => {
            quote!(
                let target = &#target();
                #quoted_validation
            )
        }
    }
}

impl ToValidifyTokens for Ip {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationType {
        let field_name = field_info.name();
        let validator_param = field_info.quote_validator_param();
        let quoted_error = quote_error(self, field_name);

        let Ip { ref format, .. } = self;

        let validate_fn = match format {
            Some(format) => match format {
                crate::types::IpFormat::V4 => quote!(validate_ip_v4),
                crate::types::IpFormat::V6 => quote!(validate_ip_v6),
            },
            None => quote!(validate_ip),
        };

        let quoted = quote!(
            if !::validify::#validate_fn(#validator_param) {
                #quoted_error
                err.add_param("actual", &#validator_param);
                err.set_location(#field_name);
                errors.add(err);
            }
        );

        ValidationType::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

impl ToValidifyTokens for Length {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationType {
        let field_name = field_info.name();
        let validator_param = field_info.quote_validator_param();

        let Length {
            ref min,
            ref max,
            ref equal,
            ..
        } = self;

        let min_err_param_quoted = if let Some(ref v) = min {
            let v = v.to_tokens();
            quote!(err.add_param("min", &#v);)
        } else {
            quote!()
        };

        let max_err_param_quoted = if let Some(ref v) = max {
            let v = v.to_tokens();
            quote!(err.add_param("max", &#v);)
        } else {
            quote!()
        };

        let equal_err_param_quoted = if let Some(ref v) = equal {
            let v = v.to_tokens();
            quote!(err.add_param("equal", &#v);)
        } else {
            quote!()
        };

        let min_tokens = option_to_tokens(
            &min.clone()
                .as_ref()
                .map(ValueOrPath::to_tokens)
                .map(|x| quote!(#x as u64)),
        );

        let max_tokens = option_to_tokens(
            &max.clone()
                .as_ref()
                .map(ValueOrPath::to_tokens)
                .map(|x| quote!(#x as u64)),
        );

        let equal_tokens = option_to_tokens(
            &equal
                .clone()
                .as_ref()
                .map(ValueOrPath::to_tokens)
                .map(|x| quote!(#x as u64)),
        );

        let quoted_error = quote_error(self, field_name);

        let is_collection = field_info.is_list() || field_info.is_map();
        // For strings it's ok to add the param, but we don't want to add whole collections
        let added_param =
            (!is_collection).then_some(quote!(err.add_param("actual", &#validator_param);));

        let quoted = quote!(
            if !::validify::validate_length(
                #validator_param,
                #min_tokens,
                #max_tokens,
                #equal_tokens
            ) {
                #quoted_error
                #min_err_param_quoted
                #max_err_param_quoted
                #equal_err_param_quoted
                #added_param
                err.set_location(#field_name);
                errors.add(err);
            }
        );

        ValidationType::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

impl ToValidifyTokens for Range {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationType {
        let field_name = field_info.name();
        let quoted_ident = field_info.quote_validator_param();

        let Range {
            ref min, ref max, ..
        } = self;

        let min_err_param_quoted = if let Some(v) = min {
            let v = v.to_tokens();
            quote!(err.add_param("min", &#v);)
        } else {
            quote!()
        };

        let max_err_param_quoted = if let Some(v) = max {
            let v = v.to_tokens();
            quote!(err.add_param("max", &#v);)
        } else {
            quote!()
        };

        // Can't interpolate None
        let min_tokens = min
            .as_ref()
            .map(ValueOrPath::to_tokens)
            .map(|x| quote!(#x as f64));

        let min_tokens = option_to_tokens(&min_tokens);

        let max_tokens = max
            .as_ref()
            .map(ValueOrPath::to_tokens)
            .map(|x| quote!(#x as f64));

        let max_tokens = option_to_tokens(&max_tokens);

        let quoted_error = quote_error(self, field_name);
        let quoted = quote!(
            if !::validify::validate_range(
                *#quoted_ident as f64,
                #min_tokens,
                #max_tokens
            ) {
                #quoted_error
                #min_err_param_quoted
                #max_err_param_quoted
                err.add_param("actual", &#quoted_ident);
                err.set_location(#field_name);
                errors.add(err);
            }
        );

        ValidationType::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

impl ToValidifyTokens for CreditCard {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationType {
        let field_name = field_info.name();
        let validator_param = field_info.quote_validator_param();

        let quoted_error = quote_error(self, field_name);
        let quoted = quote!(
            if !::validify::validate_credit_card(#validator_param) {
                #quoted_error
                err.add_param("actual", &#validator_param);
                err.set_location(#field_name);
                errors.add(err);
            }
        );

        ValidationType::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

impl ToValidifyTokens for Phone {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationType {
        let field_name = field_info.name();
        let validator_param = field_info.quote_validator_param();

        let quoted_error = quote_error(self, field_name);
        let quoted = quote!(
            if !::validify::validate_phone(#validator_param) {
                #quoted_error
                err.add_param("actual", &#validator_param);
                err.set_location(#field_name);
                errors.add(err);
            }
        );

        ValidationType::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

impl ToValidifyTokens for NonControlChar {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationType {
        let field_name = field_info.name();
        let validator_param = field_info.quote_validator_param();

        let quoted_error = quote_error(self, field_name);
        let quoted = quote!(
            if !::validify::validate_non_control_character(#validator_param) {
                #quoted_error
                err.add_param("actual", &#validator_param);
                err.set_location(#field_name);
                errors.add(err);
            }
        );

        ValidationType::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}
impl ToValidifyTokens for Url {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationType {
        let field_name = field_info.name();
        let validator_param = field_info.quote_validator_param();

        let quoted_error = quote_error(self, field_name);
        let quoted = quote!(
            if !::validify::validate_url(#validator_param) {
                #quoted_error
                err.add_param("actual", &#validator_param);
                err.set_location(#field_name);
                errors.add(err);
            }
        );

        ValidationType::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

impl ToValidifyTokens for Email {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationType {
        let field_name = field_info.name();
        let validator_param = field_info.quote_validator_param();

        let quoted_error = quote_error(self, field_name);
        let quoted = quote!(
            if !::validify::validate_email(#validator_param) {
                #quoted_error
                err.add_param("actual", &#validator_param);
                err.set_location(#field_name);
                errors.add(err);
            }
        );

        ValidationType::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

impl ToValidifyTokens for MustMatch {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationType {
        let ident = &field_info.field.ident;
        let field_name = field_info.name();
        let MustMatch { ref value, .. } = self;
        let quoted_error = quote_error(self, field_name);
        let quoted = quote!(
            if !::validify::validate_must_match(&self.#ident, &self.#value) {
                #quoted_error
                err.add_param("actual", &self.#ident);
                err.add_param("target", &self.#value);
                err.set_location(#field_name);
                errors.add(err);
            }
        );

        ValidationType::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

impl ToValidifyTokens for Custom {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationType {
        let validator_param = field_info.quote_validator_param();
        let Custom { ref path, .. } = self;

        let err_with_msg = if let Some(msg) = self.message() {
            quote!(err.with_message(#msg.to_string()))
        } else {
            quote!(err)
        };

        let quoted = quote!(
            match #path(#validator_param) {
                ::std::result::Result::Ok(()) => (),
                ::std::result::Result::Err(mut err) => {
                    let field = err.field_name().unwrap().to_string();
                    err.set_location(field);
                    errors.add(#err_with_msg);
                },
            };
        );

        ValidationType::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

impl ToValidifyTokens for Regex {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationType {
        let field_name = field_info.name();
        let validator_param = field_info.quote_validator_param();

        let Regex { ref path, .. } = self;
        let quoted_error = quote_error(self, field_name);

        let quoted = quote!(
        if !#path.is_match(#validator_param) {
            #quoted_error
            err.add_param("actual", &#validator_param);
            err.set_location(#field_name);
            errors.add(err);
        });

        ValidationType::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

/// This is a bit of a special case where we can't use the wrap if option since this is usually used with const slices where we'll
/// usually need a double reference
impl ToValidifyTokens for In {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationType {
        let field_name = field_info.name();

        let field_ident = &field_info.field.ident;
        let In { ref path, not, .. } = self;
        let quoted_error = quote_error(self, field_name);

        // Cast strings to strs because the usual application for string comparisons
        // with this kind of validation is with const arrays
        let as_str = if field_info.is_string() {
            quote!(.as_str())
        } else {
            quote!()
        };

        if field_info.is_option() {
            return ValidationType::Normal(quote!(
                if let Some(ref param) = self.#field_ident {
                    if !::validify::validate_in(#path, &param #as_str, #not) {
                        #quoted_error
                        err.set_location(#field_name);
                        errors.add(err);
                    }
                }
            ));
        }

        ValidationType::Normal(quote!(
            if !::validify::validate_in(#path, &self.#field_ident #as_str, #not) {
                #quoted_error
                err.set_location(#field_name);
                errors.add(err);
        }))
    }
}

impl ToValidifyTokens for Contains {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationType {
        let field_name = field_info.name();
        let validator_param = field_info.quote_validator_param();
        let Contains { not, ref value, .. } = self;

        let quoted_error = quote_error(self, field_name);

        let validation_val = if matches!(value, Some(ValueOrPath::Value(syn::Lit::Str(_)))) {
            quote!(String::from(#value))
        } else {
            quote!(#value)
        };

        // Only add the target if it's a literal since otherwise it will just be the variable name
        let added_param = matches!(value, Some(ValueOrPath::Value(_)))
            .then_some(quote!(err.add_param("target", &#value);));

        // Only add the value if it's a string since we don't want to serialize whole collections
        let added_value = (!field_info.is_list() && !field_info.is_map())
            .then_some(quote!(err.add_param("actual", &#validator_param);));

        let quoted = quote!(
            if !::validify::validate_contains(#validator_param, &#validation_val, #not) {
                #quoted_error
                #added_param
                #added_value
                err.set_location(#field_name);
                errors.add(err);
            }
        );

        ValidationType::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

impl ToValidifyTokens for Required {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationType {
        let field_name = field_info.name();
        let ident = &field_info.field.ident;
        let validator_param = quote!(&self.#ident);

        let quoted_error = quote_error(self, field_name);
        ValidationType::Normal(quote!(
            if !::validify::validate_required(#validator_param) {
                #quoted_error
                err.set_location(#field_name);
                errors.add(err);
            }
        ))
    }
}
