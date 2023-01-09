pub trait Modify {
    /// Apply the provided modifiers to self
    fn modify(&mut self);
}

pub trait Validify: Modify + validator::Validate {
    /// Apply the provided modifiers to self and run validations.
    fn validate(&mut self) -> Result<(), validator::ValidationErrors> {
        <Self as Modify>::modify(self);
        <Self as validator::Validate>::validate(self)
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
