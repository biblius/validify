#![allow(unused)]

use serde::Deserialize;
use validify::Validify;

// Here as a compilation test

enum Error {}

type Result<T> = std::result::Result<T, Error>;

#[derive(Deserialize, Validify)]
pub struct Test {
    #[modify(trim)]
    #[validate(length(equal = 11))]
    phone: String,
}
