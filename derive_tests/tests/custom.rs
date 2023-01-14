use validator::{Validate, ValidationError};

fn valid_custom_fn(_: &str) -> Result<(), ValidationError> {
    Ok(())
}

fn invalid_custom_fn(_: &str) -> Result<(), ValidationError> {
    Err(ValidationError::new_field("meh", "val"))
}

#[test]
fn can_validate_custom_fn_ok() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(custom = "valid_custom_fn")]
        val: String,
    }

    let s = TestStruct {
        val: "hello".to_string(),
    };

    assert!(s.validate().is_ok());
}

#[test]
fn can_fail_custom_fn_validation() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(custom = "invalid_custom_fn")]
        val: String,
    }

    let s = TestStruct { val: String::new() };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "meh");
    assert_eq!(errs[0].params()["value"], "");
}

#[test]
fn can_specify_message_for_custom_fn() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(custom(function = "invalid_custom_fn", message = "oops"))]
        val: String,
    }
    let s = TestStruct { val: String::new() };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].clone().message().unwrap(), "oops");
}
