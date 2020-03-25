use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use lazy_static::lazy_static;
use wasmtime::Memory;

use crate::{dump_json, load_json, Error, Functions, Value, ValueAddr};

macro_rules! btry {
    ($expr:expr) => {
        match $expr {
            ::std::result::Result::Ok(val) => val,
            ::std::result::Result::Err(err) => {
                println!("builtin error: {}", err);
                return ValueAddr(0);
            }
        }
    };
}

type Arity0 = fn() -> Result<Value, Error>;
type Arity1 = fn(Value) -> Result<Value, Error>;
type Arity2 = fn(Value, Value) -> Result<Value, Error>;
type Arity3 = fn(Value, Value, Value) -> Result<Value, Error>;
type Arity4 = fn(Value, Value, Value, Value) -> Result<Value, Error>;

lazy_static! {
    static ref BUILTIN0: HashMap<&'static str, Arity0> = { HashMap::new() };
    static ref BUILTIN1: HashMap<&'static str, Arity1> = {
        let mut b: HashMap<&'static str, Arity1> = HashMap::new();
        b.insert("count", count);
        b
    };
    static ref BUILTIN2: HashMap<&'static str, Arity2> = {
        let mut b: HashMap<&'static str, Arity2> = HashMap::new();
        b.insert("plus", plus);
        b
    };
    static ref BUILTIN3: HashMap<&'static str, Arity3> = { HashMap::new() };
    static ref BUILTIN4: HashMap<&'static str, Arity4> = { HashMap::new() };
    static ref BUILTIN_NAMES: HashSet<&'static str> = {
        BUILTIN0
            .keys()
            .chain(BUILTIN1.keys())
            .chain(BUILTIN2.keys())
            .chain(BUILTIN3.keys())
            .chain(BUILTIN4.keys())
            .map(|k| *k)
            .collect::<HashSet<&'static str>>()
    };
}

#[derive(Clone, Default)]
pub struct Builtins {
    inner: Arc<RefCell<Option<Inner>>>,
}

impl Builtins {
    pub fn replace(&self, functions: Functions, memory: Memory) -> Result<(), Error> {
        let inner = Inner::new(functions, memory)?;
        self.inner.replace(Some(inner));
        Ok(())
    }

    pub fn builtin0(&self, id: i32, ctx_addr: ValueAddr) -> ValueAddr {
        let maybe_inner = self.inner.borrow();
        let inner = btry!(maybe_inner.as_ref().ok_or(Error::Initialization));
        inner.builtin0(id, ctx_addr)
    }

    pub fn builtin1(&self, id: i32, ctx_addr: ValueAddr, value: ValueAddr) -> ValueAddr {
        let maybe_inner = self.inner.borrow();
        let inner = btry!(maybe_inner.as_ref().ok_or(Error::Initialization));
        inner.builtin1(id, ctx_addr, value)
    }

    pub fn builtin2(&self, id: i32, ctx_addr: ValueAddr, a: ValueAddr, b: ValueAddr) -> ValueAddr {
        let maybe_inner = self.inner.borrow();
        let inner = btry!(maybe_inner.as_ref().ok_or(Error::Initialization));
        inner.builtin2(id, ctx_addr, a, b)
    }

    pub fn builtin3(
        &self,
        id: i32,
        ctx_addr: ValueAddr,
        a: ValueAddr,
        b: ValueAddr,
        c: ValueAddr,
    ) -> ValueAddr {
        let maybe_inner = self.inner.borrow();
        let inner = btry!(maybe_inner.as_ref().ok_or(Error::Initialization));
        inner.builtin3(id, ctx_addr, a, b, c)
    }

    pub fn builtin4(
        &self,
        id: i32,
        ctx_addr: ValueAddr,
        a: ValueAddr,
        b: ValueAddr,
        c: ValueAddr,
        d: ValueAddr,
    ) -> ValueAddr {
        let maybe_inner = self.inner.borrow();
        let inner = btry!(maybe_inner.as_ref().ok_or(Error::Initialization));
        inner.builtin4(id, ctx_addr, a, b, c, d)
    }
}

struct Inner {
    functions: Functions,
    memory: Memory,
    lookup: HashMap<i32, String>,
}

impl Inner {
    fn new(functions: Functions, memory: Memory) -> Result<Self, Error> {
        let builtins_addr = functions.builtins()?;
        let val: Value = dump_json(&functions, &memory, builtins_addr)
            .and_then(|s| serde_json::from_str(&s).map_err(Error::DeserializeJson))?;

        if !val.is_object() {
            return Err(Error::InvalidType("Object", val));
        }

        let mut lookup = HashMap::new();
        for (k, v) in val.into_object().expect("invalid obj check").into_iter() {
            if !BUILTIN_NAMES.contains(k.as_str()) {
                return Err(Error::UnknownBuiltin(k));
            }

            if !v.is_i64() {
                return Err(Error::InvalidType("Number", v));
            }
            lookup.insert(v.as_i64().expect("invalid i64 check") as i32, k);
        }

        let inner = Inner {
            functions,
            memory,
            lookup,
        };
        Ok(inner)
    }

    fn builtin0(&self, id: i32, _ctx_addr: ValueAddr) -> ValueAddr {
        let name = btry!(self
            .lookup
            .get(&id)
            .ok_or_else(|| Error::UnknownBuiltinId(id)));
        let func = btry!(BUILTIN0
            .get(name.as_str())
            .ok_or_else(|| Error::UnknownBuiltin(name.to_string())));
        let result = btry!(func());

        let serialized = btry!(serde_json::to_string(&result));
        btry!(load_json(&self.functions, &self.memory, &serialized))
    }

    fn builtin1(&self, id: i32, _ctx_addr: ValueAddr, value: ValueAddr) -> ValueAddr {
        let name = btry!(self
            .lookup
            .get(&id)
            .ok_or_else(|| Error::UnknownBuiltinId(id)));
        let func = btry!(BUILTIN1
            .get(name.as_str())
            .ok_or_else(|| Error::UnknownBuiltin(name.to_string())));

        let val = btry!(dump_json(&self.functions, &self.memory, value)
            .and_then(|s| serde_json::from_str(&s).map_err(Error::DeserializeJson)));

        let result = btry!(func(val));

        let serialized = btry!(serde_json::to_string(&result));
        btry!(load_json(&self.functions, &self.memory, &serialized))
    }

    fn builtin2(&self, id: i32, _ctx_addr: ValueAddr, a: ValueAddr, b: ValueAddr) -> ValueAddr {
        let name = btry!(self
            .lookup
            .get(&id)
            .ok_or_else(|| Error::UnknownBuiltinId(id)));
        let func = btry!(BUILTIN2
            .get(name.as_str())
            .ok_or_else(|| Error::UnknownBuiltin(name.to_string())));

        let val1 = btry!(dump_json(&self.functions, &self.memory, a)
            .and_then(|s| serde_json::from_str(&s).map_err(Error::DeserializeJson)));
        let val2 = btry!(dump_json(&self.functions, &self.memory, b)
            .and_then(|s| serde_json::from_str(&s).map_err(Error::DeserializeJson)));
        let result = btry!(func(val1, val2));

        let serialized = btry!(serde_json::to_string(&result));
        btry!(load_json(&self.functions, &self.memory, &serialized))
    }

    fn builtin3(
        &self,
        id: i32,
        _ctx_addr: ValueAddr,
        a: ValueAddr,
        b: ValueAddr,
        c: ValueAddr,
    ) -> ValueAddr {
        let name = btry!(self
            .lookup
            .get(&id)
            .ok_or_else(|| Error::UnknownBuiltinId(id)));
        let func = btry!(BUILTIN3
            .get(name.as_str())
            .ok_or_else(|| Error::UnknownBuiltin(name.to_string())));

        let val1 = btry!(dump_json(&self.functions, &self.memory, a)
            .and_then(|s| serde_json::from_str(&s).map_err(Error::DeserializeJson)));
        let val2 = btry!(dump_json(&self.functions, &self.memory, b)
            .and_then(|s| serde_json::from_str(&s).map_err(Error::DeserializeJson)));
        let val3 = btry!(dump_json(&self.functions, &self.memory, c)
            .and_then(|s| serde_json::from_str(&s).map_err(Error::DeserializeJson)));
        let result = btry!(func(val1, val2, val3));

        let serialized = btry!(serde_json::to_string(&result));
        btry!(load_json(&self.functions, &self.memory, &serialized))
    }

    fn builtin4(
        &self,
        id: i32,
        _ctx_addr: ValueAddr,
        a: ValueAddr,
        b: ValueAddr,
        c: ValueAddr,
        d: ValueAddr,
    ) -> ValueAddr {
        let name = btry!(self
            .lookup
            .get(&id)
            .ok_or_else(|| Error::UnknownBuiltinId(id)));
        let func = btry!(BUILTIN4
            .get(name.as_str())
            .ok_or_else(|| Error::UnknownBuiltin(name.to_string())));

        let val1 = btry!(dump_json(&self.functions, &self.memory, a)
            .and_then(|s| serde_json::from_str(&s).map_err(Error::DeserializeJson)));
        let val2 = btry!(dump_json(&self.functions, &self.memory, b)
            .and_then(|s| serde_json::from_str(&s).map_err(Error::DeserializeJson)));
        let val3 = btry!(dump_json(&self.functions, &self.memory, c)
            .and_then(|s| serde_json::from_str(&s).map_err(Error::DeserializeJson)));
        let val4 = btry!(dump_json(&self.functions, &self.memory, d)
            .and_then(|s| serde_json::from_str(&s).map_err(Error::DeserializeJson)));
        let result = btry!(func(val1, val2, val3, val4));

        let serialized = btry!(serde_json::to_string(&result));
        btry!(load_json(&self.functions, &self.memory, &serialized))
    }
}

fn count(a: Value) -> Result<Value, Error> {
    let v = match a {
        Value::Array(ref v) => Value::Number(v.len().into()),
        Value::Object(ref v) => Value::Number(v.len().into()),
        Value::Set(ref v) => Value::Number(v.len().into()),
        Value::String(ref v) => Value::Number(v.len().into()),
        _ => Value::Null,
    };
    Ok(v)
}

fn plus(a: Value, b: Value) -> Result<Value, Error> {
    let num1 = a.as_i64().ok_or_else(|| Error::InvalidType("Number", a))?;
    let num2 = b.as_i64().ok_or_else(|| Error::InvalidType("Number", b))?;
    let sum = num1 + num2;
    Ok(Value::Number(sum.into()))
}
