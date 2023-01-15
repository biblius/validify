use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use validify::{validify, Validify};

const ALLOWED: &[&str] = &["YOLO", "mcswag"];
const DISALLOWED: &[&str] = &["nono", "NO"];
const NUMBERS: &[i32] = &[1, 2, 3];
const NO_NUMBERS: &[i32] = &[4, 5, 6];

#[derive(Debug, Clone)]
#[validify]
struct T {
    #[modify(custom = "foo", trim, uppercase)]
    #[validate(length(min = 1), is_in = "ALLOWED", not_in = "DISALLOWED")]
    a: String,
    #[validify]
    b: U,
    #[modify(trim, lowercase)]
    #[validate(contains = "lmeo")]
    c: Vec<String>,
    #[modify(custom = "foo", trim, uppercase)]
    #[validate(length(min = 1), is_in = "ALLOWED", not_in = "DISALLOWED")]
    d: Option<String>,
    #[validate(is_in = "NUMBERS", not_in = "NO_NUMBERS")]
    e: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[validify]
struct U {
    #[validate(range(min = 1))]
    b: usize,
}

fn foo(a: &mut String) {
    *a = "  yolo    ".to_string();
}

#[test]
fn validate() {
    let t = T {
        a: String::from("nono"),
        b: U { b: 2 },
        c: vec!["lmeo".to_string()],
        d: Some("testovanje".to_string()),
        e: Some(2),
    };
    T::validify(t.into()).unwrap();
}
