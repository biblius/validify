# Changelog

- ## 1.0.12

- All error params now represent the violating field's value with `actual` and the expected value as `target` if applicable.
- Remove params in errors where validation failed for whole structs and collections due to the error size being pretty massive
- Due to the previous change, structs deriving `Validify` no longer have to implement `Serialize` as all the info is contained in the field and location of the error.
- Remove redundant params from `required` errors.

- ## 1.0.11

- Serde attributes are now propagated to the payload

- ## 1.0.0

- Attributes now follow rust conventional syntax
- `contains` and `is_in` can now be applied to any type
- Added simple time validation
- Most validators can now directly specify a path instead of string literals
- Errors now have a location similar to a JSON pointer
- Nested validifies now result in payload structs to also contain payload versions of their children.
