use rocket::form::{FromFormField, ValueField};
pub struct MultipleOf<const N: i32>(i32);

impl<const N: i32> MultipleOf<N> {
    pub fn value(&self) -> i32 {
        self.0
    }
}

impl<'r, const N: i32> FromFormField<'r> for MultipleOf<N> {
    fn from_value(value: ValueField<'r>) -> rocket::form::Result<'r, Self> {
        let x = value.value.parse()?;
        if x % N != 0 {
            Err(rocket::form::Error::validation(format!("not a multiple of {N}")).into())
        } else {
            Ok(Self(x))
        }
    }
}
