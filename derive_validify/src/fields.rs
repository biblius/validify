use traits::ModType;

/// Holds the `syn::Field` in `field`, its type and all the annotated
/// modifiers
#[derive(Debug)]
pub(super) struct FieldInformation {
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
