#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/**/*.rs");
    t.pass("tests/run-pass/**/*.rs");
}