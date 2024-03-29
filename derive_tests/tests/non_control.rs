use validify::Validate;

#[test]
fn can_validate_utf8_ok() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(non_control_char)]
        val: String,
    }

    let s = TestStruct {
        val: "하늘".to_string(),
    };

    assert!(s.validate().is_ok());
}

#[test]
fn utf8_with_control_fails_validation() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(non_control_char)]
        val: String,
    }

    let s = TestStruct {
        val: "\u{009F}하늘".to_string(),
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "non_control_char");
}

#[test]
fn can_specify_code_for_non_control_character() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(non_control_char(code = "oops"))]
        val: String,
    }
    let s = TestStruct {
        val: "\u{009F}하늘".to_string(),
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "oops");
    assert_eq!(errs[0].params()["actual"], "\u{9F}하늘");
}

#[test]
fn can_specify_message_for_non_control_character() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(non_control_char(message = "oops"))]
        val: String,
    }
    let s = TestStruct {
        val: "\u{009F}하늘".to_string(),
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].clone().message().unwrap(), "oops");
}
