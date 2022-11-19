pub trait Modify {
    /// Apply the provided modifiers to self
    fn modify(&mut self);
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
