/// Contains all the validators that can be used
#[derive(Debug, Clone)]
pub enum Validator {
    Email,
    Url,
    Custom {
        /// This is the name of the function that should be called
        function: String,
    },
    // String is the name of the field to match
    MustMatch(String),
    // No implementation in this crate, it's all in validator_derive
    Regex(String),
    Range {
        min: Option<ValueOrPath<f64>>,
        max: Option<ValueOrPath<f64>>,
    },
    // Any value that impl HasLen can be validated with Length
    Length {
        min: Option<ValueOrPath<u64>>,
        max: Option<ValueOrPath<u64>>,
        equal: Option<ValueOrPath<u64>>,
    },
    CreditCard,
    Phone,
    Nested,
    NonControlCharacter,
    Required,
    RequiredNested,
    Contains(String),
    DoesNotContain(String),
    In(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueOrPath<T: std::fmt::Debug + Clone + PartialEq> {
    Value(T),
    Path(String),
}

impl Validator {
    pub fn code(&self) -> &'static str {
        match *self {
            Validator::MustMatch(_) => "must_match",
            Validator::Email => "email",
            Validator::Url => "url",
            Validator::Custom { .. } => "custom",
            Validator::Contains(_) => "contains",
            Validator::Regex(_) => "regex",
            Validator::Range { .. } => "range",
            Validator::Length { .. } => "length",
            Validator::CreditCard => "credit_card",
            Validator::Phone => "phone",
            Validator::Nested => "nested",
            Validator::NonControlCharacter => "non_control_character",
            Validator::Required => "required",
            Validator::RequiredNested => "required_nested",
            Validator::DoesNotContain(_) => "does_not_contain",
            Validator::In(_) => "is_in",
        }
    }
}
