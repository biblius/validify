use validify::Validate;

#[derive(Validate)]
struct Test {
    #[validate(url)]
    s: String,
}

fn main() {}
