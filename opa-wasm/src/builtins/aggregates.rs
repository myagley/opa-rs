use crate::builtins::numbers;
use crate::{Error, Value};

pub fn count(val: Value) -> Result<Value, Error> {
    let v = match val {
        Value::Array(ref v) => Value::Number(v.len().into()),
        Value::Object(ref v) => Value::Number(v.len().into()),
        Value::Set(ref v) => Value::Number(v.len().into()),
        Value::String(ref v) => Value::Number(v.len().into()),
        val => return Err(Error::InvalidType("collection_or_string", val)),
    };
    Ok(v)
}

macro_rules! binary_loop {
    ($name:ident, $init:expr, $op:path) => {
        pub fn $name(val: Value) -> Result<Value, Error> {
            fn do_op<I: Iterator<Item = Value>>(iter: I) -> Result<Value, Error> {
                iter.fold(Ok($init.into()), |acc, v| match acc {
                    Ok(acc) => $op(acc, v),
                    e => e,
                })
            }

            let v = match val {
                Value::Array(v) => do_op(v.into_iter())?,
                Value::Set(v) => do_op(v.into_iter())?,
                val => return Err(Error::InvalidType("collection_or_string", val)),
            };
            Ok(v)
        }
    };
}

binary_loop!(sum, 0, numbers::plus);
binary_loop!(product, 1, numbers::mul);
binary_loop!(min, std::f64::MAX, numbers::min);
binary_loop!(max, std::f64::MIN, numbers::max);
binary_loop!(all, true, for_all);
binary_loop!(any, false, for_any);

fn for_all(left: Value, right: Value) -> Result<Value, Error> {
    if let (Some(l), Some(r)) = (left.as_bool(), right.as_bool()) {
        Ok(Value::Bool(l && r))
    } else {
        Ok(Value::Bool(false))
    }
}

fn for_any(left: Value, right: Value) -> Result<Value, Error> {
    if let (Some(l), Some(r)) = (left.as_bool(), right.as_bool()) {
        Ok(Value::Bool(l || r))
    } else {
        Ok(Value::Bool(false))
    }
}

pub fn sort(val: Value) -> Result<Value, Error> {
    let v = match val {
        Value::Array(mut v) => {
            v.sort();
            Value::Array(v)
        }
        Value::Set(v) => {
            let mut vec = v.into_iter().collect::<Vec<Value>>();
            vec.sort();
            Value::Array(vec)
        }
        val => return Err(Error::InvalidType("collection_or_string", val)),
    };
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum() {
        let v: &[u8] = &[1, 2, 3];
        let out = sum(v.into()).unwrap();
        let expected: Value = 6_u8.into();
        assert_eq!(expected, out);

        let v: &[Value] = &[Value::Number(1.into()), Value::String("3".to_string())];
        let out = sum(v.into());
        assert!(out.is_err());
    }

    #[test]
    fn test_product() {
        let v: &[u8] = &[1, 2, 3, 4];
        let out = product(v.into()).unwrap();
        let expected: Value = 24_u8.into();
        assert_eq!(expected, out);

        let v: &[Value] = &[Value::Number(1.into()), Value::String("3".to_string())];
        let out = product(v.into());
        assert!(out.is_err());
    }
}
