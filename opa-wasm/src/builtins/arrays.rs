use std::cmp;

use crate::{Error, Value};

pub fn concat(left: Value, right: Value) -> Result<Value, Error> {
    let mut left = left.try_into_array()?;
    let mut right = right.try_into_array()?;
    left.append(&mut right);
    Ok(Value::Array(left))
}

pub fn slice(val: Value, start: Value, end: Value) -> Result<Value, Error> {
    let array = val.try_into_array()?;
    let start = start.try_into_i64()?;
    let end = end.try_into_i64()?;

    let v = if start >= end || (start < 0 && end < 0) {
        Value::Array(vec![])
    } else {
        let len = array.len();
        let start = cmp::min(cmp::max(start, 0) as usize, len);
        let end = cmp::min(cmp::max(end, 0) as usize, len);
        Value::Array(array[start..end].into())
    };

    Ok(v)
}
