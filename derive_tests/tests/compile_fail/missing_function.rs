use validify::Validate;

#[derive(Validate)]
#[validate(())]
struct Test {
    s: i32,
}

fn hey(_: &Test) -> Option<(String, String)> {
    None
}

fn main() {}
