use serde::Deserialize;
use serde_json::json;
use validify::{Payload, Validify, ValidifyPayload};

#[test]
fn nested() {
    #[derive(Debug, Clone, Deserialize, Validify, Payload)]
    struct Testor {
        pub testor_a: String,
        #[validify]
        pub nested: nest::Nestor,
    }

    let json = json!({"testor_a": "hello", "nested": null});
    let res = Testor::validify_from(serde_json::from_value::<TestorPayload>(json).unwrap());
    let errs = res.unwrap_err();
    assert_eq!(errs.errors()[0].code(), "required");

    let json = json!({"testor_a": "hello"});
    let res = Testor::validify_from(serde_json::from_value::<TestorPayload>(json).unwrap());
    let errs = res.unwrap_err();
    assert_eq!(errs.errors()[0].code(), "required");

    let json = json!({"testor_a": "hello", "nested": {"a": "world"}});
    let res = Testor::validify_from(serde_json::from_value::<TestorPayload>(json).unwrap());
    let errs = res.unwrap_err();
    assert_eq!(errs.errors()[0].code(), "length");

    let json = json!({"testor_a": "hello", "nested": {"a": "world ah ah jdklsdaldks"}});
    let res = Testor::validify_from(serde_json::from_value::<TestorPayload>(json).unwrap());
    assert!(res.is_ok())
}

#[test]
fn nested_in_option() {
    #[derive(Debug, Clone, Deserialize, Validify, Payload)]
    struct Testor {
        pub testor_a: String,
        #[validify]
        pub _nested: Option<nest::Nestor>,
    }

    let json = json!({"testor_a": "hello", "_nested": null});
    let res = Testor::validify_from(serde_json::from_value::<TestorPayload>(json).unwrap());
    assert!(res.unwrap()._nested.is_none());

    let json = json!({"testor_a": "hello"});
    let res = Testor::validify_from(serde_json::from_value::<TestorPayload>(json).unwrap());
    assert!(res.unwrap()._nested.is_none());

    let json = json!({"testor_a": "hello", "_nested": {"a": "world"}});
    let res = Testor::validify_from(serde_json::from_value::<TestorPayload>(json).unwrap());
    let errs = res.unwrap_err();
    assert_eq!(errs.errors()[0].code(), "length");

    let json = json!({"testor_a": "hello", "_nested": {"a": "world ah ah jdklsdaldks"}});
    let res = Testor::validify_from(serde_json::from_value::<TestorPayload>(json).unwrap());
    assert!(res.is_ok())
}

#[test]
fn nested_in_list_collection() {
    #[derive(Debug, Clone, Deserialize, Validify, Payload)]
    struct Testor {
        pub testor_a: String,
        #[validify]
        pub _nested: Vec<nest::Nestor>,
    }

    let json = json!({"testor_a": "hello", "_nested": null});
    let res = Testor::validify_from(serde_json::from_value::<TestorPayload>(json).unwrap());
    let errs = res.unwrap_err();
    assert_eq!(errs.errors()[0].code(), "required");
    assert_eq!(errs.errors()[0].location(), "/_nested");

    let json = json!({"testor_a": "hello"});
    let res = Testor::validify_from(serde_json::from_value::<TestorPayload>(json).unwrap());
    let errs = res.unwrap_err();
    assert_eq!(errs.errors()[0].code(), "required");
    assert_eq!(errs.errors()[0].location(), "/_nested");

    let json = json!({"testor_a": "hello", "_nested": [{"a": "world"}]});
    let res = Testor::validify_from(serde_json::from_value::<TestorPayload>(json).unwrap());
    let errs = res.unwrap_err();
    assert_eq!(errs.errors()[0].code(), "length");
    assert_eq!(errs.errors()[0].location(), "/_nested/0/a");

    let json = json!({"testor_a": "hello", "_nested": [{"a": "world ah ah jdklsdaldks"}]});
    let res = Testor::validify_from(serde_json::from_value::<TestorPayload>(json).unwrap());
    assert!(res.is_ok())
}

#[test]
fn option_nested_in_list_collection() {
    #[derive(Debug, Clone, Deserialize, Validify, Payload)]
    struct Testor {
        pub testor_a: String,
        #[validify]
        pub _nested: Option<Vec<nest::Nestor>>,
    }

    let json = json!({"testor_a": "hello", "_nested": null});
    let res = Testor::validify_from(serde_json::from_value::<TestorPayload>(json).unwrap());
    assert!(res.is_ok());

    let json = json!({"testor_a": "hello"});
    let res = Testor::validify_from(serde_json::from_value::<TestorPayload>(json).unwrap());
    assert!(res.is_ok());

    let json = json!({"testor_a": "hello", "_nested": [{"a": "world"}]});
    let res = Testor::validify_from(serde_json::from_value::<TestorPayload>(json).unwrap());
    let errs = res.unwrap_err();
    assert_eq!(errs.errors()[0].code(), "length");
    assert_eq!(errs.errors()[0].location(), "/_nested/0/a");

    let json = json!({"testor_a": "hello", "_nested": [{"a": "world ah ah jdklsdaldks"}]});
    let res = Testor::validify_from(serde_json::from_value::<TestorPayload>(json).unwrap());
    assert!(res.is_ok());

    let json = json!({"testor_a": 1, "_nested": [{"a": "world ah ah jdklsdaldks"}]});
    let res = serde_json::from_value::<TestorPayload>(json);
    assert!(res.is_err())
}

#[test]
fn nest_like_no_tomorrow() {
    use nest::{Nestor, NestorPayload};

    #[derive(Debug, Clone, Deserialize, Validify, Payload)]
    struct Testor {
        #[validify]
        pub first: Option<First>,
        #[validify]
        pub second: Vec<Second>,
    }

    #[derive(Debug, Clone, Deserialize, Validify, Payload)]
    struct First {
        #[validify]
        second_nested: Second,
        #[validify]
        #[validate(length(min = 1))]
        nested: Option<Vec<nest::Nestor>>,
    }

    #[derive(Debug, Clone, Deserialize, Validify, Payload)]
    struct Second {
        i: usize,
        #[validify]
        #[validate(length(min = 1))]
        nested: Vec<Nestor>,
    }

    let json = json!({
        "first": {
            "second_nested": {
              "i": 1,
              "nested": [{ "a": "gooooooooooooood" }]
            }
        },
        "second": [
            { "i": 2, "nested": [{ "a": "gooooooooooooood" }] }]
    });
    let res = Testor::validify_from(serde_json::from_value::<TestorPayload>(json).unwrap());
    assert!(res.is_ok());

    let json = json!({
        "first": {
            "second_nested": {
              "i": 1,
              "nested": [{"a": "goooooooooooood"}]
            }
        },
        "second": [
            { "i": 1 }
        ]
    });
    let res = Testor::validify_from(serde_json::from_value::<TestorPayload>(json).unwrap());
    let errs = res.unwrap_err();
    assert_eq!(errs.errors()[0].code(), "required");
    assert_eq!(errs.errors()[0].location(), "/second/0/nested");

    let json = json!({
        "first": {
            "second_nested": {
              "i": 1,
              "nested": [{"a": "bad"}]
            }
        },
        "second": [
            { "i": 1, "nested": [{"a": "goooooooooooood"}] }
        ]
    });
    let res = Testor::validify_from(serde_json::from_value::<TestorPayload>(json).unwrap());
    let errs = res.unwrap_err();
    assert_eq!(errs.errors()[0].code(), "length");
    assert_eq!(
        errs.errors()[0].location(),
        "/first/second_nested/nested/0/a"
    );
}

#[test]
fn camel_case() {
    #[derive(Debug, Clone, Deserialize, Validify, Payload)]
    #[serde(rename_all = "camelCase")]
    struct Testor {
        #[modify(capitalize)]
        pub testor_a: String,
        #[modify(lowercase)]
        pub testor_b: String,
        #[validify]
        pub something_cameled: Option<Nestor>,
    }

    #[derive(Debug, Clone, Deserialize, Validify, Payload)]
    #[serde(rename_all = "camelCase")]
    struct Nestor {
        #[modify(trim, uppercase)]
        #[validate(length(min = 12))]
        case_this: String,
    }

    let json = json!({"testorA": "yea", "testorB": "WOOO", "somethingCameled": {"caseThis": "goooooooooooooooooood"}});
    let res = Testor::validify_from(serde_json::from_value::<TestorPayload>(json).unwrap());
    assert!(res.is_ok());
}

mod nest {
    use serde::Deserialize;
    use validify::{Payload, Validify};

    #[derive(Debug, Clone, Deserialize, Validify, Payload)]
    pub struct Nestor {
        #[modify(trim, uppercase)]
        #[validate(length(min = 12))]
        a: String,
    }
}
