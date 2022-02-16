use rocket::form::{FromFormField, ValueField};
use rocket_okapi::okapi::schemars::{self, JsonSchema};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MultipleOfError {
    #[error("Not a multiple of {n}")]
    NotAMultipleOf { n: i32 },
}

#[derive(Serialize, JsonSchema)]
pub struct MultipleOf<const N: i32>(i32);

impl<const N: i32> MultipleOf<N> {
    pub fn value(&self) -> i32 {
        self.0
    }
}

impl<const N: i32> TryFrom<i32> for MultipleOf<N> {
    type Error = MultipleOfError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value % N != 0 {
            Err(MultipleOfError::NotAMultipleOf { n: N })
        } else {
            Ok(Self(value))
        }
    }
}

impl<'r, const N: i32> FromFormField<'r> for MultipleOf<N> {
    fn from_value(value: ValueField<'r>) -> rocket::form::Result<'r, Self> {
        let x: i32 = value.value.parse()?;

        x.try_into()
            .map_err(|x: MultipleOfError| rocket::form::error::Error::custom(x).into())
    }
}
