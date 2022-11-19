use modificate::Modify;
use serde::{self, Deserialize, Serialize};
mod compile_fail;

/**
 * SIMPLE
 */

#[derive(Debug, Modify, Serialize, Deserialize)]
struct Testor {
    #[modifier(lowercase)]
    pub a: String,
    #[modifier(trim, uppercase)]
    pub b: Option<String>,
    #[modifier(custom = "do_something")]
    pub c: String,
    #[modifier(custom = "do_something")]
    #[serde(rename = "DDDDDD")]
    pub d: Option<String>,
    #[modifier(custom = "do_other")]
    pub e: usize,
    #[modifier(custom = "do_other")]
    pub f: Option<usize>,
}

fn do_something(input: &mut String) {
    *input = String::from("modified");
}

fn do_other(n: &mut usize) {
    *n = 10;
}

#[test]
fn simple_modify() {
    let mut test = Testor {
        a: "LOWER ME".to_string(),
        b: Some("  makemeshout   ".to_string()),
        c: "I'll never be the same".to_string(),
        d: Some("Me neither".to_string()),
        e: 0,
        f: Some(0),
    };
    test.modify();
    assert_eq!(test.a, "lower me");
    assert_eq!(test.b, Some("MAKEMESHOUT".to_string()));
    assert_eq!(test.c, "modified");
    assert_eq!(test.d, Some("modified".to_string()));
    assert_eq!(test.e, 10);
    assert_eq!(test.f, Some(10));
}
/**
 * NESTED
 */

#[derive(Debug, Modify, Serialize)]
struct Testamentor {
    #[modifier(trim, lowercase)]
    a: String,
    #[modifier(nested)]
    nestor: Nestor,
}

#[derive(Debug, Modify, Serialize)]
struct Nestor {
    #[modifier(custom = "do_other")]
    a: usize,
    #[modifier(custom = "do_something")]
    b: String,
    c: &'static str,
}

#[test]
fn nested_modify() {
    let mut testamentor = Testamentor {
        a: "LOWER ME".to_string(),
        nestor: Nestor {
            a: 5,
            b: "ALOHA".to_string(),
            c: "Snackbar",
        },
    };
    testamentor.modify();

    assert_eq!(testamentor.a, "lower me");
    assert_eq!(testamentor.nestor.a, 10);
    assert_eq!(testamentor.nestor.b, "modified");
    assert_eq!(testamentor.nestor.c, "Snackbar");
}

/**
 * COMPILE
 */

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("src/tests/compile-fail/**/*.rs");
    t.pass("tests/run-pass/**/*.rs");
}
