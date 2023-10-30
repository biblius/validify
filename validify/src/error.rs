use serde::{Deserialize, Serialize};
use serde_json::{to_value, Value};
use std::{collections::HashMap, fmt::Display};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    Schema {
        /// The 'code' usually indicating what type of validation failed
        code: &'static str,

        /// An optional message for the error
        message: Option<String>,

        /// Where in the struct the validation failed. Similar to JSON pointers.
        /// Always '/' for schema.
        location: String,
    },
    Field {
        /// The name of the field
        field: Option<&'static str>,

        /// The 'code' usually indicating what type of validation failed
        code: &'static str,

        /// The parameters used to specify the exact cause of validation failure.
        /// Will usually be `actual` and `target`.
        params: Box<HashMap<&'static str, Value>>,

        /// An optional message for the error
        message: Option<String>,

        /// Where in the struct the validation failed. Similar to JSON pointers.
        location: String,
    },
}

impl ValidationError {
    /// Creates a new field validation error. This should be used when returning errors from
    /// reusable custom functions as validify will automatically set field names.
    pub fn new_field(code: &'static str) -> ValidationError {
        ValidationError::Field {
            field: None,
            code,
            message: None,
            params: Box::<HashMap<&'static str, Value>>::default(),
            location: String::new(),
        }
    }

    /// Creates a new field error with the given field name and code.
    pub fn new_field_named(field: &'static str, code: &'static str) -> ValidationError {
        ValidationError::Field {
            field: Some(field),
            code,
            message: None,
            params: Box::<HashMap<&'static str, Value>>::default(),
            location: String::new(),
        }
    }

    /// Creates a new schema error with the given code
    pub fn new_schema(code: &'static str) -> ValidationError {
        ValidationError::Schema {
            code,
            message: None,
            location: String::new(),
        }
    }

    /// Set the field name to the given one if the error is a field error
    pub fn set_field(&mut self, field: &'static str) {
        if let ValidationError::Field { field: f, .. } = self {
            *f = Some(field);
        }
    }

    /// Get the error's field name if it was a field error
    pub fn field_name(&self) -> Option<&str> {
        if let ValidationError::Field { field, .. } = self {
            *field
        } else {
            None
        }
    }

    pub fn add_param<T: Serialize>(&mut self, name: &'static str, val: &T) {
        match self {
            ValidationError::Schema { .. } => {}
            ValidationError::Field { params, .. } => {
                params.insert(name, to_value(val).unwrap());
            }
        }
    }

    pub fn with_param<T: Serialize>(mut self, name: &'static str, val: &T) -> Self {
        match self {
            ValidationError::Schema { .. } => {}
            ValidationError::Field { ref mut params, .. } => {
                params.insert(name, to_value(val).unwrap());
            }
        }
        self
    }

    pub fn with_message(mut self, msg: String) -> Self {
        match self {
            ValidationError::Schema {
                ref mut message, ..
            } => *message = Some(msg),
            ValidationError::Field {
                ref mut message, ..
            } => *message = Some(msg),
        }
        self
    }

    /// Insert the provided parent to the 0th position of the current location
    pub fn set_location<T>(&mut self, parent: T)
    where
        T: Display,
    {
        match self {
            ValidationError::Field {
                ref mut location, ..
            } => location.insert_str(0, &format!("/{parent}")),
            ValidationError::Schema {
                ref mut location, ..
            } => location.insert_str(0, &format!("/{parent}")),
        }
    }

    /// Used when the struct failing validation is nested in collections. It will concat the index
    /// to the parent so as to follow the location. We always have the parent in string form in the field quoter.
    pub fn set_location_idx<T: Display>(&mut self, idx: T, parent: &str) {
        match self {
            ValidationError::Field {
                ref mut location, ..
            } => location.insert_str(0, &format!("/{parent}/{idx}")),
            ValidationError::Schema {
                ref mut location, ..
            } => location.insert_str(0, &format!("/{parent}/{idx}")),
        }
    }

    /// Returns the apsolute location of the error in a similiar manner to JSON pointers.
    pub fn location(&self) -> &str {
        match self {
            ValidationError::Schema { .. } => "/",
            ValidationError::Field { ref location, .. } => location,
        }
    }

    pub fn params(&self) -> HashMap<&'static str, Value> {
        match self {
            ValidationError::Schema { .. } => HashMap::new(),
            ValidationError::Field { params, .. } => *params.clone(),
        }
    }

    pub fn code(&self) -> String {
        match self {
            ValidationError::Schema { code, .. } => code.to_string(),
            ValidationError::Field { code, .. } => code.to_string(),
        }
    }

    pub fn message(&self) -> Option<String> {
        match self {
            ValidationError::Schema { ref message, .. } => message.clone(),
            ValidationError::Field { ref message, .. } => message.clone(),
        }
    }

    pub fn set_message(&mut self, msg: String) {
        match self {
            ValidationError::Schema {
                ref mut message, ..
            } => *message = Some(msg),
            ValidationError::Field {
                ref mut message, ..
            } => *message = Some(msg),
        }
    }
}

impl std::error::Error for ValidationError {
    fn description(&self) -> &str {
        match self {
            ValidationError::Schema { code, .. } => code,
            ValidationError::Field { code, .. } => code,
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

#[derive(Default, Debug, Serialize, Clone, PartialEq)]
pub struct ValidationErrors(Vec<ValidationError>);

impl ValidationErrors {
    pub fn new() -> ValidationErrors {
        ValidationErrors(Vec::new())
    }

    /// Merge validation errors from the given arg to the calling errors.
    pub fn merge(&mut self, errors: ValidationErrors) {
        self.0.append(
            &mut errors
                .0
                .into_iter()
                .filter(|err| !self.0.contains(err))
                .collect(),
        )
    }

    /// Returns a slice of all the errors that ocurred during validation
    pub fn errors(&self) -> &[ValidationError] {
        &self.0
    }

    pub fn add(&mut self, error: ValidationError) {
        self.0.push(error)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn errors_mut(&mut self) -> &mut [ValidationError] {
        &mut self.0
    }

    pub fn field_errors(&self) -> Vec<ValidationError> {
        self.0
            .iter()
            .filter(|err| matches!(err, ValidationError::Field { .. }))
            .cloned()
            .collect()
    }

    pub fn schema_errors(&self) -> Vec<ValidationError> {
        self.0
            .iter()
            .filter(|err| matches!(err, ValidationError::Schema { .. }))
            .cloned()
            .collect()
    }
}

impl std::error::Error for ValidationErrors {
    fn description(&self) -> &str {
        "Validation failed"
    }
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::Schema {
                code,
                ref message,
                ref location,
            } => {
                let message = message.as_ref().map_or_else(|| "", |f| f);
                write!(
                    fmt,
                    "Schema validation error: {{ code: {code} message: {message}, location: {location} }}"
                )
            }
            ValidationError::Field {
                code,
                message,
                field,
                params,
                location,
            } => {
                let message = message.as_ref().map_or_else(|| "", |f| f);
                write!(
                    fmt,
                    "Validation error: {{ code: {code} location: {location}, field: {}, message: {message}, params: {params:?} }}", field.unwrap()
                )
            }
        }
    }
}

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for err in self.errors().iter() {
            writeln!(fmt, "{err}")?;
        }
        Ok(())
    }
}
