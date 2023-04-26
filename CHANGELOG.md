# Changelog

- ## 1.0.11

- Serde attributes are now propagated to the payload

- ## 1.0.0

- Attributes now follow rust conventional syntax
- `contains` and `is_in` can now be applied to any type
- Added simple time validation
- Most validators can now directly specify a path instead of string literals
- Errors now have a location similar to a JSON pointer
- Nested validifies now result in payload structs to also contain payload versions of their children.
