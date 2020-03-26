use crate::{Error, Value};

macro_rules! is_func {
    ($func:ident) => {
        pub fn $func(val: Value) -> Result<Value, Error> {
            Ok(val.$func().into())
        }
    };
}

is_func!(is_number);
is_func!(is_string);
is_func!(is_boolean);
is_func!(is_array);
is_func!(is_set);
is_func!(is_object);
is_func!(is_null);

pub fn type_name(val: Value) -> Result<Value, Error> {
    let v = match val {
        Value::Null => Value::String("null".to_string()),
        Value::Bool(_) => Value::String("bool".to_string()),
        Value::Number(_) => Value::String("number".to_string()),
        Value::String(_) => Value::String("string".to_string()),
        Value::Array(_) => Value::String("array".to_string()),
        Value::Object(_) => Value::String("object".to_string()),
        Value::Set(_) => Value::String("set".to_string()),
    };
    Ok(v)
}
