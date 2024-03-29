use validify::{Validate, ValidationError};

fn valid_custom_fn(arr: &[u8; 2]) -> Result<(), ValidationError> {
    match arr[0] == 1 {
        true => Ok(()),
        false => Err(ValidationError::new_field("meh")),
    }
}

#[test]
fn can_validate_valid_email_with_unsupported_array() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(email)]
        val: String,
        #[allow(dead_code)]
        array: [u8; 2],
    }

    let s = TestStruct {
        val: "bob@bob.com".to_string(),
        array: [0u8; 2],
    };

    assert!(s.validate().is_ok());
}

#[test]
fn can_validate_custom_with_unsupported_array() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(email)]
        val: String,
        #[validate(custom(valid_custom_fn))]
        array: [u8; 2],
    }

    let s = TestStruct {
        val: "bob@bob.com".to_string(),
        array: [1u8, 1u8],
    };

    assert!(s.validate().is_ok());
}

#[test]
fn can_fail_custom_with_unsupported_array() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(email)]
        val: String,
        #[validate(custom(function = valid_custom_fn, code = "meh"))]
        array: [u8; 2],
    }

    let s = TestStruct {
        val: "bob@bob.com".to_string(),
        array: [0u8, 1u8],
    };

    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "meh");
}
