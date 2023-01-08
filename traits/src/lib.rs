use validator::{Validate, ValidationErrors};

pub trait Modify {
    /// Apply the provided modifiers to self
    fn modify(&mut self);
}

pub trait Validify: Modify + Validate {
    /// Apply the provided modifiers to self and run validations.
    fn validate(&mut self) -> Result<(), ValidationErrors> {
        <Self as Modify>::modify(self);
        <Self as Validate>::validate(self)
    }
}

#[derive(Debug)]
pub enum ModType {
    Trim,
    Uppercase,
    Lowercase,
    Capitalize,
    Custom { function: String },
    Nested,
}
