use serde::{Deserialize, Serialize};
use validify::{field_err, schema_err, schema_validation};
#[allow(unused_imports)]
use validify::{validify, ValidationErrors, Validify};

const ALLOWED: &[&str] = &["YOLO", "mcswag"];
const DISALLOWED: &[&str] = &["nono", "NO"];
const NUMBERS: &[i32] = &[1, 2, 3];
const NO_NUMBERS: &[i32] = &[4, 5, 6];

#[derive(Debug, Clone)]
#[validify]
#[validate(schema(function = "validator_test"))]
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

#[schema_validation]
fn validator_test(t: &T) -> Result<(), ValidationErrors> {
    if t.a == "Super no" {
        field_err!("a", "breh", "Can't be super no", errors);
    }
    if t.a == "YOLO" && t.e.is_none() {
        schema_err!("Invalid YOLO", "Can't yolo with non existent e", errors)
    }
}

#[test]
fn validate() {
    let t = T {
        a: String::from("nono"),
        b: U { b: 2 },
        c: vec!["lmeo".to_string()],
        d: Some("testovanje".to_string()),
        e: None,
    };
    let res = T::validify(t.into());
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.errors()[0].code(), "Invalid YOLO");
    assert_eq!(
        err.errors()[0].message(),
        Some("Can't yolo with non existent e".to_string())
    );
}
