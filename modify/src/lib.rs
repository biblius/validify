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
}

/// This struct holds the combined validation information for one filed
#[derive(Debug)]
pub struct FieldInformation {
    pub field: syn::Field,
    pub field_type: String,
    pub name: String,
    pub modifiers: Vec<ModType>,
}

impl FieldInformation {
    pub fn new(
        field: syn::Field,
        field_type: String,
        name: String,
        modifiers: Vec<ModType>,
    ) -> Self {
        FieldInformation {
            field,
            field_type,
            name,
            modifiers,
        }
    }
}

#[cfg(test)]
mod tests {}
