use rocket::form::{FromFormField, ValueField};
use serde::Serialize;

#[derive(Serialize)]
pub struct MultipleOf<const N: i32>(i32);

impl<const N: i32> MultipleOf<N> {
    pub fn value(&self) -> i32 {
        self.0
    }
}

impl<const N: i32> TryFrom<i32> for MultipleOf<N> {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value % N != 0 {
            Err(())
        } else {
            Ok(Self(value))
        }
    }
}

impl<'r, const N: i32> FromFormField<'r> for MultipleOf<N> {
    fn from_value(value: ValueField<'r>) -> rocket::form::Result<'r, Self> {
        let x: i32 = value.value.parse()?;

        x.try_into()
            .map_err(|_| rocket::form::Error::validation(format!("not a multiple of {N}")).into())
    }
}
