use validify::Validify;

#[derive(Validify)]
struct Test {
    #[modify(trim, uppercase)]
    s: String,
    #[modify(nested)]
    a: Nest,
}

struct Nest {
    a: String,
}

fn main() {}
