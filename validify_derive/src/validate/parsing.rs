use crate::types::{
    Contains, CreditCard, Custom, Email, In, Length, MustMatch, NonControlChar, Phone, Range,
    Regex, Required, Url, ValueOrPath,
};
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::quote;
use syn::{meta::ParseNestedMeta, punctuated::Punctuated, LitFloat, LitInt, LitStr};

macro_rules! parse_pattern {
    ($fn_id:ident, $id:ident) => {
        pub fn $fn_id(meta: &ParseNestedMeta) -> Result<$id, syn::Error> {
            let mut validation = $id::default();
            meta.parse_nested_meta(|meta| {
                code_and_message!(validation, meta);
                Err(meta.error("Unrecognized parameter, accepted are: code, message"))
            })?;

            Ok(validation)
        }
    };
}

macro_rules! code_and_message {
    ($validation:ident, $meta:ident) => {
        if $meta.path.is_ident("message") {
            let content = $meta.value()?;
            match content.parse::<LitStr>() {
                Ok(lit) => $validation.message = Some(lit.value()),
                Err(_) => return Err($meta.error("Message must be a string literal")),
            }
            return Ok(());
        }

        if $meta.path.is_ident("code") {
            let content = $meta.value()?;
            match content.parse::<LitStr>() {
                Ok(lit) => $validation.code = Some(lit.value()),
                Err(_) => return Err($meta.error("Code must be a string literal")),
            }
            return Ok(());
        }
    };
}

parse_pattern!(parse_email_full, Email);
parse_pattern!(parse_url_full, Url);
parse_pattern!(parse_non_control_char_full, NonControlChar);
parse_pattern!(parse_phone_full, Phone);
parse_pattern!(parse_credit_card_full, CreditCard);
parse_pattern!(parse_required_full, Required);

pub fn parse_length(meta: &ParseNestedMeta) -> Result<Length, syn::Error> {
    let mut validation = Length::default();

    meta.parse_nested_meta(|meta| {
        if meta.path.is_ident("min") {
            let content = meta.value()?;
            match content.parse::<LitInt>() {
                Ok(lit) => validation.min = Some(ValueOrPath::Value(lit.base10_parse::<u64>()?)),
                Err(_) => match content.parse::<syn::Path>() {
                    Ok(path) => validation.min = Some(ValueOrPath::Path(path)),
                    Err(_) => {
                        return Err(meta.error("Length parameter must be an int literal or path"))
                    }
                },
            }
            return Ok(());
        }

        if meta.path.is_ident("max") {
            let content = meta.value()?;
            match content.parse::<LitInt>() {
                Ok(lit) => validation.max = Some(ValueOrPath::Value(lit.base10_parse::<u64>()?)),
                Err(_) => match content.parse::<syn::Path>() {
                    Ok(path) => validation.max = Some(ValueOrPath::Path(path)),
                    Err(_) => {
                        return Err(meta.error("Length parameter must be an int literal or path"))
                    }
                },
            }
            return Ok(());
        }

        if meta.path.is_ident("equal") {
            let content = meta.value()?;
            if validation.max.is_some() || validation.min.is_some() {
                return Err(meta.error("equal parameter cannot be set if max or min exist"));
            }
            match content.parse::<LitInt>() {
                Ok(lit) => validation.equal = Some(ValueOrPath::Value(lit.base10_parse::<u64>()?)),
                Err(_) => match content.parse::<syn::Path>() {
                    Ok(path) => validation.equal = Some(ValueOrPath::Path(path)),
                    Err(_) => {
                        return Err(meta.error("Length parameter must be an int literal or path"))
                    }
                },
            }
            return Ok(());
        }

        code_and_message!(validation, meta);

        Err(meta.error("Unrecognized length parameter"))
    })?;

    Ok(validation)
}

pub fn parse_range(meta: &ParseNestedMeta) -> Result<Range, syn::Error> {
    let mut validation = Range::default();
    meta.parse_nested_meta(|meta| {
        if meta.path.is_ident("min") {
            let content = meta.value()?;
            match content.parse::<LitFloat>() {
                Ok(lit) => validation.min = Some(ValueOrPath::Value(lit.base10_parse::<f64>()?)),
                Err(_) => match content.parse::<syn::Path>() {
                    Ok(path) => validation.min = Some(ValueOrPath::Path(path)),
                    Err(_) => {
                        return Err(meta.error("Range parameter must be a float literal or path"))
                    }
                },
            }
            return Ok(());
        }

        if meta.path.is_ident("max") {
            let content = meta.value()?;
            match content.parse::<LitFloat>() {
                Ok(lit) => validation.max = Some(ValueOrPath::Value(lit.base10_parse::<f64>()?)),
                Err(_) => match content.parse::<syn::Path>() {
                    Ok(path) => validation.max = Some(ValueOrPath::Path(path)),
                    Err(_) => {
                        return Err(meta.error("Range parameter must be a float literal or path"))
                    }
                },
            }
            return Ok(());
        }

        code_and_message!(validation, meta);

        Err(meta.error("Unrecognized range parameter"))
    })?;

    Ok(validation)
}

pub fn parse_contains_full(meta: &ParseNestedMeta, not: bool) -> Result<Contains, syn::Error> {
    let mut validation = Contains::new(String::new(), not);
    meta.parse_nested_meta(|meta| {
        if meta.path.is_ident("value") {
            let content = meta.value()?;
            match content.parse::<LitStr>() {
                Ok(lit) => {
                    if lit.value().is_empty() {
                        abort!(lit.span(), "Value can not be empty")
                    }
                    validation.value = lit.value();
                }
                Err(_) => return Err(meta.error("Contains value must be an int literal or path")),
            }
            return Ok(());
        }

        code_and_message!(validation, meta);

        Err(meta.error("Unrecognized contains parameter, accepted are: value, code, message"))
    })?;

    if validation.value.is_empty() {
        abort!(meta.input.span(), "Contains validation must have a value")
    }

    Ok(validation)
}

pub fn parse_must_match_full(meta: &ParseNestedMeta) -> Result<MustMatch, syn::Error> {
    let mut validation = MustMatch {
        value: syn::Ident::new("_____NO_____", Span::call_site()),
        code: None,
        message: None,
    };
    meta.parse_nested_meta(|meta| {
        if meta.path.is_ident("value") {
            let content = meta.value()?;
            match content.parse::<syn::Ident>() {
                Ok(id) => {
                    validation.value = id;
                }
                Err(_) => {
                    return Err(meta.error(
                        "must_match value must be a field identifier for the current struct",
                    ))
                }
            }
            return Ok(());
        }

        code_and_message!(validation, meta);

        Err(meta.error("Unrecognized contains parameter, accepted are: value, code, message"))
    })?;

    if validation.value.to_string().as_str() == "_____NO_____" {
        abort!(meta.input.span(), "must_match validation must have a value")
    }

    Ok(validation)
}

pub fn parse_custom_full(meta: &ParseNestedMeta) -> Result<Custom, syn::Error> {
    let mut validation = Custom {
        path: syn::Path {
            leading_colon: None,
            segments: Punctuated::new(),
        },
        code: None,
        message: None,
    };
    meta.parse_nested_meta(|meta| {
        if meta.path.is_ident("path") {
            let content = meta.value()?;
            match content.parse::<syn::Path>() {
                Ok(path) => {
                    validation.path = path;
                }
                Err(_) => return Err(meta.error(
                    "custom value must be a path to a function that takes in the type of the field",
                )),
            }
            return Ok(());
        }

        code_and_message!(validation, meta);

        Err(meta.error("Unrecognized custom parameter, accepted are: path, code, message"))
    })?;

    if validation.path.segments.is_empty() {
        abort!(meta.input.span(), "custom validation must contain a path")
    }

    Ok(validation)
}

pub fn parse_regex_full(meta: &ParseNestedMeta) -> Result<Regex, syn::Error> {
    let mut validation = Regex {
        path: syn::Path {
            leading_colon: None,
            segments: Punctuated::new(),
        },
        code: None,
        message: None,
    };

    meta.parse_nested_meta(|meta| {
        if meta.path.is_ident("path") {
            let content = meta.value()?;
            match content.parse::<syn::Path>() {
                Ok(path) => {
                    validation.path = path;
                }
                Err(_) => return Err(meta.error(
                    "regex value must be a path to a function that takes in the type of the field",
                )),
            }
            return Ok(());
        }

        code_and_message!(validation, meta);

        Err(meta.error("Unrecognized regex parameter, accepted are: path, code, message"))
    })?;

    if validation.path.segments.is_empty() {
        abort!(meta.input.span(), "regex validation must contain a path")
    }

    Ok(validation)
}

pub fn parse_in_full(meta: &ParseNestedMeta, not: bool) -> Result<In, syn::Error> {
    let mut validation = In::new(
        syn::Path {
            leading_colon: None,
            segments: Punctuated::new(),
        },
        not,
    );
    meta.parse_nested_meta(|meta| {
        if meta.path.is_ident("value") {
            let content = meta.value()?;
            match content.parse::<syn::Path>() {
                Ok(path) => {
                    validation.path = path;
                }
                Err(_) => return Err(meta.error("[not_]in value must be a path")),
            }
            return Ok(());
        }

        code_and_message!(validation, meta);

        Err(meta.error("Unrecognized [not_]in parameter, accepted are: path, code, message"))
    })?;

    if validation.path.segments.is_empty() {
        abort!(meta.input.span(), "[not_]in validation must have a path")
    }

    Ok(validation)
}

pub fn option_to_tokens<T: quote::ToTokens>(opt: &Option<T>) -> proc_macro2::TokenStream {
    match opt {
        Some(ref t) => quote!(::std::option::Option::Some(#t)),
        None => quote!(::std::option::Option::None),
    }
}
