# Validify

A procedural macro built on top of the [validator](https://docs.rs/validator/latest/validator/) crate that provides attributes for field modifiers. Particularly useful in the context of web payloads.

## **Modifiers**

|   Modifier    |  Type    |        Description
|---------------|----------|-----------------------
|  trim*        |  String  | Removes surrounding whitespace
|  uppercase*   |  String  | Calls `.to_uppercase()`
|  lowercase*   |  String  | Calls `.to_lowercase()`
|  capitalize*  |  String  | Makes the first char of the string uppercase
|  custom       |    Any   | Takes a function whose argument is `&mut <Type>`
|  validify*    |  Struct  | Can only be used on fields that are structs implementing the `Validify` trait. Runs all the nested struct's modifiers and validations

\*Also works for Vec\<T> by running validate on each element.

## **Validators**

|       Validator       |    Type     |      Params     |        Description
|-----------------------|-------------|-----------------|-----------------------
| email                 |  String     |        --       | Checks emails based on [this spec](https://html.spec.whatwg.org/multipage/input.html#valid-e-mail-address).
| url                   |  String     |        --       | Checks if the string is a URL.
| length                | Collection  | min, max, equal | Checks if the field's collection length is within the specified params.
| range                 |  Number     |     min, max    | Checks if the field's value is in the specified range.
| must_match            |    Any      |       Any*      | Checks if the field matches the specified value
| contains              | Collection  |      Item*      | Checks if the collection contains the specified value
| does_not_contain      | Collection  |      Item*      | Checks if the collection doesn't contain the specified value
| non_control_character |  String     |        --       | Checks if the field contains control characters
| custom                |  Function   |      FnItem*    | Executes custom validation on the field specified by the end user
| regex                 |  String     |      Regex*     | Matches the provided regex against the field
| credit_card           |  String     |        --       | Checks if the field's value is a valid credit card number
| phone                 |  String     |        --       | Checks if the field's value is a valid phone number
| required              |  Option     |        --       | Checks whether the field's value is Some
| is_in                 |  String/Num |    Collection*  | Checks whether the field's value is in the provided collection
| not_in                |  String/Num |    Collection*  | Checks whether the field's value is not in the provided collection

\* Params are specified in string notation, i.e. `"param"`.

The crate provides the `Validify` trait and the `validify` attribute macro and supports all the functionality of the validator crate. The main addition here is that payloads can be modified before being validated.

This is useful, for example, when a payload's `String` field has a minimum length restriction and you don't want it to be just spaces. Validify allows you to modify the field before it gets validated so as to mitigate this problem.

Annotate the struct you want to modify and validate with the `validify` macro:

```rust
use validify::{validify, Validify};
#[validify]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Testor {
    #[modify(lowercase, trim)]
    #[validate(length(equal = 8))]
    pub a: String,
    #[modify(trim, uppercase)]
    pub b: Option<String>,
    #[modify(custom = "do_something")]
    pub c: String,
    #[modify(custom = "do_something")]
    pub d: Option<String>,
    #[validify]
    pub nested: Nestor,
}
#[validify]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Nestor {
    #[modify(trim, uppercase)]
    #[validate(length(equal = 12))]
    a: String,
    #[modify(capitalize)]
    #[validate(length(equal = 14))]
    b: String,
}
fn do_something(input: &mut String) {
    *input = String::from("modified");
}
let mut test = Testor {
  a: "   LOWER ME     ".to_string(),
  b: Some("  makemeshout   ".to_string()),
  c: "I'll never be the same".to_string(),
  d: Some("Me neither".to_string()),
  nested: Nestor {
    a: "   notsotinynow   ".to_string(),
      b: "capitalize me.".to_string(),
  },
};
// The magic line
let res = Testor::validify(test.into());

assert!(matches!(res, Ok(_)));

let test = res.unwrap();
// Parent
assert_eq!(test.a, "lower me");
assert_eq!(test.b, Some("MAKEMESHOUT".to_string()));
assert_eq!(test.c, "modified");
assert_eq!(test.d, Some("modified".to_string()));
// Nested
assert_eq!(test.nested.a, "NOTSOTINYNOW");
assert_eq!(test.nested.b, "Capitalize me.");
```

Notice how even though field `d` is an option, the function used to modify the field still takes in `&mut String`. This is because modifiers and validations are only executed when the field isn't `None`.

## How it works

Every struct annotated with `#[validify]` gets an associated payload struct, e.g.

```rust
#[validify]
struct Something {
  a: usize,
  b: String,
  c: Option<bool>
}
```

behind the scenes will generate an intermediary

```rust
#[derive(Debug, Clone, Deserialize, validify::Validate)]
struct SomethingPayload {
  #[validate(required)]
  a: Option<usize>,
  #[validate(required)]
  b: Option<String>
  c: Option<bool>

  /* From and Into impls */
}
```

Note that every field that isn't an option will be an 'optional' required field in the payload (solely to avoid deserialization errors). The `Validify` implementation first validates the required fields of the generated payload. If any required fields are missing, no further modification/validation is done and the errors are returned. Next, the payload is transformed to the original struct and modifications and validations are run on it.

Validify's `validify` method always takes in the generated payload and outputs the original struct if all validations have passed.

The macro automatically implements validator's `Validate` trait and validify's `Modify` trait in the wrapper trait `Validify`. This wrapper trait contains only the method `validify` which in the above example expands to:

```rust
    fn validify(payload: Self::Payload) -> Result<(), ValidationErrors> {
        <Self::Payload as ::validify::Validate>::validate(&payload)?;
        let mut this = Self::from(payload);
        let mut errors: Vec<::validify::ValidationErrors> = Vec::new();
        if let Err(e) = <Nestor as ::validify::Validify>::validify(this.nested.clone().into()) {
            errors.push(e.into());
        }
        <Self as ::validify::Modify>::modify(&mut this);
        if let Err(e) = <Self as ::validify::Validate>::validate(&this) {
            errors.push(e.into());
        }
        if !errors.is_empty() {
            let mut errs = ::validify::ValidationErrors::new();
            for err in errors {
                errs = errs.merge(err);
            }
            return Err(errs);
        }
        Ok(this)
    }
```

If you need schema level validations, schema validation from the validator crate is still supported, e.g.:

```rust
#[validify]
#[validate(schema(function = "validate_testor"))]
struct Testor { /* ... */ }

fn validate_testor(t: &Testor) {
  /* ... */
}
```

Like field level validation, schema level validation is performed after modification.

### **Example with route handler**

```rust
    fn actix_test() {
      #[validify]
      #[derive(Debug, Serialize)]
      struct JsonTest {
          #[modify(lowercase)]
          a: String,
          #[modify(trim, uppercase)]
          #[validate(length(equal = 11))]
          b: String,
      }

      let jt = JsonTest {
          a: "MODIFIED".to_string(),
          b: "    makemeshout    ".to_string(),
      };

      let json = actix_web::web::Json(jt.into());
      mock_handler(json)
    }

    fn mock_handler(data: actix_web::web::Json<JsonTestPayload> 
    /* OR data: actix_web::web::Json<<JsonTest as Validify>::Payload> */) {
      let data = data.0;
      let data = JsonTest::validify(data).unwrap();
      mock_service(data);
    }

    fn mock_service(data: JsonTest) {
      assert_eq!(data.a, "modified".to_string());
      assert_eq!(data.b, "MAKEMESHOUT".to_string())
    }
```

### **Example with Big Boi**

```rust

const WORKING_HOURS: &[&str] = &["08", "09", "10", "11", "12", "13", "14", "15", "16"];
const CAREER_LEVEL: &[&str] = &["One", "Two", "Over 9000"];
const STATUSES: &[&str] = &["online", "offline"];
const CONTRACT_TYPES: &[&str] = &["Fulltime", "Temporary"];
const ALLOWED_MIME: &[&str] = &["jpeg", "png"];
const ALLOWED_DURATIONS: &[i32] = &[1, 2, 3];

#[validify]
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[validate(schema(function = "schema_validation"))]
struct BigBoi {
    #[modify(trim)]
    #[validate(length(max = 300))]
    title: String,
    #[modify(trim)]
    #[validate(is_in = "STATUSES")]
    status: String,
    #[modify(capitalize, trim)]
    city_country: String,
    #[validate(length(max = 1000))]
    education: String,
    #[modify(capitalize)]
    type_of_workplace: Vec<String>,
    #[validate(is_in = "WORKING_HOURS")]
    working_hours: String,
    part_time_period: Option<String>,
    #[modify(capitalize)]
    #[validate(is_in = "CONTRACT_TYPES")]
    contract_type: String,
    indefinite_probation_period: bool,
    #[validate(is_in = "ALLOWED_DURATIONS")]
    indefinite_probation_period_duration: Option<i32>,
    #[validate(is_in = "CAREER_LEVEL")]
    career_level: String,
    #[modify(capitalize)]
    benefits: String,
    #[validate(length(max = 60))]
    meta_title: String,
    #[validate(length(max = 160))]
    meta_description: String,
    #[validate(is_in = "ALLOWED_MIME")]
    meta_image: String,
    #[validate(custom = "greater_than_now")]
    published_at: String,
    #[validate(custom = "greater_than_now")]
    expires_at: String,
    #[validify]
    languages: Vec<TestLanguages>,
    #[validify]
    tags: TestTags,
}


fn schema_validation(bb: &BigBoi) -> Result<(), ValidationErrors> {
    let mut errs = ValidationErrors::new();
    if bb.contract_type == "Fulltime" && bb.part_time_period.is_some() {
        errs.add(ValidationError::new_schema(
            "Fulltime contract cannot have part time period",
        ));
    }

    if bb.contract_type == "Fulltime"
        && bb.indefinite_probation_period
        && bb.indefinite_probation_period_duration.is_none()
    {
        errs.add(
            ValidationError::new_schema("No probation duration")
                .with_message("Indefinite probation duration must be specified".to_string()),
        );
    }
    if errs.is_empty() {
        return Ok(());
    }
    Err(errs)
}

fn greater_than_now(date: &str) -> Result<(), ValidationError> {
    let parsed = chrono::NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M:%S");
    match parsed {
        Ok(date) => {
            if date
                < chrono::NaiveDateTime::from_timestamp_opt(chrono::Utc::now().timestamp(), 0)
                    .unwrap()
            {
                Err(ValidationError::new_field(
                    "Date cannot be less than now",
                    "lmao",
                ))
            } else {
                Ok(())
            }
        }
        Err(e) => {
            eprintln!("Error parsing date: {e}");
            Err(ValidationError::new_field("Could not parse date", "lmao"))
        }
    }
}

#[validify]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct TestTags {
    #[modify(trim)]
    #[validate(length(min = 1, max = 10), custom = "validate_names")]
    names: Vec<String>,
}

fn validate_names(names: &[String]) -> Result<(), ValidationError> {
    for n in names.iter() {
        if n.len() > 10 || n.is_empty() {
            return Err(ValidationError::new_field(
                "Maximum length of 10 exceeded for name",
                "names",
            ));
        }
    }
    Ok(())
}

const PROFICIENCY: &[&str] = &["dunno", "killinit"];

#[validify]
#[derive(Serialize, Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TestLanguages {
    company_opening_id: String,
    #[modify(trim)]
    language: String,
    #[modify(trim)]
    #[validate(is_in = "PROFICIENCY")]
    proficiency: Option<String>,
    required: Option<bool>,
    created_by: String,
}


#[test]
fn biggest_of_bois() {
  let tags = TestTags {
        // Invalid length due to `validate_names`
        names: vec![
            "taggggggggggggggggggggggggg".to_string(),
            "tag".to_string(),
            "tag".to_string(),
        ],
    };

    let languages = vec![
        TestLanguages {
            company_opening_id: "yolo mcswag".to_string(),
            language: "    tommorrowlang     ".to_string(),

            // Invalid proficiency
            proficiency: Some("invalid      ".to_string()),
            required: Some(true),
            created_by: "me".to_string(),
        },
        TestLanguages {
            company_opening_id: "divops".to_string(),
            language: "go".to_string(),

            // Invalid proficiency
            proficiency: Some("    invalid".to_string()),
            required: None,
            created_by: "they".to_string(),
        },
    ];

    let big = BigBoi {
        title: "me so big".to_string(),

        // Invalid status
        status: "invalid".to_string(),

        city_country: "gradrzava".to_string(),
        description_roles_responsibilites: "ask no questions tell no lies".to_string(),
        education: "any".to_string(),
        type_of_workplace: vec!["dumpster".to_string(), "mcdonalds".to_string()],

        // Invalid working hours
        working_hours: "invalid".to_string(),

        // Part time period with fulltime contract type
        part_time_period: Some(String::new()),
        contract_type: "Fulltime".to_string(),

        // Fulltime period with no duration
        indefinite_probation_period: true,
        indefinite_probation_period_duration: None,

        // Invalid career level
        career_level: "Over 100000".to_string(),

        benefits: "none".to_string(),
        meta_title: "this struct is getting pretty big".to_string(),
        meta_description: "and it's king of annoying".to_string(),

        // Invalid mime type
        meta_image: "heic".to_string(),

        // Invalid time
        published_at: "1999-01-01 00:00:00".to_string(),

        // Invalid time
        expires_at: "1999-01-01 00:00:00".to_string(),
        languages,
        tags,
    };

    let res = BigBoi::validify(big.into());
    assert!(matches!(res, Err(e) if e.errors().len() == 11));

    let schema_errs = res.as_ref().unwrap_err().schema_errors();
    let field_errs = res.unwrap_err().field_errors();

    assert_eq!(schema_errs.len(), 2);
    assert_eq!(field_errs.len(), 9);
}

```
