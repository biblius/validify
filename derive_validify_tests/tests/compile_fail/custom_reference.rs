use validify::Validify;

#[derive(Validify)]
struct Test {
    #[modify(trim, uppercase)]
    s: String,
    #[modify(custom = "aloha")]
    p: &'static str,
}
fn aloha(s: &mut str) {}

fn main() {}
