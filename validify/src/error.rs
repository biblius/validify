use std::borrow::Cow;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{to_value, Value};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    Schema {
        code: Cow<'static, str>,
        message: Option<String>,
    },
    Field {
        name: Cow<'static, str>,
        code: Cow<'static, str>,
        params: HashMap<Cow<'static, str>, Value>,
        message: Option<String>,
    },
}

impl ValidationError {
    pub fn new_field(field: &'static str, code: &'static str) -> ValidationError {
        ValidationError::Field {
            name: Cow::from(field),
            code: Cow::from(code),
            message: None,
            params: HashMap::new(),
        }
    }

    pub fn new_schema(code: &'static str) -> ValidationError {
        ValidationError::Schema {
            code: Cow::from(code),
            message: None,
        }
    }

    pub fn add_param<T: Serialize>(&mut self, name: Cow<'static, str>, val: &T) {
        match self {
            ValidationError::Schema { .. } => {}
            ValidationError::Field { params, .. } => {
                params.insert(name, to_value(val).unwrap());
            }
        }
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

    pub fn params(&self) -> HashMap<Cow<'static, str>, Value> {
        match self {
            ValidationError::Schema { .. } => HashMap::new(),
            ValidationError::Field { params, .. } => params.clone(),
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
            ValidationError::Schema { ref code, .. } => code,
            ValidationError::Field { ref code, .. } => code,
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

impl PartialOrd for ValidationError {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self {
            ValidationError::Schema { .. } if matches!(other, ValidationError::Field { .. }) => {
                Some(std::cmp::Ordering::Greater)
            }
            ValidationError::Field { .. } if matches!(other, ValidationError::Schema { .. }) => {
                Some(std::cmp::Ordering::Less)
            }
            _ => Some(std::cmp::Ordering::Equal),
        }
    }
}

impl Ord for ValidationError {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use ValidationError::*;
        match self {
            Schema { .. } if matches!(other, Field { .. }) => std::cmp::Ordering::Greater,
            Field { .. } if matches!(other, Schema { .. }) => std::cmp::Ordering::Less,
            _ => std::cmp::Ordering::Equal,
        }
    }
}

#[derive(Default, Debug, Serialize, Clone, PartialEq)]
pub struct ValidationErrors(Vec<ValidationError>);

impl ValidationErrors {
    pub fn new() -> ValidationErrors {
        ValidationErrors(Vec::new())
    }

    /// Returns the combined outcome of a struct's validation result along with the nested
    /// validation result for one of its fields.
    pub fn merge(&mut self, mut errors: ValidationErrors) {
        self.0.append(&mut errors.0)
    }

    /// Returns a map of field-level validation errors found for the struct that was validated and
    /// any of it's nested structs that are tagged for validation.
    pub fn errors(&self) -> &[ValidationError] {
        &self.0
    }

    pub fn sort(&mut self) {
        self.0.sort_by(|a, b| b.cmp(a))
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
            ValidationError::Schema { code, ref message } => {
                write!(
                    fmt,
                    "Schema validation error: {} [{:?}]",
                    code,
                    message.as_ref().map_or_else(|| "", |msg| msg)
                )
            }
            ValidationError::Field {
                code,
                message,
                name,
                params,
            } => {
                write!(
                    fmt,
                    "Field ({}) validation error: {}, {{ message: {}, params: {:?} }}",
                    name,
                    code,
                    message.as_ref().map_or_else(|| "", |f| f),
                    params,
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
