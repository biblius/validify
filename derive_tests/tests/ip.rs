use validify::Validate;

#[test]
fn validates_valid_ip() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(ip)]
        val: String,
    }

    let s = TestStruct {
        val: "0.0.0.0".to_string(),
    };

    assert!(s.validate().is_ok());

    let s = TestStruct {
        val: "fe80::223:6cff:fe8a:2e8a".to_string(),
    };

    assert!(s.validate().is_ok());
}

#[test]
fn validates_valid_ip_v4() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(ip(format = "v4"))]
        val: String,
    }

    let s = TestStruct {
        val: "127.0.0.1".to_string(),
    };

    assert!(s.validate().is_ok())
}

#[test]
fn validates_valid_ip_v6() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(ip(format = "v6"))]
        val: String,
    }

    let s = TestStruct {
        val: "fe80::223:6cff:fe8a:2e8a".to_string(),
    };

    assert!(s.validate().is_ok())
}

#[test]
fn errors_bad_ip() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(ip)]
        val: String,
    }

    let s = TestStruct {
        val: "25,1,1,1".to_string(),
    };

    assert!(s.validate().is_err());

    let s = TestStruct {
        val: "2a02::223:6cff :fe8a:2e8a".to_string(),
    };

    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "ip");
}

#[test]
fn errors_bad_ip_v4() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(ip(format = "v4", code = "BAD", message = "NOT_GOOD"))]
        val: String,
    }

    let s = TestStruct {
        val: "192.0,4.16".to_string(),
    };

    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "BAD");
    assert!(matches!(errs[0].message(), Some(val) if val == "NOT_GOOD"));
}

#[test]
fn errors_bad_ip_v6() {
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(ip(format = "v6", code = "BAD", message = "NOT_GOOD"))]
        val: String,
    }

    let s = TestStruct {
        val: "2a02::223:6cff :fe8a:2e8a".to_string(),
    };

    let res = s.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.field_errors();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].code(), "BAD");
    assert!(matches!(errs[0].message(), Some(val) if val == "NOT_GOOD"));
}
