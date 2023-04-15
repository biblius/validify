use validify::Validate;

const ALLOWED_STRS: &[&str] = &["YES", "GOOD"];
const DISALLOWED_STRS: &[&str] = &["NO", "BAD"];

const ALLOWED_NUMS: &[u64] = &[1, 2, 3];
const DISALLOWED_NUMS: &[u64] = &[4, 5, 6];

#[test]
fn properly_validates() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(is_in(ALLOWED_STRS))]
        a: String,
        #[validate(is_in(ALLOWED_NUMS))]
        b: u64,
        #[validate(not_in(DISALLOWED_STRS))]
        c: String,
        #[validate(not_in(DISALLOWED_NUMS))]
        d: u64,
    }

    let s = TestStruct {
        a: "YES".to_string(),
        b: 1,
        c: "Relax".to_string(),
        d: 7,
    };

    assert!(s.validate().is_ok());

    let s = TestStruct {
        a: "GOOD".to_string(),
        b: 3,
        c: "Hello".to_string(),
        d: 1,
    };

    assert!(s.validate().is_ok());
}

#[test]
fn properly_errors() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(is_in(value = ALLOWED_STRS, message = "NOT_IN_ALLOWED"))]
        a: String,
        #[validate(is_in(value = ALLOWED_NUMS, message = "NOT_IN_ALLOWED"))]
        b: u64,
        #[validate(not_in(value = DISALLOWED_STRS, message = "IN_DISALLOWED"))]
        c: String,
        #[validate(not_in(value = DISALLOWED_NUMS, message = "IN_DISALLOWED"))]
        d: u64,
    }

    let s = TestStruct {
        a: "NO".to_string(),
        b: 1,
        c: "Still fine".to_string(),
        d: 1,
    };

    let err = s.validate();
    assert!(err.is_err());
    let err = err.unwrap_err();
    assert_eq!(err.errors()[0].code(), "in");
    assert_eq!(err.errors()[0].message().unwrap(), "NOT_IN_ALLOWED");

    let s = TestStruct {
        a: "YES".to_string(),
        b: 4,
        c: "Still fine".to_string(),
        d: 1,
    };

    let err = s.validate();
    assert!(err.is_err());
    let err = err.unwrap_err();
    assert_eq!(err.errors()[0].code(), "in");
    assert_eq!(err.errors()[0].message().unwrap(), "NOT_IN_ALLOWED");

    let s = TestStruct {
        a: "GOOD".to_string(),
        b: 1,
        c: "BAD".to_string(),
        d: 1,
    };

    let err = s.validate();
    assert!(err.is_err());
    let err = err.unwrap_err();
    assert_eq!(err.errors()[0].code(), "not_in");
    assert_eq!(err.errors()[0].message().unwrap(), "IN_DISALLOWED");

    let s = TestStruct {
        a: "YES".to_string(),
        b: 2,
        c: "Still fine".to_string(),
        d: 6,
    };

    let err = s.validate();
    assert!(err.is_err());
    let err = err.unwrap_err();
    assert_eq!(err.errors()[0].code(), "not_in");
    assert_eq!(err.errors()[0].message().unwrap(), "IN_DISALLOWED")
}
