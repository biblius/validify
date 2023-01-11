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
let res = Testor::validate(test.into());

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

Every struct annotated with `#[validify]` gets an associated payload struct. E.g.

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
#[derive(Debug, Clone, Serialize, Deserialize, validator::Validate)]
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

Validify's `validate` method always takes in the generated payload and outputs the original struct if all validations have passed.

The macro automatically implements validator's `Validate` trait and validify's `Modify` trait in the wrapper trait `Validify`. This wrapper trait contains only the method `validate` which in the above example expands to:

```rust
    fn validate(payload: Self::Payload) -> Result<(), ValidationErrors> {
        <Self::Payload as ::validator::Validate>::validate(&payload)?;
        let mut this = Self::from(payload);
        let mut errors: Vec<::validify::ValidationErrors> = Vec::new();
        if let Err(e) = <Nestor as ::validify::Validify>::validate(this.nested.clone().into()) {
            errors.push(e.into());
        }
        <Self as ::validify::Modify>::modify(&mut this);
        if let Err(e) = <Self as ::validator::Validate>::validate(&this) {
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

An example with a mock handler with actix:

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
      let data = JsonTest::validate(data).unwrap();
      mock_service(data);
    }

    fn mock_service(data: JsonTest) {
      assert_eq!(data.a, "modified".to_string());
      assert_eq!(data.b, "MAKEMESHOUT".to_string())
    }
```
