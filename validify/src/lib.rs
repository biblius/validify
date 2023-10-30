#![doc = include_str!(concat!("../", std::env!("CARGO_PKG_README")))]

mod error;
mod traits;
mod validation;

pub use error::{ValidationError, ValidationErrors};
pub use validation::time;
pub use validation::{
    cards::validate_credit_card,
    contains::validate_contains,
    email::validate_email,
    ip::{validate_ip, validate_ip_v4, validate_ip_v6},
    length::validate_length,
    must_match::validate_must_match,
    non_control_char::validate_non_control_character,
    phone::validate_phone,
    r#in::validate_in,
    range::validate_range,
    required::validate_required,
    urls::validate_url,
};
pub use validify_derive::{schema_validation, Validate, Validify};

/// Deriving [Validate] will allow you to specify struct validations, but does not create an associated
/// payload struct. Validate can be derived on structs containing references, while Validify cannot due
/// to modifiers.
pub trait Validate {
    fn validate(&self) -> Result<(), ValidationErrors>;
}

impl<T: Validate> Validate for &T {
    fn validate(&self) -> Result<(), ValidationErrors> {
        T::validate(*self)
    }
}

/// Modifies the struct based on the provided `modify` parameters. Automatically implemented when deriving Validify.
pub trait Modify {
    /// Apply the provided modifiers to self
    fn modify(&mut self);
}

/// Deriving [Validify] allows you to modify structs before they are validated by providing a out of the box validation implementations
/// as well as the ability to write custom ones. It also generates a payload struct for the deriving struct,
/// which can be used when deserialising web payloads. The payload struct is just a copy of the original, except will all the fields being
/// `Option`s. This enables the payload to be fully deserialized (given that all existing fields are of the correct type) before being validated
/// to allow for better validation errors.
///
/// ### Example
///
/// ```
/// use validify::Validify;
///
/// #[derive(Debug, Clone, serde::Deserialize, Validify)]
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
/// #[derive(Debug, Clone, serde::Deserialize, Validify)]
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
pub trait Validify: Modify + Validate + Sized + From<Self::Payload> {
    /// Represents the same structure of the implementing struct,
    /// with all its fields as options. Used to represent a completely
    /// deserializable version of the struct even if the fields are missing.
    /// Used for more detailed descriptions of what fields are missing, along
    /// with any other validation errors.
    ///
    /// This type is automatically implemented when deriving validify by creating
    /// an accompanying payload struct:
    ///
    /// ```
    /// #[derive(Debug, Clone, serde::Deserialize, validify::Validify)]
    /// struct Data {
    ///     a: String,
    ///     b: Option<String>
    /// }
    /// ```
    ///
    /// Expands to:
    ///
    /// ```ignore
    /// #[derive(Debug, Validate, serde::Deserialize)]
    /// struct DataPayload {
    ///     #[validate(required)]
    ///     a: Option<String>,
    ///     b: Option<String>
    /// }
    ///
    /// /*
    ///  * serde impls and other stuff
    ///  */
    ///
    /// impl Validify for Data {
    ///     type Payload = DataPayload;
    ///
    ///     /* fn validify(payload: Self::Payload) { ... } */
    /// }
    ///
    /// ```
    type Payload: serde::de::DeserializeOwned + Validate;

    /// Apply the provided modifiers to the payload and run validations, returning the original
    /// struct if all the validations pass.
    fn validify(payload: Self::Payload) -> Result<Self, ValidationErrors>;

    /// Apply the provided modifiers to self and run validations.
    fn validify_self(&mut self) -> Result<(), ValidationErrors> {
        self.modify();
        self.validate()
    }
}

#[macro_export]
/// Designed to be used with the [schema_validation] proc macro. Used for ergonomic custom error handling.
///
/// Adds a schema validaton error to the generated `ValidationErrors`.
///
/// The errors argument should pass in an instance of `ValidationErrors`,
/// and usually is used with the one generated from `schema_validation`.
///
/// Accepts:
///
/// `("code", errors)`
/// `("code", "custom message", errors)`
macro_rules! schema_err {
    ($code:literal, $errors:expr) => {
        $errors.add(::validify::ValidationError::new_schema($code));
    };
    ($code:literal, $message:literal, $errors:expr) => {
        $errors
            .add(::validify::ValidationError::new_schema($code).with_message($message.to_string()));
    };
    ($code:literal, $message:expr, $errors:expr) => {
        $errors.add(::validify::ValidationError::new_schema($code).with_message($message));
    };
}

#[macro_export]
/// Designed to be used with the [schema_validation] proc macro. Used for ergonomic custom error handling.
///
/// Adds a field validaton error to the generated `ValidationErrors`
///
///  The errors argument should pass in an instance of `ValidationErrors`,
/// and usually is used with the one generated from `schema_validation`.
///
/// Accepts:
///
/// `("field_name", "code", errors)`
/// `("field_name", "code", "custom message", errors)`
macro_rules! field_err {
    ($field:literal, $code:literal, $errors:expr) => {
        $errors.add(::validify::ValidationError::new_field($field, $code));
    };
    ($field:literal, $code:literal, $message:literal, $errors:expr) => {
        $errors.add(
            ::validify::ValidationError::new_field($field, $code)
                .with_message($message.to_string()),
        );
    };
    ($field:literal, $code:literal, $message:expr, $errors:expr) => {
        $errors.add(::validify::ValidationError::new_field($field, $code).with_message($message));
    };
}
