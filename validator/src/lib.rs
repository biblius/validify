//! # Example:
//!
//! ```ignore, no_run
//! use serde::Deserialize;
//!
//! // A trait that the Validate derive will impl
//! use validator::{Validate, ValidationError};
//!
//! #[derive(Debug, Validate, Deserialize)]
//! struct SignupData {
//!     #[validate(email)]
//!     mail: String,
//!     #[validate(phone)]
//!     phone: String,
//!     #[validate(url)]
//!     site: String,
//!     #[validate(length(min = 1), custom = "validate_unique_username")]
//!     #[serde(rename = "firstName")]
//!     first_name: String,
//!     #[validate(range(min = 18, max = 20))]
//!     age: u32,
//! }
//!
//! fn validate_unique_username(username: &str) -> Result<(), ValidationError> {
//!     if username == "xXxShad0wxXx" {
//!         // the value of the username will automatically be added later
//!         return Err(ValidationError::new("terrible_username"));
//!     }
//!
//!     Ok(())
//! }
//!
//! match signup_data.validate() {
//!   Ok(_) => (),
//!   Err(e) => return e;
//! };
//! ```
//!
//! # Available Validations:
//! - `email`                 
//! - `url`                   
//! - `length`                
//! - `range`                 
//! - `must_match`            
//! - `contains`              
//! - `does_not_contain`      
//! - `custom`                
//! - `regex`                 
//! - `credit_card`           
//! - `phone`                 
//! - `non_control_character`
//! - `nested`                
//! - `required`              
//!
//! [Checkout the project README of an in-depth usage description with examples.](https://github.com/Keats/validator/blob/master/README.md)
//!
//! # Installation:
//! Add the validator to the dependencies in the Cargo.toml file.
//!
//! ```toml
//! [dependencies]
//! validator = { version = "0.12", features = ["derive"] }
//! ```

mod error;
mod traits;
mod validation;

pub use validation::cards::validate_credit_card;
pub use validation::contains::validate_contains;
pub use validation::does_not_contain::validate_does_not_contain;
pub use validation::email::validate_email;
pub use validation::ip::{validate_ip, validate_ip_v4, validate_ip_v6};
pub use validation::length::validate_length;
pub use validation::must_match::validate_must_match;
pub use validation::non_control_character::validate_non_control_character;
pub use validation::phone::validate_phone;
pub use validation::range::validate_range;

pub use validation::required::validate_required;
pub use validation::urls::validate_url;

pub use error::{ValidationError, ValidationErrors};
pub use traits::{Contains, HasLen, Validate, ValidateArgs};

pub use derive_validator::Validate;
