# Validify

[![Build](https://img.shields.io/github/actions/workflow/status/biblius/validify/check.yml?logo=github&style=plastic)](https://github.com/biblius/validify)
[![test](https://img.shields.io/github/actions/workflow/status/biblius/validify/test.yml?label=test&logo=github&style=plastic)](https://github.com/biblius/validify)
[![coverage](https://img.shields.io/codecov/c/github/biblius/validify?color=%23383&logo=codecov&style=plastic)](https://app.codecov.io/gh/biblius/validify/tree/master)
[![docs](https://img.shields.io/docsrs/validify?logo=rust&style=plastic)](https://docs.rs/validify/latest/validify/)
[![version](https://img.shields.io/crates/v/validify?logo=rust&style=plastic)](https://crates.io/crates/validify)

A procedural macro that provides attributes for field validation and modifiers. Particularly useful in the context of web payloads.

## **Modifiers**

| Modifier     | Type                                                 | Description                                                                                                                                                |
| ------------ | ---------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| trim\*       | String                                               | Removes surrounding whitespace                                                                                                                             |
| uppercase\*  | String                                               | Calls `.to_uppercase()`                                                                                                                                    |
| lowercase\*  | String                                               | Calls `.to_lowercase()`                                                                                                                                    |
| capitalize\* | String                                               | Makes the first char of the string uppercase                                                                                                               |
| custom       | Any                                                  | Takes a function whose argument is `&mut <Type>`                                                                                                           |
| validify     | impl Validify / impl Iterator\<Item = impl Validify> | Can only be used on fields that are structs (or collections of) implementing the `Validify` trait. Runs all the nested struct's modifiers and validations. |

\*Also works for Vec\<String> by running the modifier on each element.

## **Validators**

All validators also take in a `code` and `message` as parameters, their values are must be string literals if specified.

| Validator        | Type             | Params          | Param type    | Description                                                                                                                           |
| ---------------- | ---------------- | --------------- | ------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| email            | String           | --              | --            | Checks emails based on [this spec](https://html.spec.whatwg.org/multipage/input.html#valid-e-mail-address).                           |
| ip               | String           | format          | Ident (v4/v6) | Checks if the string is an IP address.                                                                                                |
| url              | String           | --              | --            | Checks if the string is a URL.                                                                                                        |
| length           | Collection       | min, max, equal | LitInt        | Checks if the collection length is within the specified params. Works through the HasLen trait.                                       |
| range            | Int/Float        | min, max        | LitFloat      | Checks if the value is in the specified range.                                                                                        |
| must_match       | Any              | value           | Ident         | Checks if the field matches another field of the struct. The value must be equal to a field identifier on the deriving struct.        |
| contains         | Collection       | value           | Lit/Path      | Checks if the collection contains the specified value. If used on a K,V collection, it checks whether it has the provided key.        |
| contains_not     | Collection       | value           | Lit/Path      | Checks if the collection doesn't contain the specified value. If used on a K,V collection, it checks whether it has the provided key. |
| non_control_char | String           | --              | --            | Checks if the field contains control characters                                                                                       |
| custom           | Function         | function        | Path          | Executes custom validation on the field by calling the provided function                                                              |
| regex            | String           | path            | Path          | Matches the provided regex against the field. Intended to be used with lazy_static by providing a path to an initialised regex.       |
| credit_card      | String           | --              | --            | Checks if the field's value is a valid credit card number                                                                             |
| phone            | String           | --              | --            | Checks if the field's value is a valid phone number                                                                                   |
| required         | Option\<T>       | --              | --            | Checks whether the field's value is Some                                                                                              |
| is_in            | impl PartialEq   | collection      | Path          | Checks whether the field's value is in the specified collection                                                                       |
| not_in           | impl PartialEq   | collection      | Path          | Checks whether the field's value is not in the specified collection                                                                   |
| validate         | impl Validate    | --              | --            | Calls the underlying structs                                                                                                          |
| time             | NaiveDate\[Time] | See below       | See below     | Performs a check based on the specified op                                                                                            |

### **Time operators**

All time operators can take in `inclusive = bool`.

`in_period` and the `*_from_now` operators are inclusive by default.

The `target` param must be a string literal date or a path to an argless function that returns a date\[time].

If the target is a string literal, it must contain a `format` param, as per [this](https://docs.rs/chrono/latest/chrono/format/strftime/index.html).

Accepted interval parameters are `seconds`, `minutes`, `hours`, `days`, `weeks`.

The `_from_now` operators should not use negative duration due to how they validate the inputs,
negative duration for `in_period` works fine.

| Op              | Params           | Description                                                                      |
| --------------- | ---------------- | -------------------------------------------------------------------------------- |
| before          | target           | Check whether a date\[time] is before the target one                             |
| after           | target           | Check whether a date\[time] is after the target one                              |
| before_now      | --               | Check whether a date\[time] is before today\[now]                                |
| after_now       | --               | Check whether a date\[time] is after today\[now]                                 |
| before_from_now | interval         | Check whether a date\[time] is before the specified interval from today\[now]    |
| after_from_now  | interval         | Check whether a date\[time] is after the specified interval from the today\[now] |
| in_period       | target, interval | Check whether a date\[time] falls within a certain period                        |

Annotate the struct you want to modify and validate with the `Validify` attribute (if you do not need payload modification, derive the `validify::Validate` trait):

```rust
use validify::Validify;

#[derive(Debug, Clone, serde::Deserialize, Validify)]
struct Testor {
    #[modify(lowercase, trim)]
    #[validate(length(equal = 8))]
    pub a: String,
    #[modify(trim, uppercase)]
    pub b: Option<String>,
    #[modify(custom(do_something))]
    pub c: String,
    #[modify(custom(do_something))]
    pub d: Option<String>,
    #[validify]
    pub nested: Nestor,
}

#[derive(Debug, Clone, serde::Deserialize, Validify)]
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

Every struct annotated with `#[derive(Validify)]` gets an associated payload struct, e.g.

```rust
#[derive(validify::Validify)]
struct Something {
  a: usize,
  b: String,
  c: Option<bool>
}
```

behind the scenes will generate an intermediary

```rust
#[derive(Debug, Clone, serde::Deserialize, validify::Validate)]
struct SomethingPayload {
  #[validate(required)]
  a: Option<usize>,
  #[validate(required)]
  b: Option<String>,
  c: Option<bool>,

  /* From and Into impls */
}
```

Note that every field that isn't an option will be an 'optional' required field in the payload. This is done to avoid deserialization errors for missing fields.

- _Do note that if a field exists in the incoming client payload, but is of the wrong type, a deserialization error will still occur as the payload is only being validated for whether the necessary fields exist. The same applies for invalid date\[time] formats._

Even though the payload struct cannot help with wrong types, it can still prove useful and provide a bit more meaningful error messages when fields are missing.

Struct level annotations, such as `#[serde(renameAll = "...")]` are propagated to the payload.

When a struct contains nested validifies (child structs annotated with `#[validify]`), all the children in the payload will also be transformed and validated as payloads first.

Validify exposes two methods for validation/modification;

`validify` which takes in the payload and validates its required fields first and

`validify_self` which runs modifications and validations on the original struct, without ever using the payload.

In the context of web, you'll most likely be using `validify`. As such, the request handler should always take in the payload struct.

The `Validify` implementation first validates the required fields of the generated payload. If any required fields are missing, no further modification/validation is done and the errors are returned. Next, the payload is transformed to the original struct and modifications and validations are run on it.

Validify's `validify` method is called from the original struct with the associated payload struct as its argument and outputs the original struct if all validations have passed.

## Schema validation

Schema level validations can be performed using the following:

```rust
use validify::{Validify, ValidationErrors, schema_validation, schema_err};
#[derive(validify::Validify)]
#[validate(validate_testor)]
struct Testor {
    a: String,
    b: usize,
}

#[schema_validation]
fn validate_testor(t: &Testor) -> Result<(), ValidationErrors> {
  if t.a.as_str() == "yolo" && t.b < 2 {
    schema_err!("Invalid Yolo", "Cannot yolo with b < 2");
  }
}
```

The `#[schema_validation]` proc macro expands the function to:

```rust, ignore
fn validate_testor(t: &Testor) -> Result<(), ValidationErrors> {
    let mut errors = ValidationErrors::new();
    if t.a == "yolo" && t.b < 2 {
        errors.add(ValidationError::new_schema("Invalid Yolo").with_message("Cannot yolo with b < 2".to_string()));
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
```

This makes schema validations a bit more ergonomic and concise.
Like field level validation, schema level validation is performed after modification.

## Errors

The main ValidationError is an enum with 2 variants, Field and Schema. Field errors are, as the name suggests, created when fields fail validation and are usually automatically generated unless using custom handlers (custom field validation functions always must return a result whose Err variant is ValidationError).

If you want to provide a message along with the error, you can directly specify it in the attribute (the same goes for the code),
for example:

`#[validate(contains(value = "something", message = "Does not contain something", code = "MUST_CONTAIN"))]`

Keep in mind, when specifying validations this way, all attribute parameters MUST be specified as [NameValue](https://docs.rs/syn/latest/syn/struct.MetaNameValue.html) pairs. This means that if you write

`#[validate(contains("something", message = "Bla"))]`,

you will get an error because the parser expects either a single value or multiple name value pairs.

The `field_err!` macro provides a shorthand for creating field errors when using custom functions.

### Location

Locations are tracked for each error in a similar manner to [JSON pointers](https://opis.io/json-schema/2.x/pointers.html). When using custom validation, whatever field name you specify in the returned error will be used in the location for that field. Keep in mind locations are not reliable when dealing with hashed map/set collections as the item ordering for those is not guaranteed.

Error location display will depend on the original client payload, i.e. they will be displayed in the original case the payload was received (e.g. when using serde's `rename_all`). Any overriden field names will be displayed as such.

### Schema

Schema errors are usually created by the user in schema validation. The `schema_err!` macro alongside `#[schema_validation]` provides an ergonomic way to create schema errors. All errors are composed to a `ValidationErrors` struct which contains a vec of all the validation errors.

### Params

When sensible, validify automatically appends failing parameters and the target values they were validated against to the errors created to provide more clarity to the client and to save some manual work.

One parameter that is always appended is the `actual` field which represents the specific property of the violating field's validator during the validation. Some validators append additional data to the errors representing the expected values for the field.

## The payload struct and serde

Struct level attributes, such as `rename_all` are propagated to the payload. When attributes that modify field names are present, any field names in returned errors will be represented as the original (i.e. client payload).

There are a few special serde attributes that validify treats differently; `rename`, `with` and `deserialize_with`.
It is **highly** advised these attributes are kept in a separate annotation from any other serde attributes, due to the way
they are parsed for the payload.

The `rename` attribute is used by validify to set the field name in any errors during validation. The `with` and `deserialize_with` will be transfered to the payload field and will create a special deserialization function that will call the original and wrap the result in an option. If the custom deserializer already returns an option, it will do nothing.

## **Examples**

### **Date\[times]s**

```rust
use chrono::{NaiveDate, NaiveDateTime};

#[derive(Debug, validify::Validate)]
struct DateTimeExamples {
    #[validate(time(op = before, target = "2500-04-20", format = "%Y-%m-%d", inclusive = true))]
    before: NaiveDate,
    #[validate(time(op = before, target = "2500-04-20T12:00:00.000", format = "%Y-%m-%-dT%H:%M:%S%.3f"))]
    before_dt: NaiveDateTime,
    #[validate(time(op = after, target = "2022-04-20", format = "%Y-%m-%d"))]
    after: NaiveDate,
    #[validate(time(op = after, target = "2022-04-20T12:00:00.000", format = "%Y-%m-%-dT%H:%M:%S%.3f"))]
    after_dt: NaiveDateTime,
    #[validate(time(op = in_period, target = "2022-04-20", format = "%Y-%m-%d", weeks = -2))]
    period: NaiveDate,
}
```

### **With route handler**

```rust
    use validify::Validify;

    #[derive(Debug, Validify)]
    struct JsonTest {
        #[modify(lowercase)]
        a: String,
        #[modify(trim, uppercase)]
        #[validate(length(equal = 11))]
        b: String,
    }

    // This would normally come from a framework
    struct Json<T>(T);

    fn test() {
      let jt = JsonTest {
          a: "MODIFIED".to_string(),
          b: "    makemeshout    ".to_string(),
      };
      let json = Json(jt.into());
      mock_handler(json)
    }

    fn mock_handler(data: Json<JsonTestPayload>
    /* OR data: Json<<JsonTest as Validify>::Payload> */) {
      let data = data.0;
      let data = JsonTest::validify(data).unwrap();
      mock_service(data);
    }

    fn mock_service(data: JsonTest) {
      assert_eq!(data.a, "modified".to_string());
      assert_eq!(data.b, "MAKEMESHOUT".to_string())
    }
```

### **Big Boi**

```rust
use validify::{Validify, ValidationError, ValidationErrors, schema_validation, schema_err};
use serde::Deserialize;

const WORKING_HOURS: &[&str] = &["08", "09", "10", "11", "12", "13", "14", "15", "16"];
const CAREER_LEVEL: &[&str] = &["One", "Two", "Over 9000"];
const STATUSES: &[&str] = &["online", "offline"];
const CONTRACT_TYPES: &[&str] = &["Fulltime", "Temporary"];
const ALLOWED_MIME: &[&str] = &["jpeg", "png"];
const ALLOWED_DURATIONS: &[i32] = &[1, 2, 3];

#[derive(Clone, Deserialize, Debug, Validify)]
#[serde(rename_all = "camelCase")]
#[validate(schema_validation)]
struct BigBoi {
    #[modify(trim)]
    #[validate(length(max = 300))]
    title: String,

    #[modify(trim)]
    #[validate(is_in(STATUSES))]
    status: String,

    #[modify(capitalize, trim)]
    city_country: String,

    #[validate(length(max = 1000))]
    education: String,

    #[modify(capitalize)]
    type_of_workplace: Vec<String>,

    #[validate(is_in(WORKING_HOURS))]
    working_hours: String,

    part_time_period: Option<String>,

    #[modify(capitalize)]
    #[validate(is_in(CONTRACT_TYPES))]
    contract_type: String,

    indefinite_probation_period: bool,

    #[validate(is_in(ALLOWED_DURATIONS))]
    indefinite_probation_period_duration: Option<i32>,

    #[validate(is_in(CAREER_LEVEL))]
    career_level: String,

    #[modify(capitalize)]
    benefits: String,

    #[validate(length(max = 60))]
    meta_title: String,

    #[validate(length(max = 160))]
    meta_description: String,

    #[validate(is_in(ALLOWED_MIME))]
    meta_image: String,

    #[validate(custom(greater_than_now))]
    published_at: String,

    #[validate(custom(greater_than_now))]
    expires_at: String,

    #[validify]
    languages: Vec<TestLanguages>,

    #[validify]
    tags: TestTags,
}


#[schema_validation]
fn schema_validation(bb: &BigBoi) -> Result<(), ValidationErrors> {
    if bb.contract_type == "Fulltime" && bb.part_time_period.is_some() {
        schema_err!("Fulltime contract cannot have part time period");
    }

    if bb.contract_type == "Fulltime"
        && bb.indefinite_probation_period
        && bb.indefinite_probation_period_duration.is_none()
    {
        schema_err!(
            "No probation duration",
            "Indefinite probation duration must be specified",
        );
    }
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
                    "invalid_date",
                ))
            } else {
                Ok(())
            }
        }
        Err(e) => {
            Err(ValidationError::new_field("invalid_date"))
        }
    }
}

#[derive(Deserialize, Debug, Clone, Validify)]
#[serde(rename_all = "camelCase")]
struct TestTags {
    #[modify(trim)]
    #[validate(length(min = 1, max = 10), custom(validate_names))]
    names: Vec<String>,
}

fn validate_names(names: &[String]) -> Result<(), ValidationError> {
    for n in names.iter() {
        if n.len() > 10 || n.is_empty() {
            return Err(ValidationError::new_field(
                "invalid_name"
            ));
        }
    }
    Ok(())
}

const PROFICIENCY: &[&str] = &["dunno", "killinit"];

#[derive(Clone, Deserialize, Debug, Validify)]
#[serde(rename_all = "camelCase")]
struct TestLanguages {
    company_opening_id: String,
    #[modify(trim)]
    language: String,

    #[modify(trim)]
    #[validate(is_in(PROFICIENCY))]
    proficiency: Option<String>,

    required: Option<bool>,
    created_by: String,
}

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
        meta_description: "and it's kind of annoying".to_string(),

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
    assert!(matches!(res, Err(ref e) if e.errors().len() == 11));

    let schema_errs = res.as_ref().unwrap_err().schema_errors();
    let field_errs = res.unwrap_err().field_errors();

    assert_eq!(schema_errs.len(), 2);
    assert_eq!(field_errs.len(), 9);
}

```
