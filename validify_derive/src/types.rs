use syn::__private::quote::{self, quote};

/// Contains all the validators that can be used
#[derive(Debug)]
pub enum Validator {
    Email(Email),
    Url(Url),
    CreditCard(CreditCard),
    Phone(Phone),
    Custom(Custom),
    Range(Range),
    Length(Length),
    NonControlCharacter(NonControlChar),
    Required(Required),
    MustMatch(MustMatch),
    Regex(Regex),
    Contains(Contains),
    In(In),
    Nested,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Modifier {
    Trim,
    Uppercase,
    Lowercase,
    Capitalize,
    Custom { function: syn::Path },
    Nested,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueOrPath<T: std::fmt::Debug + Clone + PartialEq> {
    Value(T),
    Path(syn::Path),
}

impl<T> ValueOrPath<T>
where
    T: quote::ToTokens + std::clone::Clone + std::cmp::PartialEq + std::fmt::Debug,
{
    pub fn to_tokens(&self) -> proc_macro2::TokenStream {
        match self {
            ValueOrPath::Value(val) => quote!(#val),
            ValueOrPath::Path(path) => quote!(#path),
        }
    }
}

#[derive(Debug)]
pub struct SchemaValidation {
    pub function: syn::Path,
}

pub trait Describe {
    fn code(&self) -> &str;

    fn message<'a>(&'a self) -> Option<&'a str>;
}

macro_rules! validation {
    ($id:ident : $code:literal $(,)? $($der:path),* ; $($key:ident : $typ:ty $(,)?),*) => {
        #[derive(Debug, $($der),*)]
        pub struct $id {
            $(pub $key:$typ,)*
            pub code: Option<String>,
            pub message: Option<String>,
        }

        impl $crate::types::Describe for $id {
            fn code(&self) -> &str {
                if let Some(ref code) = self.code {
                   code
                } else {
                    $code
                }
            }

            fn message<'a>(&'a self) -> Option<&'a str> {
                self.message.as_ref().map(|s| s.as_str())
            }
        }
    };
}

validation!(
    In : "in",
    Clone;
    path: syn::Path,
    not: bool
);

impl In {
    pub fn new(path: syn::Path, not: bool) -> Self {
        Self {
            path,
            not,
            code: None,
            message: None,
        }
    }
}

validation!(
    Email : "email",
    Default, Clone;
);

validation!(
    Url : "url",
    Default, Clone;
);

validation!(
    Phone : "phone",
    Default, Clone;
);

validation!(
    CreditCard : "credit_card",
    Default, Clone;
);

validation!(
    NonControlChar : "non_control_char",
    Default, Clone;
);

validation!(
    Length : "length",
    Default;
    min: Option<ValueOrPath<u64>>,
    max: Option<ValueOrPath<u64>>,
    equal: Option<ValueOrPath<u64>>
);

validation!(
    Range : "range",
    Default;
    min: Option<ValueOrPath<f64>>,
    max: Option<ValueOrPath<f64>>
);

validation!(
    MustMatch : "must_match";
    value: syn::Ident
);

impl MustMatch {
    pub fn new(id: syn::Ident) -> Self {
        Self {
            value: id,
            code: None,
            message: None,
        }
    }
}

#[derive(Debug)]
pub struct Contains {
    pub not: bool,
    pub value: String,
    pub code: Option<String>,
    pub message: Option<String>,
}

impl Describe for Contains {
    fn code(&self) -> &str {
        if let Some(ref code) = self.code {
            code
        } else if self.not {
            "contains_not"
        } else {
            "contains"
        }
    }

    fn message<'a>(&'a self) -> Option<&'a str> {
        self.message.as_ref().map(|s| s.as_str())
    }
}

impl Contains {
    pub fn new(value: String, not: bool) -> Self {
        Self {
            not,
            value,
            code: None,
            message: None,
        }
    }
}

validation!(
    Nested : "nested",
    Default;
);

validation!(
    Required : "required",
    Default;
);

validation!(
    Custom : "custom",
    Clone;
    path: syn::Path
);

impl Custom {
    pub fn new(f: syn::Path) -> Self {
        Self {
            path: f,
            code: None,
            message: None,
        }
    }
}

validation!(
    Regex : "regex",
    Clone;
    path: syn::Path);

impl Regex {
    pub fn new(path: syn::Path) -> Self {
        Self {
            path,
            code: None,
            message: None,
        }
    }
}
