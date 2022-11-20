use validify::Validify;

#[derive(Validify)]
struct Test {
    #[modify(trim, uppercase)]
    s: String,
    #[modify(nested)]
    a: String,
}

fn main() {}
