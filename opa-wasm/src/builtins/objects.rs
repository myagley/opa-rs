use crate::value::Map;
use crate::{Error, Value};

pub fn get(object: Value, key: Value, default: Value) -> Result<Value, Error> {
    let mut object = object.try_into_object()?;
    let key = key.try_into_string()?;
    let v = object.remove(&key).unwrap_or(default);
    Ok(v)
}

pub fn remove(object: Value, keys: Value) -> Result<Value, Error> {
    let object = object.try_into_object()?;
    match keys {
        Value::Array(v) => remove_all(object, v.into_iter()),
        Value::Set(v) => remove_all(object, v.into_iter()),
        Value::Object(v) => remove_all(object, v.into_iter().map(|(k, _v)| Value::String(k))),
        v => Err(Error::InvalidType("iterator of strings", v)),
    }
}

fn remove_all<I>(mut map: Map<String, Value>, iter: I) -> Result<Value, Error>
where
    I: Iterator<Item = Value>,
{
    for key in iter {
        map.remove(&key.try_into_string()?);
    }
    Ok(map.into())
}
