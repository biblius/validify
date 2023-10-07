use lazy_static::lazy_static;
use regex::Regex;
use validify::Validate;

lazy_static! {
    static ref RE2: Regex = Regex::new(r"^[a-z]{2}$").unwrap();
}

#[test]
fn can_validate_valid_regex() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(regex(RE2))]
        val: String,
    }

    let s = TestStruct {
        val: "aa".to_string(),
    };

    assert!(s.validate().is_ok());
}

#[test]
fn bad_value_for_regex_fails_validation() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(regex(RE2))]
        val: String,
    }

    let s = TestStruct {
        val: "2".to_string(),
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "regex");
    assert_eq!(errs[0].params()["actual"], "2");
}

#[test]
fn can_specify_code_for_regex() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(regex(path = RE2, code = "oops"))]
        val: String,
    }
    let s = TestStruct {
        val: "2".to_string(),
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "oops");
}

#[test]
fn can_specify_message_for_regex() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(regex(path = RE2, message = "oops"))]
        val: String,
    }
    let s = TestStruct {
        val: "2".to_string(),
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].clone().message().unwrap(), "oops");
}
