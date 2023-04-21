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

/// Combines `Validate` and `Modify` in one trait and provides the intermediary payload struct.
///
/// Deriving this will allow you to annotate fields with the `modify` attribute. Modifiers are simple functions that modify
/// the struct before validation. You can use the few out of the box ones or create your own.
///
/// Structs deriving this trait will also get an associated `Payload` struct
/// which is just a copy of the original, except with all the fields as
/// `Option`s. The payload struct derives `Validate` and is named the same as
/// the original suffixed with `...Payload`.
///
/// Fields in the original struct that are not options will be annotated
/// with the `#[required]` flag and will be validated before the original struct.
/// This enables the payload to be fully deserialized before being validated and is necessary for better validation errors,
/// as deserialization errors are generally not that descriptive.
///
/// Validify's `validify` method takes in as its argument the generated payload as its primary focus is
/// toward web payloads. The payload is meant to be used in handlers and after being validated transformed back
/// to the original for further processing.
/// The `validify` method returns the original struct upon successfull validation.
///
/// Visit the [repository](https://github.com/biblius/validify) to see the list of available validations and
/// modifiers as well as more examples.
///
///  ### Example
///
/// ```ignore
/// use validify::Validify;
///
/// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Validify)]
/// struct Testor {
///     #[modify(lowercase, trim)]
///     #[validate(length(equal = 8))]
///     pub a: String,
///     #[modify(trim, uppercase)]
///     pub b: Option<String>,
///     #[modify(custom(do_something))]
///     pub c: String,
///     #[modify(custom(do_something))]
///     pub d: Option<String>,
///     #[validify]
///     pub nested: Nestor,
/// }
///
/// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Validify)]
/// struct Nestor {
///     #[modify(trim, uppercase)]
///     #[validate(length(equal = 12))]
///     a: String,
///     #[modify(capitalize)]
///     #[validate(length(equal = 14))]
///     b: String,
/// }
///
/// fn do_something(input: &mut String) {
///     *input = String::from("modified");
/// }
///
/// let mut test = Testor {
///   a: "   LOWER ME     ".to_string(),
///   b: Some("  makemeshout   ".to_string()),
///   c: "I'll never be the same".to_string(),
///   d: Some("Me neither".to_string()),
///   nested: Nestor {
///     a: "   notsotinynow   ".to_string(),
///       b: "capitalize me.".to_string(),
///   },
/// };
///
/// // The magic line
/// let res = Testor::validify(test.into());
///
/// assert!(matches!(res, Ok(_)));
///
/// let test = res.unwrap();
///
/// // Parent
/// assert_eq!(test.a, "lower me");
/// assert_eq!(test.b, Some("MAKEMESHOUT".to_string()));
/// assert_eq!(test.c, "modified");
/// assert_eq!(test.d, Some("modified".to_string()));
/// // Nested
/// assert_eq!(test.nested.a, "NOTSOTINYNOW");
/// assert_eq!(test.nested.b, "Capitalize me.");
/// ```
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
#[proc_macro_attribute]
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
