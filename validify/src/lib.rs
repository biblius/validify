//! A procedural macro built on top of the [validator](https://docs.rs/validator/latest/validator/)
//! crate that provides attributes for field modifiers. Particularly useful in the context of web payloads.
//!
//! Visit the [repository](https://github.com/biblius/validify) to see exactly how it works.
//!
//!  ### Example
//!
//! ```
//! use validify::{validify, Validify};
//!
//! #[validify]
//! #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
//! struct Testor {
//!     #[modify(lowercase, trim)]
//!     #[validate(length(equal = 8))]
//!     pub a: String,
//!     #[modify(trim, uppercase)]
//!     pub b: Option<String>,
//!     #[modify(custom = "do_something")]
//!     pub c: String,
//!     #[modify(custom = "do_something")]
//!     pub d: Option<String>,
//!     #[validify]
//!     pub nested: Nestor,
//! }
//!
//! #[validify]
//! #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
//! struct Nestor {
//!     #[modify(trim, uppercase)]
//!     #[validate(length(equal = 12))]
//!     a: String,
//!     #[modify(capitalize)]
//!     #[validate(length(equal = 14))]
//!     b: String,
//! }
//!
//! fn do_something(input: &mut String) {
//!     *input = String::from("modified");
//! }
//!
//! let mut test = Testor {
//!   a: "   LOWER ME     ".to_string(),
//!   b: Some("  makemeshout   ".to_string()),
//!   c: "I'll never be the same".to_string(),
//!   d: Some("Me neither".to_string()),
//!   nested: Nestor {
//!     a: "   notsotinynow   ".to_string(),
//!       b: "capitalize me.".to_string(),
//!   },
//! };
//!
//! // The magic line
//! let res = Testor::validify(test.into());
//!
//! assert!(matches!(res, Ok(_)));
//!
//! let test = res.unwrap();
//!
//! // Parent
//! assert_eq!(test.a, "lower me");
//! assert_eq!(test.b, Some("MAKEMESHOUT".to_string()));
//! assert_eq!(test.c, "modified");
//! assert_eq!(test.d, Some("modified".to_string()));
//! // Nested
//! assert_eq!(test.nested.a, "NOTSOTINYNOW");
//! assert_eq!(test.nested.b, "Capitalize me.");
//! ```

mod error;
mod traits;
mod validation;

pub use validation::cards::validate_credit_card;
pub use validation::contains::validate_contains;
pub use validation::does_not_contain::validate_does_not_contain;
pub use validation::email::validate_email;
pub use validation::ip::{validate_ip, validate_ip_v4, validate_ip_v6};
pub use validation::length::validate_length;
pub use validation::must_match::validate_must_match;
pub use validation::non_control_character::validate_non_control_character;
pub use validation::phone::validate_phone;
pub use validation::range::validate_range;
pub use validation::required::validate_required;
pub use validation::urls::validate_url;

pub use derive_validator::Validate;
pub use derive_validify::{validify, Validify};
pub use error::{ValidationError, ValidationErrors};

use serde::de::DeserializeOwned;

/// Validates structs based on the provided `validate` parameters. Can be implemented on its own if one doesn't need payload modifications.
pub trait Validate {
    fn validate(&self) -> Result<(), ValidationErrors>;
}

impl<T: Validate> Validate for &T {
    fn validate(&self) -> Result<(), ValidationErrors> {
        T::validate(*self)
    }
}

/// Modifies struct based on the provided `modify` parameters. Automatically implemented when deriving Validify.
pub trait Modify {
    /// Apply the provided modifiers to self
    fn modify(&mut self);
}

/// Combines `Validate` and `Modify` in one trait and provides the intermediary payload struct. This trait should never be implemented manually,
/// and should be derived with the `#[validify]` macro which automatically implements `Validate`, `Modify` and `Validify`.
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
    /// ```ignore
    /// #[validify]
    /// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    /// struct Data {
    ///     a: String
    /// }
    ///
    /// // expands to
    /// struct DataPayload {
    ///     a: Option<String>
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
    type Payload: DeserializeOwned + Validate;

    /// Apply the provided modifiers to self and run validations.
    fn validify(payload: Self::Payload) -> Result<Self, ValidationErrors> {
        // Since the payload is all options, this will
        // only check if there are missing required fields
        <Self::Payload as Validate>::validate(&payload)?;
        let mut this = Self::from(payload);
        <Self as Modify>::modify(&mut this);
        <Self as Validate>::validate(&this)?;
        Ok(this)
    }
}
