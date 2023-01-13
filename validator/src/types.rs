use std::borrow::Cow;
use std::collections::HashMap;

use serde::ser::Serialize;
use serde_derive::{Deserialize, Serialize};
use serde_json::{to_value, Value};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum ErrorType {
    Schema,
    Field,
}

impl PartialOrd for ErrorType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use ErrorType::*;
        match self {
            Schema if *other == Field => Some(std::cmp::Ordering::Greater),
            Field if *other == Schema => Some(std::cmp::Ordering::Less),
            _ => Some(std::cmp::Ordering::Equal),
        }
    }
}

impl Ord for ErrorType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use ErrorType::*;
        match self {
            Schema if *other == Field => std::cmp::Ordering::Greater,
            Field if *other == Schema => std::cmp::Ordering::Less,
            _ => std::cmp::Ordering::Equal,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub code: Cow<'static, str>,
    pub ty: ErrorType,
    pub field: Option<String>,
    pub message: Option<String>,
    pub params: HashMap<Cow<'static, str>, Value>,
}

impl ValidationError {
    pub fn new(code: &'static str, ty: ErrorType, field: Option<String>) -> ValidationError {
        ValidationError {
            code: Cow::from(code),
            ty,
            field,
            message: None,
            params: HashMap::new(),
        }
    }

    pub fn add_param<T: Serialize>(&mut self, name: Cow<'static, str>, val: &T) {
        self.params.insert(name, to_value(val).unwrap());
    }

    pub fn with_message<T: Serialize>(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }
}

impl std::error::Error for ValidationError {
    fn description(&self) -> &str {
        &self.code
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

impl PartialOrd for ValidationError {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        ErrorType::partial_cmp(&self.ty, &other.ty)
    }
}

impl Ord for ValidationError {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        ErrorType::cmp(&self.ty, &other.ty)
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

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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

impl Iterator for ValidationErrors {
    type Item = ValidationError;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}
