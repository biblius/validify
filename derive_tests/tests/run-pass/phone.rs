use validify::Validate;

#[derive(Validate)]
struct Test {
    #[validate(phone)]
    s: String,
}

fn main() {}
