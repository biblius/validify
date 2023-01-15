#![recursion_limit = "128"]

use asserts::{assert_has_len, assert_has_range, assert_string_type, assert_type_matches};
use lit::*;
use proc_macro2::Span;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use quote::ToTokens;
use quoting::{quote_schema_validations, quote_validator, FieldQuoter};
use std::{collections::HashMap, unreachable};
use syn::ItemFn;
use syn::{parse_quote, spanned::Spanned};
use validation::*;
use validify_types::Validator;

mod asserts;
mod lit;
mod quoting;
mod validation;

#[proc_macro_attribute]
/// A shortcut for ergonomic error creation in custom schema validator functions.
///
/// Prepends a `let mut errors = ValidationErrors::new()` to the beginning of the function block,
/// and appends a `if errors.is_empty() { Ok(()) } else { Err(errors) }` to the end.
///
/// Designed to be used in conjuction with `field_err` and `schema_err` macros.
///
/// ```ignore
/// use validify::{validify, ValidationErrors, Validify};
///
/// #[derive(Debug, Clone)]
/// #[validify]
/// #[validate(schema(function = "schema_validation"))]
/// struct Foo {
///     a: String,
///     b: usize,
/// }
///
/// #[schema_validation]
/// fn schema_validation(foo: &Foo) -> Result<(), ValidationErrors> {
///     if foo.a == "no" {
///         field_err("a", "Can't be no", "Try again" errors);
///     }
///     if foo.b == 0 && foo.a == "no" {
///         schema_err("super no", "Done goofd", errors);
///     }
/// }
/// ```
///
/// `schema_validation` Desugars to:
///
/// ```ignore
/// fn schema_validation(foo: &Foo) -> Result<(), ValidationErrors> {
///     let mut errors = ::validify::ValidationErrors::new();
///     if foo.a == "no" {
///         errors.add(ValidationError::new_field("a", "Can't be no").with_message("Try again".to_string()));
///     }
///     if foo.b == 0 && foo.a == "no" {
///         errors.add(ValidationError::new_schema("super no", "Done goofd"));
///     }
///     if errors.is_empty() { Ok(()) } else { Err(errors) }
/// }
/// ```
pub fn schema_validation(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut func: ItemFn = syn::parse(input).unwrap();

    // Add error and return value
    let err_tokens =
        syn::parse(quote! { let mut errors = ::validify::ValidationErrors::new(); }.into())
            .unwrap();

    func.block.stmts.insert(0, err_tokens);
    let return_tokens = syn::parse(
        quote!(if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        })
        .into(),
    )
    .unwrap();

    func.block.stmts.push(return_tokens);
    func.to_token_stream().into()
}

#[proc_macro_derive(Validate, attributes(validate))]
#[proc_macro_error]
pub fn derive_validation(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_validate(&ast).into()
}

fn impl_validate(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    // Collecting the validators
    let fields_validations = collect_field_validations(ast);
    let struct_validations = find_struct_validations(&ast.attrs);
    let (validations, nested_validations) = quote_field_validations(fields_validations);

    let schema_validations = quote_schema_validations(&struct_validations);

    // Struct specific definitions
    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

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

fn collect_fields(ast: &syn::DeriveInput) -> Vec<syn::Field> {
    match ast.data {
        syn::Data::Struct(syn::DataStruct { ref fields, .. }) => {
            if fields.iter().any(|field| field.ident.is_none()) {
                abort!(
                    fields.span(),
                    "Struct has unnamed fields";
                    help = "#[derive(Validate)] can only be used on structs with named fields";
                );
            }
            fields.iter().cloned().collect::<Vec<syn::Field>>()
        }
        _ => abort!(
            ast.span(),
            "#[derive(Validate)] can only be used on structs with named fields"
        ),
    }
}

fn collect_field_validations(ast: &syn::DeriveInput) -> Vec<FieldInformation> {
    let mut fields = collect_fields(ast);

    let field_types = find_fields_type(&fields);
    fields.drain(..).fold(vec![], |mut acc, field| {
        let key = field.ident.clone().unwrap().to_string();
        let (name, validations) = find_validators_for_field(&field, &field_types);
        acc.push(FieldInformation::new(
            field,
            field_types.get(&key).unwrap().clone(),
            name,
            validations,
        ));
        acc
    })
}

fn quote_field_validations(
    mut fields: Vec<FieldInformation>,
) -> (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>) {
    let mut validations = vec![];
    let mut nested_validations = vec![];

    fields.drain(..).for_each(|x| {
        let field_ident = x.field.ident.clone().unwrap();
        let field_quoter = FieldQuoter::new(field_ident, x.name, x.field_type);

        for validation in &x.validations {
            quote_validator(
                &field_quoter,
                validation,
                &mut validations,
                &mut nested_validations,
            );
        }
    });

    (validations, nested_validations)
}

/// Find if a struct has some schema validation and returns the info if so
fn find_struct_validation(attr: &syn::Attribute) -> SchemaValidation {
    let error = |span: Span, msg: &str| -> ! {
        abort!(span, "Invalid schema level validation: {}", msg);
    };

    let Ok(syn::Meta::List(syn::MetaList { ref nested, .. })) = attr.parse_meta() else { error(attr.span(), "Unexpected struct validator") };
    let syn::NestedMeta::Meta(syn::Meta::List(syn::MetaList { ref path, ref nested, .. })) = nested[0] else { error(attr.span(), "Unexpected struct validator") };

    let ident = path.get_ident().unwrap();
    if ident != "schema" {
        error(
            attr.span(),
            "Only `schema` validation is allowed on a struct",
        )
    }

    let mut function = String::new();
    let mut code = None;
    let mut message = None;

    for arg in nested {
        let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue { ref path, ref lit, .. })) = *arg else { error(arg.span(), "Unexpected args") };

        let ident = path.get_ident().unwrap();
        match ident.to_string().as_ref() {
            "function" => {
                function = match lit_to_string(lit) {
                    Some(s) => s,
                    None => error(
                        lit.span(),
                        "Invalid argument type for `function`: only strings are allowed",
                    ),
                };
            }
            "code" => {
                code = match lit_to_string(lit) {
                    Some(s) => Some(s),
                    None => error(
                        lit.span(),
                        "Invalid argument type for `code`: only strings are allowed",
                    ),
                };
            }
            "message" => {
                message = match lit_to_string(lit) {
                    Some(s) => Some(s),
                    None => error(
                        lit.span(),
                        "Invalid argument type for `message`: only strings are allowed",
                    ),
                };
            }
            _ => error(lit.span(), "Unknown argument"),
        }
    }

    if function.is_empty() {
        error(path.span(), "`function` is required");
    }

    SchemaValidation {
        function,
        code,
        message,
    }
}

/// Finds all struct schema validations
fn find_struct_validations(struct_attrs: &[syn::Attribute]) -> Vec<SchemaValidation> {
    struct_attrs
        .iter()
        .filter(|attribute| attribute.path == parse_quote!(validate))
        .map(find_struct_validation)
        .collect()
}

/// Find the types (as string) for each field of the struct
/// Needed for the `must_match` filter
fn find_fields_type(fields: &[syn::Field]) -> HashMap<String, String> {
    let mut types = HashMap::new();

    for field in fields {
        let field_ident = field.ident.clone().unwrap().to_string();
        let field_type = match field.ty {
            syn::Type::Path(syn::TypePath { ref path, .. }) => {
                let mut tokens = proc_macro2::TokenStream::new();
                path.to_tokens(&mut tokens);
                tokens.to_string().replace(' ', "")
            }
            syn::Type::Reference(syn::TypeReference {
                ref lifetime,
                ref elem,
                ..
            }) => {
                let mut tokens = proc_macro2::TokenStream::new();
                elem.to_tokens(&mut tokens);
                let mut name = tokens.to_string().replace(' ', "");
                if lifetime.is_some() {
                    name.insert(0, '&')
                }
                name
            }
            syn::Type::Group(syn::TypeGroup { ref elem, .. }) => {
                let mut tokens = proc_macro2::TokenStream::new();
                elem.to_tokens(&mut tokens);
                tokens.to_string().replace(' ', "")
            }
            _ => {
                let mut field_type = proc_macro2::TokenStream::new();
                field.ty.to_tokens(&mut field_type);
                field_type.to_string().replace(' ', "")
            }
        };

        //println!("{:?}", field_type);
        types.insert(field_ident, field_type);
    }

    types
}

/// Find everything we need to know about a field: its real name if it's changed from the serialization
/// and the list of validators to run on it
fn find_validators_for_field(
    field: &syn::Field,
    field_types: &HashMap<String, String>,
) -> (String, Vec<FieldValidation>) {
    let rust_ident = field.ident.clone().unwrap().to_string();
    let mut field_ident = field.ident.clone().unwrap().to_string();

    let error = |span: Span, msg: &str| -> ! {
        abort!(
            span,
            "Invalid attribute #[validate] on field `{}`: {}",
            field.ident.clone().unwrap().to_string(),
            msg
        );
    };

    let field_type = field_types.get(&field_ident).unwrap();

    let mut validators = vec![];
    let mut has_validate = false;

    for attr in &field.attrs {
        if attr.path != parse_quote!(validate) && attr.path != parse_quote!(serde) {
            continue;
        }

        if attr.path == parse_quote!(validate) {
            has_validate = true;
        }

        match attr.parse_meta() {
            Ok(syn::Meta::List(syn::MetaList { ref nested, .. })) => {
                let meta_items = nested.iter().collect::<Vec<&syn::NestedMeta>>();
                // original name before serde rename
                if attr.path == parse_quote!(serde) {
                    if let Some(s) = find_original_field_name(&meta_items) {
                        field_ident = s;
                    }
                    continue;
                }

                // only validation from there on
                for meta_item in meta_items {
                    match *meta_item {
                        syn::NestedMeta::Meta(ref item) => match *item {
                            // email, url, phone, credit_card, non_control_character
                            syn::Meta::Path(ref name) => {
                                match name.get_ident().unwrap().to_string().as_ref() {
                                    "email" => {
                                        assert_string_type("email", field_type, &field.ty);
                                        validators.push(FieldValidation::new(Validator::Email));
                                    }
                                    "url" => {
                                        assert_string_type("url", field_type, &field.ty);
                                        validators.push(FieldValidation::new(Validator::Url));
                                    }
                                    "phone" => {
                                        assert_string_type("phone", field_type, &field.ty);
                                        validators.push(FieldValidation::new(Validator::Phone));
                                    }
                                    "credit_card" => {
                                        assert_string_type("credit_card", field_type, &field.ty);
                                        validators
                                            .push(FieldValidation::new(Validator::CreditCard));
                                    }
                                    "non_control_character" => {
                                        assert_string_type(
                                            "non_control_character",
                                            field_type,
                                            &field.ty,
                                        );
                                        validators.push(FieldValidation::new(
                                            Validator::NonControlCharacter,
                                        ));
                                    }
                                    "required" => {
                                        validators.push(FieldValidation::new(Validator::Required));
                                    }
                                    "required_nested" => {
                                        validators.push(FieldValidation::new(Validator::Required));
                                        validators.push(FieldValidation::new(Validator::Nested));
                                    }
                                    _ => {
                                        let mut ident = proc_macro2::TokenStream::new();
                                        name.to_tokens(&mut ident);
                                        abort!(name.span(), "Unexpected validator: {}", ident)
                                    }
                                }
                            }
                            // custom, contains, must_match, regex
                            syn::Meta::NameValue(syn::MetaNameValue {
                                ref path, ref lit, ..
                            }) => {
                                let ident = path.get_ident().unwrap();
                                match ident.to_string().as_ref() {
                                    "custom" => {
                                        match lit_to_string(lit) {
                                            Some(s) => validators.push(FieldValidation::new(Validator::Custom {
                                                function: s,
                                            })),
                                            None => error(lit.span(), "Invalid argument for `custom` validator: only strings are allowed"),
                                        };
                                    }
                                    "contains" => {
                                        match lit_to_string(lit) {
                                            Some(s) => validators.push(FieldValidation::new(Validator::Contains(s))),
                                            None => error(lit.span(), "Invalid argument for `contains` validator: only strings are allowed"),
                                        };
                                    }
                                    "does_not_contain" => {
                                        match lit_to_string(lit) {
                                            Some(s) => validators.push(FieldValidation::new(Validator::DoesNotContain(s))),
                                            None => error(lit.span(), "Invalid argument for `does_not_contain` validator: only strings are allowed"),
                                        };
                                    }
                                    "regex" => {
                                        match lit_to_string(lit) {
                                            Some(s) => validators.push(FieldValidation::new(Validator::Regex(s))),
                                            None => error(lit.span(), "Invalid argument for `regex` validator: only strings are allowed"),
                                        };
                                    }
                                    "must_match" => {
                                        match lit_to_string(lit) {
                                            Some(s) => {
                                                assert_type_matches(rust_ident.clone(), field_type, field_types.get(&s), attr);
                                                validators.push(FieldValidation::new(Validator::MustMatch(s)));
                                            }
                                            None => error(lit.span(), "Invalid argument for `must_match` validator: only strings are allowed"),
                                        };
                                    }
                                    "is_in" => {
                                        match lit_to_string(lit) {
                                            Some(s) => validators.push(FieldValidation::new(Validator::In(s))),
                                            None => error(lit.span(), "Invalid argument for `is_in` validator: only strings are allowed"),
                                        }
                                    }
                                    "not_in" => {
                                        match lit_to_string(lit) {
                                            Some(s) => validators.push(FieldValidation::new(Validator::NotIn(s))),
                                            None => error(lit.span(), "Invalid argument for `not_in` validator: only strings are allowed"),
                                        }
                                    }
                                    v => abort!(
                                        path.span(),
                                        "Unexpected name value validator: {:?}",
                                        v
                                    ),
                                };
                            }
                            // Validators with several args
                            syn::Meta::List(syn::MetaList {
                                ref path,
                                ref nested,
                                ..
                            }) => {
                                let meta_items =
                                    nested.iter().cloned().collect::<Vec<syn::NestedMeta>>();
                                let ident = path.get_ident().unwrap();
                                match ident.to_string().as_ref() {
                                    "length" => {
                                        assert_has_len(rust_ident.clone(), field_type, &field.ty);
                                        validators.push(extract_length_validation(
                                            rust_ident.clone(),
                                            attr,
                                            &meta_items,
                                        ));
                                    }
                                    "range" => {
                                        assert_has_range(rust_ident.clone(), field_type, &field.ty);
                                        validators.push(extract_range_validation(
                                            rust_ident.clone(),
                                            attr,
                                            &meta_items,
                                        ));
                                    }
                                    "custom" => {
                                        validators.push(extract_custom_validation(
                                            rust_ident.clone(),
                                            attr,
                                            &meta_items,
                                        ));
                                    }
                                    "email"
                                    | "url"
                                    | "phone"
                                    | "credit_card"
                                    | "non_control_character"
                                    | "required" => {
                                        validators.push(extract_argless_validation(
                                            ident.to_string(),
                                            rust_ident.clone(),
                                            &meta_items,
                                        ));
                                    }
                                    "contains" | "does_not_contain" => {
                                        validators.push(extract_one_arg_validation(
                                            "pattern",
                                            ident.to_string(),
                                            rust_ident.clone(),
                                            &meta_items,
                                        ));
                                    }
                                    "regex" => {
                                        validators.push(extract_one_arg_validation(
                                            "path",
                                            ident.to_string(),
                                            rust_ident.clone(),
                                            &meta_items,
                                        ));
                                    }
                                    "must_match" => {
                                        let validation = extract_one_arg_validation(
                                            "other",
                                            ident.to_string(),
                                            rust_ident.clone(),
                                            &meta_items,
                                        );
                                        if let Validator::MustMatch(ref t2) = validation.validator {
                                            assert_type_matches(
                                                rust_ident.clone(),
                                                field_type,
                                                field_types.get(t2),
                                                attr,
                                            );
                                        }
                                        validators.push(validation);
                                    }
                                    "is_in" | "not_in" => {
                                        validators.push(extract_one_arg_validation(
                                            "other",
                                            ident.to_string(),
                                            rust_ident.clone(),
                                            &meta_items,
                                        ));
                                    }
                                    v => abort!(path.span(), "Unexpected list validator: {:?}", v),
                                }
                            }
                        },
                        _ => unreachable!("Found a non Meta while looking for validations"),
                    };
                }
            }
            Ok(syn::Meta::Path(_)) => validators.push(FieldValidation::new(Validator::Nested)),
            Ok(syn::Meta::NameValue(_)) => abort!(attr.span(), "Unexpected name=value argument"),
            Err(e) => {
                let error_string = format!("{:?}", e);
                if error_string == "Error(\"Expected literal\")" {
                    abort!(
                        attr.span(),
                        "Invalid attributes for field `{}`",
                        field_ident
                    );
                } else {
                    abort!(
                        attr.span(),
                        "Unable to parse attribute for field `{}` with error: {:?}",
                        field_ident,
                        e
                    );
                }
            }
        }

        if has_validate && validators.is_empty() {
            error(attr.span(), "Needs at least one validator");
        }
    }

    (field_ident, validators)
}

/// Serde can be used to rename fields on deserialization but most of the times
/// we want the error on the original field.
///
/// For example a JS frontend might send camelCase fields and Rust converts them to snake_case
/// but we want to send the errors back with the original name
fn find_original_field_name(meta_items: &[&syn::NestedMeta]) -> Option<String> {
    let mut original_name = None;

    for meta_item in meta_items {
        match **meta_item {
            syn::NestedMeta::Meta(ref item) => match *item {
                syn::Meta::Path(_) => continue,
                syn::Meta::NameValue(syn::MetaNameValue {
                    ref path, ref lit, ..
                }) => {
                    let ident = path.get_ident().unwrap();
                    if ident == "rename" {
                        original_name = Some(lit_to_string(lit).unwrap());
                    }
                }
                syn::Meta::List(syn::MetaList { ref nested, .. }) => {
                    return find_original_field_name(
                        &nested.iter().collect::<Vec<&syn::NestedMeta>>(),
                    );
                }
            },
            _ => unreachable!(),
        };

        if original_name.is_some() {
            return original_name;
        }
    }

    original_name
}
