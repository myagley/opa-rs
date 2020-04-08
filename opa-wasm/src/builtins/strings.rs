use crate::{Error, Value};

pub fn upper(string: Value) -> Result<Value, Error> {
    let s = string.try_into_string()?;
    Ok(Value::String(s.to_uppercase()))
}
