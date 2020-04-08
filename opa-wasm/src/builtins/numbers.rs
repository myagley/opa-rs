use crate::{Error, Value};

macro_rules! unary_op {
    ($name:ident, $op:ident) => {
        pub fn $name(val: Value) -> Result<Value, Error> {
            let v = match val {
                val if val.is_i64() => {
                    let val = val.try_into_i64()?;
                    let result = val.$op();
                    Value::Number(result.into())
                }
                Value::Number(val) => {
                    let val = val.try_into_f64()?;
                    let result = val.$op();
                    Value::Number(result.into())
                }
                val => return Err(Error::InvalidType("number", val)),
            };
            Ok(v)
        }
    };
}

macro_rules! binary_op {
    ($name:ident, $op:tt) => (
        pub fn $name(left: Value, right: Value) -> Result<Value, Error> {
            let v = match (left, right) {
                (left, right) if left.is_i64() && right.is_i64() => {
                    let left = left.try_into_i64()?;
                    let right = right.try_into_i64()?;
                    let result = left $op right;
                    Value::Number(result.into())
                },
                (Value::Number(left), Value::Number(right)) => {
                    let left = left.try_into_f64()?;
                    let right = right.try_into_f64()?;
                    let result = left $op right;
                    Value::Number(result.into())
                },
                (a, _) => return Err(Error::InvalidType("number", a)),
            };
            Ok(v)
        }
    );
}

macro_rules! binary_op_func {
    ($name:ident, $op:tt) => {
        pub fn $name(left: Value, right: Value) -> Result<Value, Error> {
            let v = match (left, right) {
                (left, right) if left.is_i64() && right.is_i64() => {
                    let left = left.try_into_i64()?;
                    let right = right.try_into_i64()?;
                    let result = left.$op(right);
                    Value::Number(result.into())
                }
                (Value::Number(left), Value::Number(right)) => {
                    let left = left.try_into_f64()?;
                    let right = right.try_into_f64()?;
                    let result = left.$op(right);
                    Value::Number(result.into())
                }
                (a, _) => return Err(Error::InvalidType("number", a)),
            };
            Ok(v)
        }
    };
}

unary_op!(abs, abs);

binary_op!(plus, +);
binary_op!(mul, *);
binary_op!(div, /);
binary_op!(rem, %);

binary_op_func!(min, min);
binary_op_func!(max, max);

pub fn minus(left: Value, right: Value) -> Result<Value, Error> {
    let v = match (left, right) {
        (left, right) if left.is_i64() && right.is_i64() => {
            let left = left.try_into_i64()?;
            let right = right.try_into_i64()?;
            let result = left - right;
            Value::Number(result.into())
        }
        (Value::Number(left), Value::Number(right)) => {
            let left = left.try_into_f64()?;
            let right = right.try_into_f64()?;
            let result = left - right;
            Value::Number(result.into())
        }
        (Value::Set(left), Value::Set(right)) => {
            Value::Set(left.difference(&right).cloned().collect())
        }
        (a, _) => return Err(Error::InvalidType("number", a)),
    };
    Ok(v)
}

pub fn round(val: Value) -> Result<Value, Error> {
    let v = match val {
        val if val.is_i64() => {
            let val = val.try_into_i64()?;
            Value::Number(val.into())
        }
        Value::Number(val) => {
            let val = val.try_into_f64()?;
            let result = val.round();
            Value::Number(result.into())
        }
        val => return Err(Error::InvalidType("Number", val)),
    };
    Ok(v)
}
