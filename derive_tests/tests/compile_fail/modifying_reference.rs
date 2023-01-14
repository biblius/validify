use validify::Validify;

#[derive(Validify)]
struct Test {
    #[modify(trim, uppercase)]
    s: String,
    #[modify(trim, uppercase)]
    p: &'static str,
}

fn main() {}
