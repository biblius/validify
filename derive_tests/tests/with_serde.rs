use serde::Deserialize;
use serde_json::json;
use validify::{Validate, Validify};

#[test]
fn returns_original_field_names() {
    #[derive(Debug, Validate, Deserialize)]
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
    #[derive(Debug, Validate, Deserialize)]
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

#[test]
fn returns_original_field_names_with_rename_all() {
    #[derive(Debug, Validate, Deserialize)]
    #[serde(rename_all = "camelCase")]
    #[validate(foo)]
    struct Test {
        #[validate(length(min = 1))]
        snake_case: String,
        #[validate(length(max = 5))]
        snake_case_two: String,
    }

    let test = json!({"snakeCase": "", "snakeCaseTwo": "123123"}).to_string();

    let res = serde_json::from_str::<Test>(&test).unwrap().validate();
    let err = res.unwrap_err();
    assert_eq!(err.errors().len(), 2);
    assert_eq!(err.errors()[0].field_name().unwrap(), "snakeCase");
    assert_eq!(err.errors()[1].field_name().unwrap(), "snakeCaseTwo");
    assert_eq!(err.errors()[0].location(), "/snakeCase");
    assert_eq!(err.errors()[1].location(), "/snakeCaseTwo");

    fn foo(_t: &Test) -> Result<(), validify::ValidationErrors> {
        Ok(())
    }
}

#[test]
fn returns_original_field_names_with_rename_all_deser() {
    #[derive(Debug, Validate, Deserialize)]
    #[serde(rename_all(serialize = "snake_case", deserialize = "camelCase"))]
    #[validate(foo)]
    struct Test {
        #[validate(length(min = 1))]
        snake_case: String,
        #[validate(length(max = 5))]
        snake_case_two: String,
    }

    let test = json!({"snakeCase": "", "snakeCaseTwo": "123123"}).to_string();

    let res = serde_json::from_str::<Test>(&test).unwrap().validate();
    let err = res.unwrap_err();
    assert_eq!(err.errors().len(), 2);
    assert_eq!(err.errors()[0].field_name().unwrap(), "snakeCase");
    assert_eq!(err.errors()[1].field_name().unwrap(), "snakeCaseTwo");
    assert_eq!(err.errors()[0].location(), "/snakeCase");
    assert_eq!(err.errors()[1].location(), "/snakeCaseTwo");

    fn foo(_t: &Test) -> Result<(), validify::ValidationErrors> {
        Ok(())
    }
}

#[test]
fn returns_original_field_names_with_custom_serde() {
    #[derive(Debug, Validify, Deserialize)]
    #[serde(rename_all(deserialize = "camelCase"))]
    #[validate(foo)]
    struct Test {
        #[validate(length(min = 10))]
        #[serde(deserialize_with = "custom_serde::deserialize")]
        snake_case: String,
        #[validate(length(max = 5))]
        #[serde(with = "custom_serde")]
        snake_case_two: String,
    }

    let test = json!({"snakeCase": "", "snakeCaseTwo": "123123"}).to_string();

    let json = serde_json::from_str::<Test>(&test).unwrap();
    assert_eq!(json.snake_case_two, "SUCCESS");
    let res = json.validate();
    let err = res.unwrap_err();
    assert_eq!(err.errors().len(), 2);
    assert_eq!(err.errors()[0].field_name().unwrap(), "snakeCase");
    assert_eq!(err.errors()[1].field_name().unwrap(), "snakeCaseTwo");
    assert_eq!(err.errors()[0].location(), "/snakeCase");
    assert_eq!(err.errors()[1].location(), "/snakeCaseTwo");

    fn foo(_t: &Test) -> Result<(), validify::ValidationErrors> {
        Ok(())
    }

    mod custom_serde {
        use serde::{self, Deserialize, Deserializer};
        pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
        where
            D: Deserializer<'de>,
        {
            String::deserialize(deserializer)?;
            Ok("SUCCESS".to_string())
        }
    }
}
