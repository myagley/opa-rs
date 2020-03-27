use crate::{Error, Value};

pub fn and(left: Value, right: Value) -> Result<Value, Error> {
    let left = left.try_into_set()?;
    let right = right.try_into_set()?;
    Ok(Value::Set(left.intersection(&right).cloned().collect()))
}

pub fn or(left: Value, right: Value) -> Result<Value, Error> {
    let left = left.try_into_set()?;
    let right = right.try_into_set()?;
    Ok(Value::Set(left.union(&right).cloned().collect()))
}
