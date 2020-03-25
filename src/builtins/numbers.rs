use crate::{Error, Value};

macro_rules! unary_op {
    ($name:ident, $op:ident) => {
        pub fn $name(val: Value) -> Result<Value, Error> {
            let v = match val {
                val if val.is_i64() => {
                    let val = val.as_i64().ok_or_else(|| Error::InvalidType("i64", val))?;
                    let result = val.$op();
                    Value::Number(result.into())
                }
                Value::Number(val) => {
                    let val = val
                        .as_f64()
                        .ok_or_else(|| Error::InvalidType("i64", val.into()))?;
                    let result = val.$op();
                    Value::Number(result.into())
                }
                val => return Err(Error::InvalidType("Number", val)),
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
                    let left = left.as_i64().ok_or_else(|| Error::InvalidType("i64", left))?;
                    let right = right.as_i64().ok_or_else(|| Error::InvalidType("i64", right))?;
                    let result = left $op right;
                    Value::Number(result.into())
                },
                (Value::Number(left), Value::Number(right)) => {
                    let left = left.as_f64().ok_or_else(|| Error::InvalidType("f64", left.into()))?;
                    let right = right.as_f64().ok_or_else(|| Error::InvalidType("f64", right.into()))?;
                    let result = left $op right;
                    Value::Number(result.into())
                },
                (a, _) => return Err(Error::InvalidType("Number", a)),
            };
            Ok(v)
        }
    );
}

unary_op!(abs, abs);

binary_op!(plus, +);
binary_op!(minus, -);
binary_op!(mul, *);
binary_op!(div, /);
binary_op!(rem, %);

pub fn round(val: Value) -> Result<Value, Error> {
    let v = match val {
        val if val.is_i64() => {
            let val = val.as_i64().ok_or_else(|| Error::InvalidType("i64", val))?;
            Value::Number(val.into())
        }
        Value::Number(val) => {
            let val = val
                .as_f64()
                .ok_or_else(|| Error::InvalidType("i64", val.into()))?;
            let result = val.round();
            Value::Number(result.into())
        }
        val => return Err(Error::InvalidType("Number", val)),
    };
    Ok(v)
}
