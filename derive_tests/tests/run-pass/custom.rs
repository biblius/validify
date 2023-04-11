use validify::{Validate, ValidationError};

#[derive(Validate)]
struct Test {
    #[validate(custom(validate_something))]
    s: String,
}

fn validate_something(_s: &str) -> Result<(), ValidationError> {
    Ok(())
}

fn main() {}
