use validify::Validate;

#[test]
fn can_validate_valid_must_match() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(must_match(val2))]
        val: String,
        val2: String,
    }

    let s = TestStruct {
        val: "bob".to_string(),
        val2: "bob".to_string(),
    };

    assert!(s.validate().is_ok());
}

#[test]
fn not_matching_fails_validation() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(must_match(val2))]
        val: String,
        val2: String,
    }

    let s = TestStruct {
        val: "bob".to_string(),
        val2: "bobby".to_string(),
    };

    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "must_match");
    assert_eq!(errs[0].params()["actual"], "bob");
    assert_eq!(errs[0].params()["target"], "bobby");
}

#[test]
fn can_specify_code_for_must_match() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(must_match(value = val2, code = "oops"))]
        val: String,
        val2: String,
    }
    let s = TestStruct {
        val: "bob".to_string(),
        val2: "bobb".to_string(),
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "oops");
}

#[test]
fn can_specify_message_for_must_match() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(must_match(value = val2, message = "oops"))]
        val: String,
        val2: String,
    }
    let s = TestStruct {
        val: "bob".to_string(),
        val2: "bobb".to_string(),
    };
    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].clone().message().unwrap(), "oops");
}
