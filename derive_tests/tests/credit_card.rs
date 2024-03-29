use validify::Validate;

#[test]
fn can_validate_valid_card_number() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(credit_card)]
        val: String,
    }

    let s = TestStruct {
        val: "5236313877109142".to_string(),
    };

    assert!(s.validate().is_ok());
}

#[test]
fn bad_credit_card_fails_validation() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(credit_card)]
        val: String,
    }

    let s = TestStruct {
        val: "bob".to_string(),
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "credit_card");
    assert_eq!(errs[0].params()["actual"], "bob");
}

#[test]
fn can_specify_code_for_credit_card() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(credit_card(code = "oops"))]
        val: String,
    }
    let s = TestStruct {
        val: "bob".to_string(),
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "oops");
}

#[test]
fn can_specify_message_for_credit_card() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(credit_card(message = "oops"))]
        val: String,
    }
    let s = TestStruct {
        val: "bob".to_string(),
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].clone().message().unwrap(), "oops");
}
