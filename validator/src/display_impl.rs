use std::fmt::{self};

use crate::{ValidationError, ValidationErrors};

impl fmt::Display for ValidationError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(msg) = self.message.as_ref() {
            write!(fmt, "{}", msg)
        } else {
            write!(fmt, "Validation error: {} [{:?}]", self.code, self.params)
        }
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        for err in self.errors().iter() {
            writeln!(fmt, "{}", err)?;
        }
        Ok(())
    }
}
