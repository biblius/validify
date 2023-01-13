use serde::de::DeserializeOwned;
use validator::{Validate, ValidationErrors};

pub trait Modify {
    /// Apply the provided modifiers to self
    fn modify(&mut self);
}

pub trait Validify: Modify + validator::Validate + Sized + From<Self::Payload> {
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
    ///     /* fn validate(payload: Self::Payload) { ... } */
    /// }
    ///
    /// ```
    type Payload: DeserializeOwned + Validate;

    /// Apply the provided modifiers to self and run validations.
    fn validate(payload: Self::Payload) -> Result<Self, ValidationErrors> {
        // Since the payload is all options, this will
        // only check if there are missing required fields
        <Self::Payload as ::validator::Validate>::validate(&payload)?;
        let mut this = Self::from(payload);
        <Self as Modify>::modify(&mut this);
        <Self as validator::Validate>::validate(&this)?;
        Ok(this)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ModType {
    Trim,
    Uppercase,
    Lowercase,
    Capitalize,
    Custom { function: String },
    Nested,
}
