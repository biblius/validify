use validify::Validify;

#[derive(Validify)]
struct Test {
    #[modify()]
    s: String,
}

fn main() {}
