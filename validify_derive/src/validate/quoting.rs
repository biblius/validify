use super::parsing::option_to_tokens;
use crate::fields::FieldInformation;
use crate::quoter::FieldQuoter;
use crate::types::{
    Contains, CreditCard, Custom, Describe, Email, In, Length, MustMatch, NonControlChar, Phone,
    Range, Regex, Required, SchemaValidation, Url, ValueOrPath,
};
use crate::Validator;
use proc_macro2::{self};
use quote::quote;

/// Quote an actual end-user error creation automatically
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

pub fn quote_field_validations(
    mut fields: Vec<FieldInformation>,
) -> (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>) {
    let mut validations = vec![];
    let mut nested_validations = vec![];

    fields.drain(..).for_each(|x| {
        let field_ident = x.field.ident.clone().unwrap();
        let field_quoter = FieldQuoter::new(field_ident, x.name, x.field_type);

        for validator in x.validations {
            quote_validator(
                &field_quoter,
                validator,
                &mut validations,
                &mut nested_validations,
            );
        }
    });

    (validations, nested_validations)
}

pub fn quote_validator(
    field_quoter: &FieldQuoter,
    validator: Validator,
    validations: &mut Vec<proc_macro2::TokenStream>,
    nested_validations: &mut Vec<proc_macro2::TokenStream>,
) {
    match validator {
        Validator::Length(length) => {
            validations.push(quote_length_validation(field_quoter, length))
        }
        Validator::Range(validation) => {
            validations.push(quote_range_validation(field_quoter, validation))
        }
        Validator::Email(validation) => {
            validations.push(quote_email_validation(field_quoter, validation))
        }
        Validator::Url(validation) => {
            validations.push(quote_url_validation(field_quoter, validation))
        }
        Validator::MustMatch(validation) => {
            validations.push(quote_must_match_validation(field_quoter, validation))
        }
        Validator::Custom(validation) => {
            validations.push(quote_custom_validation(field_quoter, validation))
        }
        Validator::Contains(validation) => {
            validations.push(quote_contains_validation(field_quoter, validation))
        }
        Validator::Regex(validation) => {
            validations.push(quote_regex_validation(field_quoter, validation))
        }
        Validator::CreditCard(validation) => {
            validations.push(quote_credit_card_validation(field_quoter, validation))
        }
        Validator::Phone(validation) => {
            validations.push(quote_phone_validation(field_quoter, validation))
        }
        Validator::Nested => nested_validations.push(quote_nested_validation(field_quoter)),
        Validator::NonControlCharacter(validation) => validations.push(
            quote_non_control_character_validation(field_quoter, validation),
        ),
        Validator::Required(validation) => {
            validations.push(quote_required_validation(field_quoter, validation))
        }
        Validator::In(validation) => {
            validations.push(quote_in_validation(field_quoter, validation))
        }
    }
}

pub fn quote_struct_validations(validation: &[SchemaValidation]) -> Vec<proc_macro2::TokenStream> {
    validation.iter().map(quote_struct_validation).collect()
}

pub fn quote_struct_validation(validation: &SchemaValidation) -> proc_macro2::TokenStream {
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

pub fn quote_length_validation(
    field_quoter: &FieldQuoter,
    length: Length,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    let Length {
        ref min,
        ref max,
        ref equal,
        ..
    } = length;

    let min_err_param_quoted = if let Some(ref v) = min {
        let v = v.to_tokens();
        quote!(err.add_param(::std::borrow::Cow::from("min"), &#v);)
    } else {
        quote!()
    };

    let max_err_param_quoted = if let Some(ref v) = max {
        let v = v.to_tokens();
        quote!(err.add_param(::std::borrow::Cow::from("max"), &#v);)
    } else {
        quote!()
    };

    let equal_err_param_quoted = if let Some(ref v) = equal {
        let v = v.to_tokens();
        quote!(err.add_param(::std::borrow::Cow::from("equal"), &#v);)
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

    let quoted_error = quote_error(&length, &field_name);

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
            err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
            errors.add(err);
        }
    );

    field_quoter.wrap_validator_if_option(quoted)
}

pub fn quote_range_validation(
    field_quoter: &FieldQuoter,
    range: Range,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let quoted_ident = field_quoter.quote_validator_param();

    let Range {
        ref min, ref max, ..
    } = range;

    let min_err_param_quoted = if let Some(v) = min {
        let v = v.to_tokens();
        quote!(err.add_param(::std::borrow::Cow::from("min"), &#v);)
    } else {
        quote!()
    };

    let max_err_param_quoted = if let Some(v) = max {
        let v = v.to_tokens();
        quote!(err.add_param(::std::borrow::Cow::from("max"), &#v);)
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

    let quoted_error = quote_error(&range, field_name);
    let quoted = quote!(
        if !::validify::validate_range(
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

    field_quoter.wrap_validator_if_option(quoted)
}

pub fn quote_credit_card_validation(
    field_quoter: &FieldQuoter,
    credit_card: CreditCard,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    let quoted_error = quote_error(&credit_card, field_name);
    let quoted = quote!(
        if !::validify::validate_credit_card(#validator_param) {
            #quoted_error
            err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
            errors.add(err);
        }
    );

    field_quoter.wrap_validator_if_option(quoted)
}

pub fn quote_phone_validation(
    field_quoter: &FieldQuoter,
    phone: Phone,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    let quoted_error = quote_error(&phone, field_name);
    let quoted = quote!(
        if !::validify::validate_phone(#validator_param) {
            #quoted_error
            err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
            errors.add(err);
        }
    );

    field_quoter.wrap_validator_if_option(quoted)
}

pub fn quote_non_control_character_validation(
    field_quoter: &FieldQuoter,
    non_cc: NonControlChar,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    let quoted_error = quote_error(&non_cc, field_name);
    let quoted = quote!(
        if !::validify::validate_non_control_character(#validator_param) {
            #quoted_error
            err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
            errors.add(err);
        }
    );

    field_quoter.wrap_validator_if_option(quoted)
}

pub fn quote_url_validation(field_quoter: &FieldQuoter, url: Url) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    let quoted_error = quote_error(&url, field_name);
    let quoted = quote!(
        if !::validify::validate_url(#validator_param) {
            #quoted_error
            err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
            errors.add(err);
        }
    );

    field_quoter.wrap_validator_if_option(quoted)
}

pub fn quote_email_validation(
    field_quoter: &FieldQuoter,
    email: Email,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    let quoted_error = quote_error(&email, field_name);
    let quoted = quote!(
        if !::validify::validate_email(#validator_param) {
            #quoted_error
            err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
            errors.add(err);
        }
    );

    field_quoter.wrap_validator_if_option(quoted)
}

pub fn quote_must_match_validation(
    field_quoter: &FieldQuoter,
    must_match: MustMatch,
) -> proc_macro2::TokenStream {
    let ident = &field_quoter.ident;
    let field_name = &field_quoter.name;
    let MustMatch { ref value, .. } = must_match;
    let quoted_error = quote_error(&must_match, field_name);
    let quoted = quote!(
        if !::validify::validate_must_match(&self.#ident, &self.#value) {
            #quoted_error
            err.add_param(::std::borrow::Cow::from("value"), &self.#ident);
            err.add_param(::std::borrow::Cow::from("other"), &self.#value);
            errors.add(err);
        }
    );

    field_quoter.wrap_validator_if_option(quoted)
}

pub fn quote_custom_validation(
    field_quoter: &FieldQuoter,
    custom: Custom,
) -> proc_macro2::TokenStream {
    let validator_param = field_quoter.quote_validator_param();
    let field_name = &field_quoter.name;
    let Custom { ref path, .. } = custom;
    let quoted_error = quote_error(&custom, field_name);

    let quoted = quote!(
        match #path(#validator_param) {
            ::std::result::Result::Ok(()) => (),
            ::std::result::Result::Err(mut err) => {
                #quoted_error
                err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
                errors.add(err);
            },
        };
    );

    field_quoter.wrap_validator_if_option(quoted)
}

pub fn quote_regex_validation(
    field_quoter: &FieldQuoter,
    regex: Regex,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    let Regex { ref path, .. } = regex;
    let quoted_error = quote_error(&regex, field_name);

    let quoted = quote!(
    if !#path.is_match(#validator_param) {
        #quoted_error
        err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
        errors.add(err);
    });

    field_quoter.wrap_validator_if_option(quoted)
}

pub fn quote_nested_validation(field_quoter: &FieldQuoter) -> proc_macro2::TokenStream {
    let validator_field = field_quoter.quote_validator_field();
    let quoted = quote!(
        if let Err(errs) = #validator_field.validate() {
            errors.merge(errs);
        }
    );
    field_quoter.wrap_validator_if_option(field_quoter.wrap_if_collection(quoted))
}

/// This is a bit of a special case where we can't use the wrap if option since this is usually used with const slices where we'll
/// usually need a double reference
pub fn quote_in_validation(field_quoter: &FieldQuoter, r#in: In) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;

    let validator_param = field_quoter.quote_validator_param();
    let In { ref path, not, .. } = r#in;
    let quoted_error = quote_error(&r#in, field_name);

    // Cast strings to strs because the usual application for string comparisons
    // is with const arrays
    let as_str = if field_quoter._type.contains("String") {
        quote!(.as_str())
    } else {
        quote!()
    };

    if field_quoter._type.starts_with("Option<") {
        return quote!(
            if let Some(ref param) = self.#validator_param {
                if !::validify::validate_in(#path, &param #as_str, #not) {
                    #quoted_error
                    err.add_param(::std::borrow::Cow::from("value"), &self.#validator_param);
                    err.add_param(::std::borrow::Cow::from("disallowed"), &#path);
                    errors.add(err);
                }
            }
        );
    }

    quote!(
        if !::validify::validate_in(#path, &#validator_param #as_str, #not) {
            #quoted_error
            err.add_param(::std::borrow::Cow::from("value"), #validator_param);
            err.add_param(::std::borrow::Cow::from("disallowed"), &#path);
            errors.add(err);
    })
}

pub fn quote_contains_validation(
    field_quoter: &FieldQuoter,
    contains: Contains,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();
    let Contains { not, ref value, .. } = contains;

    let quoted_error = quote_error(&contains, field_name);

    let quoted = quote!(
        if !::validify::validate_contains(#validator_param, &#value, #not) {
            #quoted_error
            err.add_param(::std::borrow::Cow::from("actual"), &#validator_param);
            err.add_param(::std::borrow::Cow::from("value"), &#value);
            errors.add(err);
        }
    );

    field_quoter.wrap_validator_if_option(quoted)
}

pub fn quote_required_validation(
    field_quoter: &FieldQuoter,
    required: Required,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let ident = &field_quoter.ident;
    let validator_param = quote!(&self.#ident);

    let quoted_error = quote_error(&required, field_name);
    quote!(
        if !::validify::validate_required(#validator_param) {
            #quoted_error
            err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
            errors.add(err);
        }
    )
}
