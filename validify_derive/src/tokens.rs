use crate::fields::FieldInfo;
use crate::validate::parser::ValueOrPath;
use crate::validate::validation::{
    Contains, CreditCard, Custom, Describe, Email, In, Ip, Length, MustMatch, NonControlChar,
    Phone, Range, Regex, Required, SchemaValidation, Time, TimeMultiplier, Url, Validator,
};
use proc_macro2::{self};
use quote::quote;

/// Implement on validators/modifiers that need to be expanded in the derive invocation.
pub trait ToValidationTokens {
    /// Output the validation/modification tokens in a validation type
    /// that specifies whether the tokens are a direct validation/modification call or a nested
    /// validation/modification call.
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationTokens;

    /// Quote an error based on the validation's settings
    fn quote_error(&self, field_name: &str) -> proc_macro2::TokenStream
    where
        Self: Describe,
    {
        let add_message_quoted = if let Some(ref m) = self.message() {
            quote!(err.set_message(String::from(#m));)
        } else {
            quote!()
        };

        let code = self.code();

        quote!(
            let mut err = ::validify::ValidationError::new_field(#code);
            err.set_field(#field_name);
            #add_message_quoted
        )
    }
}

/// Whether the tokens are for nested or direct validations.
pub enum ValidationTokens {
    Normal(proc_macro2::TokenStream),
    Nested(proc_macro2::TokenStream),
}

/// Output the necessary tokens for schema validation when implementing `Validate`.
pub fn quote_schema_validations(validation: &[SchemaValidation]) -> Vec<proc_macro2::TokenStream> {
    validation
        .iter()
        .map(|v| {
            let fn_ident = &v.function;
            quote!(
                if let Err(mut errs) = #fn_ident(&self) {
                        errors.merge(errs);
                };
            )
        })
        .collect()
}

/// Output the necessary tokens for field validations when implementing `Validate`.
pub fn quote_field_validations(fields: Vec<FieldInfo>) -> Vec<proc_macro2::TokenStream> {
    let mut validations = vec![];

    for field_info in fields {
        let tokens = field_info.quote_validation();
        validations.extend(tokens);
    }

    validations
}

/// Creates a token stream applying the modifiers based on the field annotations.
pub(super) fn quote_field_modifiers(
    fields: Vec<FieldInfo>,
) -> (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>) {
    let mut modifications = vec![];
    let mut nested_validifies = vec![];

    for field_info in fields {
        let (mods, nested) = field_info.quote_validifes();
        modifications.extend(mods);
        nested_validifies.extend(nested);
    }

    (modifications, nested_validifies)
}

impl ToValidationTokens for Validator {
    fn to_validify_tokens(&self, field_info: &crate::fields::FieldInfo) -> ValidationTokens {
        match self {
            Validator::Email(v) => v.to_validify_tokens(field_info),
            Validator::Url(v) => v.to_validify_tokens(field_info),
            Validator::CreditCard(v) => v.to_validify_tokens(field_info),
            Validator::Phone(v) => v.to_validify_tokens(field_info),
            Validator::Custom(v) => v.to_validify_tokens(field_info),
            Validator::Range(v) => v.to_validify_tokens(field_info),
            Validator::Length(v) => v.to_validify_tokens(field_info),
            Validator::NonControlCharacter(v) => v.to_validify_tokens(field_info),
            Validator::Required(v) => v.to_validify_tokens(field_info),
            Validator::MustMatch(v) => v.to_validify_tokens(field_info),
            Validator::Regex(v) => v.to_validify_tokens(field_info),
            Validator::Contains(v) => v.to_validify_tokens(field_info),
            Validator::Time(v) => v.to_validify_tokens(field_info),
            Validator::In(v) => v.to_validify_tokens(field_info),
            Validator::Ip(v) => v.to_validify_tokens(field_info),
            Validator::Nested => {
                let validator_field = field_info.quote_validator_field();
                let field_name = field_info.name();
                let quoted = quote!(
                    if let Err(mut errs) = #validator_field.validate() {
                        errs.errors_mut().iter_mut().for_each(|err| err.set_location(#field_name));
                        errors.merge(errs);
                    }
                );
                ValidationTokens::Nested(field_info.wrap_tokens_if_option(
                    field_info.wrap_validator_if_collection(validator_field, quoted),
                ))
            }
        }
    }
}

impl ToValidationTokens for Time {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationTokens {
        let field_name = field_info.name();
        let validator_param = field_info.quote_validator_param();
        let quoted_error = self.quote_error(field_name);

        let Time {
            op,
            inclusive,
            multiplier,
            ref format,
            ref duration,
            ref target,
            ..
        } = self;

        let code = self.code();
        let quoted_parse_error = quote!(
            let mut err = ::validify::ValidationError::new_field(#code);
            err.set_field(#field_name);
            err.add_param("actual", &#validator_param);
            err.add_param("format", &#format);
            err.set_location(#field_name);
            errors.add(err);
        );

        let has_time = field_info.has_time();
        let duration = if let Some(duration) = duration {
            match duration {
                ValueOrPath::Value(val) => quote!(chrono::Duration::seconds(#val)),
                ValueOrPath::Path(path) => match multiplier {
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

        use crate::validate::validation::TimeOp as TO;
        let validation_fn = match op {
            TO::BeforeNow => {
                if has_time {
                    quote!(::validify::time::before_now(#validator_param, #inclusive))
                } else {
                    quote!(::validify::time::before_today(#validator_param, #inclusive))
                }
            }
            TO::AfterNow => {
                if has_time {
                    quote!(::validify::time::after_now(#validator_param, #inclusive))
                } else {
                    quote!(::validify::time::after_today(#validator_param, #inclusive))
                }
            }
            TO::BeforeFromNow => {
                if has_time {
                    quote!(::validify::time::before_from_now(#validator_param, #duration))
                } else {
                    quote!(::validify::time::before_from_now_date(#validator_param, #duration))
                }
            }
            TO::AfterFromNow => {
                if has_time {
                    quote!(::validify::time::after_from_now(#validator_param, #duration))
                } else {
                    quote!(::validify::time::after_from_now_date(#validator_param, #duration))
                }
            }
            TO::Before => {
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
                return ValidationTokens::Normal(field_info.wrap_tokens_if_option(quoted));
            }
            TO::After => {
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
                return ValidationTokens::Normal(field_info.wrap_tokens_if_option(quoted));
            }
            TO::InPeriod => {
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
                return ValidationTokens::Normal(field_info.wrap_tokens_if_option(quoted));
            }
            TO::None => unreachable!(),
        };

        let quoted = quote!(
            if !#validation_fn {
                #quoted_error
                err.add_param("actual", &#validator_param);
                err.set_location(#field_name);
                errors.add(err);
            }
        );

        ValidationTokens::Normal(field_info.wrap_tokens_if_option(quoted))
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

impl ToValidationTokens for Ip {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationTokens {
        let field_name = field_info.name();
        let validator_param = field_info.quote_validator_param();
        let quoted_error = self.quote_error(field_name);

        let Ip { ref format, .. } = self;

        let validate_fn = match format {
            Some(format) => match format {
                crate::validate::validation::IpFormat::V4 => quote!(validate_ip_v4),
                crate::validate::validation::IpFormat::V6 => quote!(validate_ip_v6),
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

        ValidationTokens::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

impl ToValidationTokens for Length {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationTokens {
        let field_name = field_info.name();
        let validator_param = field_info.quote_validator_param();

        let Length {
            ref min,
            ref max,
            ref equal,
            ..
        } = self;

        let min_err_param_quoted = if let Some(ref v) = min {
            quote!(err.add_param("min", &#v);)
        } else {
            quote!()
        };

        let max_err_param_quoted = if let Some(ref v) = max {
            quote!(err.add_param("max", &#v);)
        } else {
            quote!()
        };

        let equal_err_param_quoted = if let Some(ref v) = equal {
            quote!(err.add_param("equal", &#v);)
        } else {
            quote!()
        };

        let min_tokens = &min
            .as_ref()
            .map(ValueOrPath::tokens)
            .map(|x| quote!(Some(#x as u64)))
            .unwrap_or(quote!(None));

        let max_tokens = &max
            .as_ref()
            .map(ValueOrPath::tokens)
            .map(|x| quote!(Some(#x as u64)))
            .unwrap_or(quote!(None));

        let equal_tokens = &equal
            .as_ref()
            .map(ValueOrPath::tokens)
            .map(|x| quote!(Some(#x as u64)))
            .unwrap_or(quote!(None));

        let quoted_error = self.quote_error(field_name);

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

        ValidationTokens::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

impl ToValidationTokens for Range {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationTokens {
        let field_name = field_info.name();
        let quoted_ident = field_info.quote_validator_param();

        let Range {
            ref min, ref max, ..
        } = self;

        let min_err_param_quoted = if let Some(v) = min {
            quote!(err.add_param("min", &#v);)
        } else {
            quote!()
        };

        let max_err_param_quoted = if let Some(v) = max {
            quote!(err.add_param("max", &#v);)
        } else {
            quote!()
        };

        let min_tokens = min
            .as_ref()
            .map(ValueOrPath::tokens)
            .map(|x| quote!(Some(#x as f64)))
            .unwrap_or(quote!(None));

        let max_tokens = max
            .as_ref()
            .map(ValueOrPath::tokens)
            .map(|x| quote!(Some(#x as f64)))
            .unwrap_or(quote!(None));

        let quoted_error = self.quote_error(field_name);
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

        ValidationTokens::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

impl ToValidationTokens for CreditCard {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationTokens {
        let field_name = field_info.name();
        let validator_param = field_info.quote_validator_param();

        let quoted_error = self.quote_error(field_name);
        let quoted = quote!(
            if !::validify::validate_credit_card(#validator_param) {
                #quoted_error
                err.add_param("actual", &#validator_param);
                err.set_location(#field_name);
                errors.add(err);
            }
        );

        ValidationTokens::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

impl ToValidationTokens for Phone {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationTokens {
        let field_name = field_info.name();
        let validator_param = field_info.quote_validator_param();

        let quoted_error = self.quote_error(field_name);
        let quoted = quote!(
            if !::validify::validate_phone(#validator_param) {
                #quoted_error
                err.add_param("actual", &#validator_param);
                err.set_location(#field_name);
                errors.add(err);
            }
        );

        ValidationTokens::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

impl ToValidationTokens for NonControlChar {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationTokens {
        let field_name = field_info.name();
        let validator_param = field_info.quote_validator_param();

        let quoted_error = self.quote_error(field_name);
        let quoted = quote!(
            if !::validify::validate_non_control_character(#validator_param) {
                #quoted_error
                err.add_param("actual", &#validator_param);
                err.set_location(#field_name);
                errors.add(err);
            }
        );

        ValidationTokens::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}
impl ToValidationTokens for Url {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationTokens {
        let field_name = field_info.name();
        let validator_param = field_info.quote_validator_param();

        let quoted_error = self.quote_error(field_name);
        let quoted = quote!(
            if !::validify::validate_url(#validator_param) {
                #quoted_error
                err.add_param("actual", &#validator_param);
                err.set_location(#field_name);
                errors.add(err);
            }
        );

        ValidationTokens::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

impl ToValidationTokens for Email {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationTokens {
        let field_name = field_info.name();
        let validator_param = field_info.quote_validator_param();

        let quoted_error = self.quote_error(field_name);
        let quoted = quote!(
            if !::validify::validate_email(#validator_param) {
                #quoted_error
                err.add_param("actual", &#validator_param);
                err.set_location(#field_name);
                errors.add(err);
            }
        );

        ValidationTokens::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

impl ToValidationTokens for MustMatch {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationTokens {
        let ident = &field_info.field.ident;
        let field_name = field_info.name();
        let MustMatch { ref value, .. } = self;
        let quoted_error = self.quote_error(field_name);
        let quoted = quote!(
            if !::validify::validate_must_match(&self.#ident, &self.#value) {
                #quoted_error
                err.add_param("actual", &self.#ident);
                err.add_param("target", &self.#value);
                err.set_location(#field_name);
                errors.add(err);
            }
        );

        ValidationTokens::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

impl ToValidationTokens for Custom {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationTokens {
        let validator_param = field_info.quote_validator_param();
        let field_name = field_info.name();

        let Custom { ref path, .. } = self;

        let err_with_msg = if let Some(msg) = self.message() {
            quote!(err.with_message(#msg.to_string()))
        } else {
            quote!(err)
        };

        let quoted = quote!(
            if let Err(mut err) = #path(#validator_param) {
                let f_name = err.field_name().map(|s|s.to_string());
                if let Some(field_name) = f_name {
                    err.set_location(field_name);
                } else {
                    err.set_field(#field_name);
                    err.set_location(#field_name);
                }
                errors.add(#err_with_msg);
            };
        );

        ValidationTokens::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

impl ToValidationTokens for Regex {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationTokens {
        let field_name = field_info.name();
        let validator_param = field_info.quote_validator_param();

        let Regex { ref path, .. } = self;
        let quoted_error = self.quote_error(field_name);

        let quoted = quote!(
        if !#path.is_match(#validator_param) {
            #quoted_error
            err.add_param("actual", &#validator_param);
            err.set_location(#field_name);
            errors.add(err);
        });

        ValidationTokens::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

/// This is a bit of a special case where we can't use the wrap if option since this is usually used with const slices where we'll
/// usually need a double reference
impl ToValidationTokens for In {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationTokens {
        let field_name = field_info.name();

        let field_ident = &field_info.field.ident;
        let In { ref path, not, .. } = self;
        let quoted_error = self.quote_error(field_name);

        // Cast strings to strs because the usual application for string comparisons
        // with this kind of validation is with const arrays
        let as_str = if field_info.is_string() {
            quote!(.as_str())
        } else {
            quote!()
        };

        if field_info.is_option() {
            return ValidationTokens::Normal(quote!(
                if let Some(ref param) = self.#field_ident {
                    if !::validify::validate_in(#path, &param #as_str, #not) {
                        #quoted_error
                        err.set_location(#field_name);
                        errors.add(err);
                    }
                }
            ));
        }

        ValidationTokens::Normal(quote!(
            if !::validify::validate_in(#path, &self.#field_ident #as_str, #not) {
                #quoted_error
                err.set_location(#field_name);
                errors.add(err);
        }))
    }
}

impl ToValidationTokens for Contains {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationTokens {
        let field_name = field_info.name();
        let validator_param = field_info.quote_validator_param();
        let Contains { not, ref value, .. } = self;

        let quoted_error = self.quote_error(field_name);

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

        ValidationTokens::Normal(field_info.wrap_tokens_if_option(quoted))
    }
}

impl ToValidationTokens for Required {
    fn to_validify_tokens(&self, field_info: &FieldInfo) -> ValidationTokens {
        let field_name = field_info.name();
        let ident = &field_info.field.ident;
        let validator_param = quote!(&self.#ident);

        let quoted_error = self.quote_error(field_name);
        ValidationTokens::Normal(quote!(
            if !::validify::validate_required(#validator_param) {
                #quoted_error
                err.set_location(#field_name);
                errors.add(err);
            }
        ))
    }
}
