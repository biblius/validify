use proc_macro_error::proc_macro_error;
use quote::{quote, ToTokens};
use syn::ItemFn;
use types::*;

mod asserts;
mod fields;
mod quoter;
mod types;
mod validate;
mod validify;

#[proc_macro_derive(Validify, attributes(modify, validate, validify))]
#[proc_macro_error]
pub fn derive_validify(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    validify::r#impl::impl_validify(&ast).into()
}

/// Derives `Validate` based on the provided field attributes.
///
/// ### Example
///
/// ```ignore
/// use validify::{ValidationError, ValidationErrors, Validate};
///
/// #[derive(Debug, Validate, Deserialize)]
/// #[validate(validate_signup)]
/// struct SignupData {
///     #[validate(email)]
///     mail: String,
///     #[validate(url)]
///     site: String,
///     #[validate(
///         length(min = 1),
///         custom(validate_unique_username)
///     )]
///     first_name: String,
///     #[validate(range(min = 18., max = 20.))]
///     age: u32,
/// }
///
/// fn validate_unique_username(username: &str) -> Result<(), ValidationError> {
///     if username == "xXxShad0wxXx" {
///         return Err(ValidationError::new_field(
///             "first_name",
///             "terrible_username",
///         ));
///     }
///
///     Ok(())
/// }
///
/// fn validate_signup(data: &SignupData) -> Result<(), ValidationErrors> {
///     let mut errs = ValidationErrors::new();
///     if data.mail.ends_with("gmail.com") && data.age == 18 {
///         errs.add(ValidationError::new_schema("stupid_rule"));
///     }
///     if errs.is_empty() {
///         return Ok(());
///     }
///     Err(errs)
/// }
///
/// let signup = SignupData {
///     mail: "bob@bob.com".to_string(),
///     site: "http://hello.com".to_string(),
///     first_name: "Bob".to_string(),
///     age: 18,
/// };
///     
/// assert!(signup.validate().is_ok());
/// ```
#[proc_macro_derive(Validate, attributes(validate))]
#[proc_macro_error]
pub fn derive_validate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse(input).unwrap();
    validate::r#impl::impl_validate(&input).into()
}

#[proc_macro_attribute]
/// A shortcut for ergonomic error creation in custom schema validator functions.
///
/// Prepends a `let mut errors = ValidationErrors::new()` to the beginning of the function block,
/// and appends a `if errors.is_empty() { Ok(()) } else { Err(errors) }` to the end.
///
/// Designed to be used in conjuction with the `field_err` and `schema_err` macros.
///
/// ```ignore
/// use validify::{ValidationErrors, Validify, schema_validation};
///
/// #[derive(Debug, Clone, Validify)]
/// #[validate(schema_validation)]
/// struct Foo {
///     a: String,
///     b: usize,
/// }
///
/// #[schema_validation]
/// fn schema_validation(foo: &Foo) -> Result<(), ValidationErrors> {
///     if foo.a == "no" {
///         field_err("a", "Can't be no", "Try again", errors);
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
///         errors.add(ValidationError::new_field("a", "Can't be no").with_message("Try again".to_string()));;
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
