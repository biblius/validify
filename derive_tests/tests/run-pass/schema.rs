use validify::{Validate, ValidationErrors};

#[derive(Validate)]
#[validate(hey)]
struct Test {
    s: String,
}

fn hey(_: &Test) -> Result<(), ValidationErrors> {
    Ok(())
}

#[derive(Validate)]
#[validate(hey2)]
struct Test2 {
    s: String,
}

fn hey2(_: &Test2) -> Result<(), ValidationErrors> {
    Ok(())
}

fn main() {}
