use super::parsing::*;
use super::quoting::quote_field_validations;
use super::quoting::quote_struct_validations;
use crate::asserts::{
    assert_has_len, assert_has_range, is_full_pattern, is_single_lit, is_single_path,
};
use crate::fields::collect_field_info;
use crate::types::{
    Contains, CreditCard, Custom, Email, In, Ip, MustMatch, NonControlChar, Phone, Regex, Required,
    SchemaValidation, Url, Validator,
};
use proc_macro_error::abort;
use quote::quote;
use syn::parenthesized;
use syn::spanned::Spanned;

const EMAIL: &str = "email";
const URL: &str = "url";
const LENGTH: &str = "length";
const RANGE: &str = "range";
const MUST_MATCH: &str = "must_match";
const CONTAINS: &str = "contains";
const CONTAINS_NOT: &str = "contains_not";
const NON_CONTROL_CHAR: &str = "non_control_char";
const CUSTOM: &str = "custom";
const REGEX: &str = "regex";
const CREDIT_CARD: &str = "credit_card";
const PHONE: &str = "phone";
const REQUIRED: &str = "required";
const IS_IN: &str = "is_in";
const NOT_IN: &str = "not_in";
const IP: &str = "ip";
const VALIDATE: &str = "validate";

pub fn impl_validate(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let field_validations = collect_field_info(input, true).unwrap();
    let struct_validations = collect_struct_validation(&input.attrs).unwrap();

    let (validations, nested_validations) = quote_field_validations(field_validations);
    let schema_validations = quote_struct_validations(&struct_validations);

    // Struct specific definitions
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // The Validate trait implementation
    quote!(
        impl #impl_generics ::validify::Validate for #ident #ty_generics #where_clause {
            fn validate(&self) -> ::std::result::Result<(), ::validify::ValidationErrors> {
                let mut errors = ::validify::ValidationErrors::new();

                #(#validations)*

                #(#schema_validations)*

                #(#nested_validations)*

                if errors.is_empty() {
                    ::std::result::Result::Ok(())
                } else {
                    ::std::result::Result::Err(errors)
                }
            }
        }
    )
}

/// Find if a struct has some schema validation and returns the info if so
fn collect_struct_validation(
    attrs: &[syn::Attribute],
) -> Result<Vec<SchemaValidation>, syn::Error> {
    let mut validations = vec![];
    let filtered = attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("validate"));

    for attr in filtered {
        attr.parse_nested_meta(|meta| {
            validations.push(SchemaValidation {
                function: meta.path,
            });
            Ok(())
        })?;
    }
    Ok(validations)
}

pub fn collect_validations(validators: &mut Vec<Validator>, field: &syn::Field, field_type: &str) {
    let field_ident = field.ident.as_ref().unwrap().to_string();
    for attr in field.attrs.iter() {
        if !attr.path().is_ident("validate") {
            continue;
        }

        let syn::Meta::List(ref list) = attr.meta else {
            let syn::Meta::Path(ref path) = attr.meta else {
                abort!(
                    attr.meta.span(),
                    "Validate must be applied as a list, i.e. `validate(/*...*/)` or as a path `validate` for nested validation"
                )
            };
            if path.is_ident(VALIDATE) {
                validators.push(Validator::Nested)
            }
            continue;
        };

        list.parse_nested_meta(|meta| {
            if meta.path.is_ident(EMAIL) {
                if is_full_pattern(&meta) {
                    let validation = parse_email_full(&meta)?;
                    validators.push(Validator::Email(validation));
                } else {
                    validators.push(Validator::Email(Email::default()));
                }
                return Ok(());
            }

            if meta.path.is_ident(URL) {
                if is_full_pattern(&meta) {
                    let validation = parse_url_full(&meta)?;
                    validators.push(Validator::Url(validation));
                } else {
                    validators.push(Validator::Url(Url::default()));
                }

                return Ok(());
            }

            if meta.path.is_ident(LENGTH) {
                assert_has_len(field_ident.clone(), field_type, &field.ty);
                let validation = parse_length(&meta)?;
                validators.push(Validator::Length(validation));
                return Ok(());
            }

            if meta.path.is_ident(RANGE) {
                assert_has_range(field_ident.clone(), field_type, &field.ty);
                let validation = parse_range(&meta)?;
                validators.push(Validator::Range(validation));
                return Ok(());
            }

            if meta.path.is_ident(MUST_MATCH) {
                //assert_type_matches(rust_ident.clone(), field_type, field_types.get(&s), attr);
                if is_single_path(&meta, "must_match") {
                    let content;
                    parenthesized!(content in meta.input);
                    let Ok(id) = content.parse::<syn::Ident>() else {
                        return Err(meta.error("Invalid value given for `must_match` validation, must be a field on the current struct"))
                    };
                    validators.push(Validator::MustMatch(MustMatch::new(id)));
                } else {
                    let validation = parse_must_match_full(&meta)?;
                    validators.push(Validator::MustMatch(validation));
                }
                return Ok(());
            }

            if meta.path.is_ident(CONTAINS) {
                let is_single = is_single_lit(&meta, "contains");
                if is_single {
                    let content;
                    parenthesized!(content in meta.input);
                    let Ok(lit) = content.parse::<syn::LitStr>() else {
                        return Err(meta.error("Invalid value given for `contains` validation, must be a string literal"))
                    };
                    validators.push(Validator::Contains(Contains::new(lit.value(), false)));
                } else {
                    let validation = parse_contains_full(&meta, false)?;
                    validators.push(Validator::Contains(validation));
                }
                return Ok(());
            }

            if meta.path.is_ident(CONTAINS_NOT) {
                if is_single_lit(&meta, "contains_not") {
                    let content;
                    parenthesized!(content in meta.input);
                    let Ok(lit ) = content.parse::<syn::LitStr>() else {
                        return Err(meta.error("Invalid value given for `contains_not` validation, must be a string literal"))
                    };
                    validators.push(Validator::Contains(Contains::new(lit.value(), true)));
                } else {
                    let validation = parse_contains_full(&meta, true)?;
                    validators.push(Validator::Contains(validation));
                }
                return Ok(());
            }

            if meta.path.is_ident(NON_CONTROL_CHAR) {
                if is_full_pattern(&meta) {
                    let validation = parse_non_control_char_full(&meta)?;
                    validators.push(Validator::NonControlCharacter(validation))
                } else {
                    validators.push(Validator::NonControlCharacter(NonControlChar::default()));
                }
                return Ok(());
            }

            if meta.path.is_ident(CUSTOM) {
                if is_single_path(&meta, "custom") {
                    let content;
                    parenthesized!(content in meta.input);
                    let Ok(function) = content.parse::<syn::Path>() else {
                        return Err(meta.error("Invalid value given for `custom` validation"))
                    };
                    validators.push(Validator::Custom(Custom::new(function)));
                } else {
                    let validation = parse_custom_full(&meta)?;
                    validators.push(Validator::Custom(validation));
                }
                return Ok(());
            }

            if meta.path.is_ident(REGEX) {
                if is_single_path(&meta, "regex") {
                    let content;
                    parenthesized!(content in meta.input);
                    let Ok(path) = content.parse::<syn::Path>() else {
                        return Err(meta.error("Invalid value given for `regex` validation, must be a path"))
                    };
                    validators.push(Validator::Regex(Regex::new(path)));
                } else {
                    let validation = parse_regex_full(&meta)?;
                    validators.push(Validator::Regex(validation));
                }
                return Ok(());
            }

            if meta.path.is_ident(CREDIT_CARD) {
                if is_full_pattern(&meta) {
                    let validation = parse_credit_card_full(&meta)?;
                    validators.push(Validator::CreditCard(validation));
                } else {
                    validators.push(Validator::CreditCard(CreditCard::default()));
                }
                return Ok(());
            }

            if meta.path.is_ident(PHONE) {
                if is_full_pattern(&meta) {
                    let validation = parse_phone_full(&meta)?;
                    validators.push(Validator::Phone(validation));
                } else {
                    validators.push(Validator::Phone(Phone::default()));
                }
                return Ok(());
            }

            if meta.path.is_ident(REQUIRED) {
                if is_full_pattern(&meta) {
                    let validation = parse_required_full(&meta)?;
                    validators.push(Validator::Required(validation));
                } else {
                    validators.push(Validator::Required(Required::default()));
                }
                return Ok(());
            }

            if meta.path.is_ident(IS_IN) {
                if is_single_path(&meta, "in") {
                    let content;
                    parenthesized!(content in meta.input);
                    let Ok(path) = content.parse::<syn::Path>() else {
                        return Err(meta.error("Invalid value provided"))
                    };
                    validators.push(Validator::In(In::new(path, false)));
                } else {
                    let validation = parse_in_full(&meta, false)?;
                    validators.push(Validator::In(validation));
                }
                return Ok(());
            }

            if meta.path.is_ident(NOT_IN) {
                if is_single_path(&meta, "in") {
                    let content;
                    parenthesized!(content in meta.input);
                    let Ok(path) = content.parse::<syn::Path>() else {
                        return Err(meta.error("Invalid value provided"))
                    };
                    validators.push(Validator::In(In::new(path, true)));
                } else {
                    let validation = parse_in_full(&meta, true)?;
                    validators.push(Validator::In(validation));
                }
                return Ok(());
            }

            if meta.path.is_ident(IP) {
                if is_full_pattern(&meta) {
                    let validation = parse_ip_full(&meta)?;
                    validators.push(Validator::Ip(validation));
                } else {
                    validators.push(Validator::Ip(Ip::default()));
                }

                return Ok(());
            }



            Err(meta.error("Uncrecognized validate parameter")
            )
        }).unwrap_or_else(|e| abort!(e.span(), e));
    }
}
