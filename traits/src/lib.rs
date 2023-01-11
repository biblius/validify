use serde::de::DeserializeOwned;
use validator::Validate;

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

#[derive(Debug)]
pub struct ValidationErrors(Vec<validator::ValidationError>);

impl Default for ValidationErrors {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn merge(mut self, errors: ValidationErrors) -> Self {
        self.0.extend(errors);
        Self(self.0)
    }

    pub fn errors(&self) -> &[validator::ValidationError] {
        &self.0
    }
}

impl Iterator for ValidationErrors {
    type Item = validator::ValidationError;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl From<validator::ValidationErrors> for ValidationErrors {
    fn from(value: validator::ValidationErrors) -> Self {
        let mut errors = vec![];
        nest_validation_errors(value, &mut errors);
        Self(errors)
    }
}

/// Nests validation errors to one vec
fn nest_validation_errors(
    errs: validator::ValidationErrors,
    buff: &mut Vec<validator::ValidationError>,
) {
    for err in errs.errors().values() {
        match err {
            validator::ValidationErrorsKind::Struct(box_error) => {
                nest_validation_errors(*box_error.clone(), buff);
            }
            validator::ValidationErrorsKind::List(e) => {
                for er in e.clone().into_values() {
                    nest_validation_errors(*er.clone(), buff);
                }
            }
            validator::ValidationErrorsKind::Field(e) => {
                for er in e {
                    buff.push(er.clone());
                }
            }
        }
    }
}
