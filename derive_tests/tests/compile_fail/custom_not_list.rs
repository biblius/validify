use validify::Validate;

#[derive(Validate)]
struct Test {
    #[validate(custom = 2)]
    s: String,
}

fn main() {}
