use crate::value::Set;
use crate::{Error, Value};

pub fn intersection1(left: Value) -> Result<Value, Error> {
    let left = left.try_into_set()?;
    Ok(Value::Set(left))
}

pub fn intersection2(left: Value, right: Value) -> Result<Value, Error> {
    let left = left.try_into_set()?;
    let right = right.try_into_set()?;
    Ok(Value::Set(left.intersection(&right).cloned().collect()))
}

pub fn intersection3(a: Value, b: Value, c: Value) -> Result<Value, Error> {
    let a = a.try_into_set()?;
    let b = b.try_into_set()?;
    let c = c.try_into_set()?;
    let result = a
        .intersection(&b)
        .cloned()
        .collect::<Set<Value>>()
        .intersection(&c)
        .cloned()
        .collect();
    Ok(Value::Set(result))
}

pub fn intersection4(a: Value, b: Value, c: Value, d: Value) -> Result<Value, Error> {
    let a = a.try_into_set()?;
    let b = b.try_into_set()?;
    let c = c.try_into_set()?;
    let d = d.try_into_set()?;
    let result = a
        .intersection(&b)
        .cloned()
        .collect::<Set<Value>>()
        .intersection(&c)
        .cloned()
        .collect::<Set<Value>>()
        .intersection(&d)
        .cloned()
        .collect();

    Ok(Value::Set(result))
}

pub fn union1(left: Value) -> Result<Value, Error> {
    let left = left.try_into_set()?;
    Ok(Value::Set(left))
}

pub fn union2(left: Value, right: Value) -> Result<Value, Error> {
    let left = left.try_into_set()?;
    let right = right.try_into_set()?;
    Ok(Value::Set(left.union(&right).cloned().collect()))
}

pub fn union3(a: Value, b: Value, c: Value) -> Result<Value, Error> {
    let a = a.try_into_set()?;
    let b = b.try_into_set()?;
    let c = c.try_into_set()?;
    let result = a
        .union(&b)
        .cloned()
        .collect::<Set<Value>>()
        .union(&c)
        .cloned()
        .collect();
    Ok(Value::Set(result))
}

pub fn union4(a: Value, b: Value, c: Value, d: Value) -> Result<Value, Error> {
    let a = a.try_into_set()?;
    let b = b.try_into_set()?;
    let c = c.try_into_set()?;
    let d = d.try_into_set()?;
    let result = a
        .union(&b)
        .cloned()
        .collect::<Set<Value>>()
        .union(&c)
        .cloned()
        .collect::<Set<Value>>()
        .union(&d)
        .cloned()
        .collect();

    Ok(Value::Set(result))
}
