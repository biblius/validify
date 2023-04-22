use serde::{Deserialize, Serialize};
use serde_json::json;
use validify::Validate;

#[test]
fn returns_original_field_names() {
    #[derive(Debug, Validate, Serialize, Deserialize)]
    struct Test {
        #[validate(length(min = 1))]
        #[serde(rename = "snakeCase")]
        snake_case: String,
        #[validate(length(max = 5))]
        #[serde(rename(deserialize = "snakeCaseTwo", serialize = "not_important"))]
        snake_case_two: String,
    }

    let test = Test {
        snake_case: "".to_string(),
        snake_case_two: "1312213".to_string(),
    };

    let res = test.validate();
    let err = res.unwrap_err();
    assert_eq!(err.errors().len(), 2);
    assert_eq!(err.errors()[0].location(), "/snakeCase");
    assert_eq!(err.errors()[0].field_name().unwrap(), "snakeCase");
    assert_eq!(err.errors()[1].location(), "/snakeCaseTwo");
    assert_eq!(err.errors()[1].field_name().unwrap(), "snakeCaseTwo");
}

#[test]
fn returns_original_field_names_from_json() {
    #[derive(Debug, Validate, Serialize, Deserialize)]
    struct Test {
        #[validate(length(min = 1))]
        #[serde(rename = "snakeCase")]
        snake_case: String,
        #[validate(length(max = 5))]
        #[serde(rename(deserialize = "snakeCaseTwo", serialize = "not_important"))]
        snake_case_two: String,
    }

    let test = json!({"snakeCase": "", "snakeCaseTwo": "123123"}).to_string();

    let res = serde_json::from_str::<Test>(&test).unwrap().validate();
    let err = res.unwrap_err();
    assert_eq!(err.errors().len(), 2);
    assert_eq!(err.errors()[0].location(), "/snakeCase");
    assert_eq!(err.errors()[0].field_name().unwrap(), "snakeCase");
    assert_eq!(err.errors()[1].location(), "/snakeCaseTwo");
    assert_eq!(err.errors()[1].field_name().unwrap(), "snakeCaseTwo");
}
