use crate::types::{
    Contains, CreditCard, Custom, Email, In, Ip, Length, MustMatch, NonControlChar, Phone, Range,
    Regex, Required, Time, TimeMultiplier, TimeOp, Url, ValueOrPath,
};
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::quote;
use syn::{meta::ParseNestedMeta, punctuated::Punctuated, LitBool, LitFloat, LitInt, LitStr};

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
    let mut validation = Contains {
        not,
        ..Default::default()
    };

    meta.parse_nested_meta(|meta| {
        if meta.path.is_ident("value") {
            let content = meta.value()?;
            match content.parse::<syn::Lit>() {
                Ok(lit) => {
                    validation.value = Some(ValueOrPath::Value(lit));
                }
                Err(_) => match content.parse::<syn::Path>() {
                    Ok(path) => validation.value = Some(ValueOrPath::Path(path)),
                    Err(_) => {
                        return Err(meta.error("Contains parameter must be a literal or path"))
                    }
                },
            }
            return Ok(());
        }

        code_and_message!(validation, meta);

        Err(meta.error("Unrecognized contains parameter, accepted are: value, code, message"))
    })?;

    if validation.value.is_none() {
        abort!(meta.input.span(), "Contains validation must have a value")
    }

    Ok(validation)
}

pub fn parse_must_match_full(meta: &ParseNestedMeta) -> Result<MustMatch, syn::Error> {
    let mut validation = MustMatch {
        value: syn::Ident::new("BAD_____NO_____BAD", Span::call_site()),
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

    if validation.value.to_string().as_str() == "BAD_____NO_____BAD" {
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
        if meta.path.is_ident("function") {
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
        if meta.path.is_ident("collection") {
            let content = meta.value()?;
            match content.parse::<syn::Path>() {
                Ok(path) => {
                    validation.path = path;
                }
                Err(_) => return Err(meta.error("[not_]in collection must be a valid path")),
            }
            return Ok(());
        }

        code_and_message!(validation, meta);

        Err(meta.error("Unrecognized [not_]in parameter, accepted are: collection, code, message"))
    })?;

    if validation.path.segments.is_empty() {
        abort!(meta.input.span(), "[not_]in validation must have a path")
    }

    Ok(validation)
}

pub fn parse_ip_full(meta: &ParseNestedMeta) -> Result<Ip, syn::Error> {
    let mut validation = Ip::default();

    meta.parse_nested_meta(|meta| {
        if meta.path.is_ident("format") {
            let content = meta.value()?;
            match content.parse::<syn::LitStr>() {
                Ok(format) => match format.value().as_str() {
                    "v4" => validation.format = Some(crate::types::IpFormat::V4),
                    "v6" => validation.format = Some(crate::types::IpFormat::V6),
                    _ => abort!(format.span(), "Invalid IP format, accepted are: v4, v6"),
                },
                Err(_) => {
                    return Err(meta.error("ip format must be a string literal: \"v4\"/\"v6\""))
                }
            }
            return Ok(());
        }

        code_and_message!(validation, meta);

        Err(meta.error("Unrecognized ip parameter, accepted are: format, code, message"))
    })?;

    Ok(validation)
}

pub fn parse_time(meta: &ParseNestedMeta) -> Result<Time, syn::Error> {
    const INTERVALS: [&str; 5] = ["seconds", "minutes", "hours", "days", "weeks"];

    let mut validation = Time::default();

    meta.parse_nested_meta(|meta| {
        if meta.path.is_ident("op") {
            let content = meta.value()?;
            match content.parse::<syn::Ident>() {
                Ok(id) => {
                    validation.op = id.to_string().into();
                    if matches!(validation.op, TimeOp::None) {
                        return Err(meta.error(
                            "op must be a path corresponding to one of the time validation functions"
                        ))
                    }
                },
                Err(_) => return Err(meta.error(
                    "op must be a path corresponding to one of the time validation functions"
                )),
            }
            return Ok(());
        }

        if meta.path.is_ident("target") {
            let content = meta.value()?;
            match content.parse::<LitStr>() {
                Ok(date) => {
                    if date.value().is_empty() {
                        return Err(meta.error("target cannot be empty"));
                    }
                    validation.target = Some(ValueOrPath::Value(date.value()))
                },
                Err(_) => match content.parse::<syn::Path>() {
                    Ok(path) => {
                        validation.target = Some(ValueOrPath::Path(path))
                    },
                    Err(e) => {
                        return Err(
                            meta.error(format!("target must be a path or a string literal: {e}"))
                        )
                    }
                },
            }
            return Ok(());
        }

        if meta.path.is_ident("format") {
            let content = meta.value()?;
            match content.parse::<LitStr>() {
                Ok(format) => {
                    if format.value().is_empty() {
                        return Err(meta.error("format cannot be empty"));
                    }
                    validation.format = Some(format.value())
                },
                Err(_) => return Err(meta.error("format must be a string literal"))
            }
            return Ok(());
        }

        if meta.path.is_ident("inclusive") {
            let content = meta.value()?;
            match content.parse::<LitBool>() {
                Ok(inclusive) => validation.inclusive = inclusive.value(),
                Err(_) => return Err(meta.error("inclusive must be a bool literal"))
            }
            return Ok(());
        }

        for (i, interval) in INTERVALS.iter().enumerate() {
            if meta.path.is_ident(interval) {
                let content = meta.value()?;
                match content.parse::<syn::LitInt>() {
                    Ok(amount) => {
                        if validation.duration.is_some() {
                            return Err(meta.error("Interval already set"))
                        }
                        let amount = amount.base10_parse()?;
                        if amount == 0 {
                            return Err(meta.error("Interval cannot be 0"))
                        }
                        match INTERVALS[i] {
                          "seconds" => {
                            validation.path_type = TimeMultiplier::Seconds;
                            validation.duration = Some(ValueOrPath::Value(chrono::Duration::seconds(amount).num_seconds()))
                          },
                          "minutes" => {
                            validation.path_type = TimeMultiplier::Minutes;
                            validation.duration = Some(ValueOrPath::Value(chrono::Duration::minutes(amount).num_seconds()))
                          },
                          "hours" => {
                            validation.path_type = TimeMultiplier::Hours;
                            validation.duration = Some(ValueOrPath::Value(chrono::Duration::hours(amount).num_seconds()))
                          },
                          "days" => {
                            validation.path_type = TimeMultiplier::Days;
                            validation.duration = Some(ValueOrPath::Value(chrono::Duration::days(amount).num_seconds()))
                          },
                          "weeks" => {
                            validation.path_type = TimeMultiplier::Weeks;
                            validation.duration = Some(ValueOrPath::Value(chrono::Duration::weeks(amount).num_seconds()))
                          },
                          _=> unreachable!()
                        }
                    },
                    Err(_) => {
                        match content.parse::<syn::Path>() {
                            Ok(path) => {
                                validation.duration = Some(ValueOrPath::Path(path));
                                match INTERVALS[i] {
                                    "seconds" => {
                                      validation.path_type = TimeMultiplier::Seconds;
                                    },
                                    "minutes" => {
                                      validation.path_type = TimeMultiplier::Minutes;
                                    },
                                    "hours" => {
                                      validation.path_type = TimeMultiplier::Hours;
                                    },
                                    "days" => {
                                      validation.path_type = TimeMultiplier::Days;
                                    },
                                    "weeks" => {
                                      validation.path_type = TimeMultiplier::Weeks;
                                    },
                                    _=> unreachable!()
                                  }
                            },
                            Err(_) => {
                                return Err(meta.error(format!("interval must be one of the following: {INTERVALS:?} and must be an int literal or path")))
                            },
                        }
                    },
                }
                return Ok(());
            }
        }

        code_and_message!(validation, meta);

        Err(meta.error("Unrecognized time parameter"))
    })?;

    validation.assert(meta)?;

    Ok(validation)
}

pub fn option_to_tokens<T: quote::ToTokens>(opt: &Option<T>) -> proc_macro2::TokenStream {
    match opt {
        Some(ref t) => quote!(::std::option::Option::Some(#t)),
        None => quote!(::std::option::Option::None),
    }
}
