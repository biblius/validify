use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use validify::{validify, Validify};

const ALLOWED: &[&str] = &["YOLO", "mcswag"];

#[derive(Debug, Clone)]
#[validify]
struct T {
    #[modify(custom = "foo", trim, uppercase)]
    #[validate(length(min = 1), is_in = "ALLOWED")]
    a: String,
    #[validify]
    b: U,
    #[modify(trim, lowercase)]
    #[validate(contains = "lmeo")]
    c: Vec<String>,
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
        a: String::from("foo"),
        b: U { b: 2 },
        c: vec!["lmeo".to_string()],
    };
    T::validate(t.into()).unwrap();
}
