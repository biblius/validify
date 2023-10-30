use super::parser::ValueOrPath;
use chrono::{NaiveDate, NaiveDateTime};
use proc_macro_error::abort;
use syn::{meta::ParseNestedMeta, spanned::Spanned, Lit};

#[derive(Debug)]
pub struct SchemaValidation {
    pub function: syn::Path,
}

/// Trait implemented by validators to output validation codes and messages.
pub trait Describe {
    fn code(&self) -> &str;

    fn message(&self) -> Option<&str>;
}

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
    Time(Time),
    In(In),
    Ip(Ip),
    Nested,
}

macro_rules! validation {
    ($id:ident : $code:literal $(,)? $($der:path),* ; $($key:ident : $typ:ty $(,)?),*) => {
        #[derive(Debug, $($der),*)]
        pub struct $id {
            $(pub $key:$typ,)*
            pub code: Option<String>,
            pub message: Option<String>,
        }

        impl $crate::validate::validation::Describe for $id {
            fn code(&self) -> &str {
                if let Some(ref code) = self.code {
                   code
                } else {
                    $code
                }
            }

            fn message(&self) -> Option<&str> {
                self.message.as_deref()
            }
        }
    };
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
    path: syn::Path
);

impl Regex {
    pub fn new(path: syn::Path) -> Self {
        Self {
            path,
            code: None,
            message: None,
        }
    }
}

validation!(
    Ip : "ip",
    Clone, Default;
    format: Option<IpFormat>
);

#[derive(Debug, Clone)]
pub enum IpFormat {
    V4,
    V6,
}

#[derive(Debug)]
pub struct In {
    pub not: bool,
    pub path: syn::Path,
    pub code: Option<String>,
    pub message: Option<String>,
}

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

impl Describe for In {
    fn code(&self) -> &str {
        if let Some(ref code) = self.code {
            code
        } else if self.not {
            "not_in"
        } else {
            "in"
        }
    }

    fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

#[derive(Debug, Default)]
pub struct Contains {
    pub not: bool,
    pub value: Option<ValueOrPath<Lit>>,
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

    fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

impl Contains {
    pub fn new(value: ValueOrPath<Lit>, not: bool) -> Self {
        Self {
            not,
            value: Some(value),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default)]
pub struct Time {
    pub op: TimeOp,
    pub target: Option<ValueOrPath<String>>,
    pub duration: Option<ValueOrPath<i64>>,
    pub format: Option<String>,
    pub inclusive: bool,
    pub code: Option<String>,
    pub message: Option<String>,

    /// Used in case a path is used for the duration. We have to keep track of which chrono::Duration method to call.
    pub multiplier: TimeMultiplier,
}

#[derive(Debug, Clone, Copy)]
pub enum TimeMultiplier {
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
    None,
}

impl Default for TimeMultiplier {
    fn default() -> Self {
        Self::None
    }
}

impl Describe for Time {
    fn code(&self) -> &str {
        if let Some(ref code) = self.code {
            code
        } else {
            match self.op {
                TimeOp::BeforeNow => "before_now",
                TimeOp::AfterNow => "after_now",
                TimeOp::Before if self.inclusive => "before_or_equal",
                TimeOp::Before => "before",
                TimeOp::After if self.inclusive => "after_or_equal",
                TimeOp::After => "after",
                TimeOp::BeforeFromNow => "before_from_now",
                TimeOp::AfterFromNow => "after_from_now",
                TimeOp::InPeriod => "in_period",
                TimeOp::None => unreachable!(),
            }
        }
    }

    fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

impl Time {
    pub fn assert(&self, meta: &ParseNestedMeta) -> Result<(), syn::Error> {
        if matches!(self.target, Some(ValueOrPath::Value(_))) && self.format.is_none() {
            return Err(meta.error("string literal targets must contain a format"));
        }

        if matches!(self.target, Some(ValueOrPath::Path(_))) && self.format.is_some() {
            return Err(meta.error("path targets cannot contain formats"));
        }

        let no_multiplier = matches!(self.target, Some(ValueOrPath::Path(_)))
            && matches!(self.multiplier, TimeMultiplier::None);

        if let (Some(ValueOrPath::Value(date_str)), Some(format)) = (&self.target, &self.format) {
            let dt = NaiveDateTime::parse_from_str(date_str, format);
            let d = NaiveDate::parse_from_str(date_str, format);
            if dt.is_err() && d.is_err() {
                abort!(
                    meta.path.span(),
                    "The target string does not match the provided format"
                )
            }
        }

        match self.op {
            TimeOp::BeforeNow => {
                if self.duration.is_some() {
                    abort!(meta.path.span(), "before_now cannot have interval");
                }
                if self.target.is_some() {
                    abort!(meta.path.span(), "before_now cannot have target");
                }
            }
            TimeOp::AfterNow => {
                if self.duration.is_some() {
                    abort!(meta.path.span(), "after_now cannot have interval");
                }
                if self.target.is_some() {
                    abort!(meta.path.span(), "after_now cannot have target");
                }
            }
            TimeOp::Before => {
                if self.target.is_none() {
                    abort!(meta.path.span(), "before must have a target");
                }
                if self.duration.is_some() {
                    abort!(meta.path.span(), "before cannot have interval");
                }
            }
            TimeOp::After => {
                if self.target.is_none() {
                    abort!(meta.path.span(), "after must have target");
                }
                if self.duration.is_some() {
                    abort!(meta.path.span(), "after cannot have interval");
                }
            }
            TimeOp::BeforeFromNow => {
                if no_multiplier {
                    abort!(meta.path.span(), "path targets must have an interval")
                }
                if self.target.is_some() {
                    abort!(meta.path.span(), "before_from_now cannot have target");
                }
                if self.duration.is_none() {
                    abort!(meta.path.span(), "before_from_now must have interval");
                }
                if let Some(value) = self.duration.as_ref().unwrap().peek_value() {
                    if *value < 0 {
                        abort!(
                            meta.path.span(),
                            "before_from_now must have a positive duration, if you need to validate after use after_from_now"
                        );
                    }
                }
            }
            TimeOp::AfterFromNow => {
                if no_multiplier {
                    abort!(meta.path.span(), "path targets must have an interval")
                }
                if self.target.is_some() {
                    abort!(meta.path.span(), "after_from_now cannot have target");
                }
                if self.duration.is_none() {
                    abort!(meta.path.span(), "after_from_now must have interval");
                }
                if let Some(value) = self.duration.as_ref().unwrap().peek_value() {
                    if *value < 0 {
                        abort!(
                            meta.path.span(),
                            "after_from_now must have a positive duration, if you need to validate before use before_from_now"
                        );
                    }
                }
            }
            TimeOp::InPeriod => {
                if no_multiplier {
                    abort!(meta.path.span(), "path targets must have an interval")
                }
                if self.target.is_none() {
                    abort!(meta.path.span(), "in_period must have target");
                }
                if self.duration.is_none() {
                    abort!(meta.path.span(), "in_period must have interval");
                }
            }
            TimeOp::None => {
                abort!(meta.path.span(), "op must be specified")
            }
        };
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TimeOp {
    BeforeNow,
    AfterNow,
    Before,
    After,
    BeforeFromNow,
    AfterFromNow,
    InPeriod,
    None,
}

impl From<String> for TimeOp {
    fn from(value: String) -> Self {
        use TimeOp::*;

        match value.as_str() {
            "before" => Before,
            "after" => After,
            "before_now" => BeforeNow,
            "after_now" => AfterNow,
            "before_from_now" => BeforeFromNow,
            "after_from_now" => AfterFromNow,
            "in_period" => InPeriod,
            _ => Self::None,
        }
    }
}

impl Default for TimeOp {
    fn default() -> Self {
        Self::None
    }
}
