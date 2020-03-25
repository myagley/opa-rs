use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

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

    fn builtin0(&self, _id: i32, _ctx_addr: ValueAddr) -> ValueAddr {
        println!("builtin0");
        ValueAddr(0)
    }

    fn builtin1(&self, id: i32, _ctx_addr: ValueAddr, value: ValueAddr) -> ValueAddr {
        if let Some(f) = self.lookup.get(&id) {
            println!("func: {}", f);
        }

        let val = btry!(dump_json(&self.functions, &self.memory, value)
            .and_then(|s| serde_json::from_str(&s).map_err(Error::DeserializeJson)));

        let result = count(&val);

        let serialized = btry!(serde_json::to_string(&result));
        btry!(load_json(&self.functions, &self.memory, &serialized))
    }

    fn builtin2(&self, id: i32, _ctx_addr: ValueAddr, a: ValueAddr, b: ValueAddr) -> ValueAddr {
        if let Some(f) = self.lookup.get(&id) {
            println!("func: {}", f);
        }

        let val1: Value = btry!(dump_json(&self.functions, &self.memory, a)
            .and_then(|s| serde_json::from_str(&s).map_err(Error::DeserializeJson)));
        let val2: Value = btry!(dump_json(&self.functions, &self.memory, b)
            .and_then(|s| serde_json::from_str(&s).map_err(Error::DeserializeJson)));

        let num1 = btry!(val1
            .as_i64()
            .ok_or_else(|| Error::InvalidType("Number", val1)));
        let num2 = btry!(val2
            .as_i64()
            .ok_or_else(|| Error::InvalidType("Number", val2)));
        let sum = num1 + num2;
        let serialized = btry!(serde_json::to_string(&Value::Number(sum.into())));
        btry!(load_json(&self.functions, &self.memory, &serialized))
    }

    fn builtin3(
        &self,
        _id: i32,
        _ctx_addr: ValueAddr,
        _a: ValueAddr,
        _b: ValueAddr,
        _c: ValueAddr,
    ) -> ValueAddr {
        println!("builtin3");
        ValueAddr(0)
    }

    fn builtin4(
        &self,
        _id: i32,
        _ctx_addr: ValueAddr,
        _a: ValueAddr,
        _b: ValueAddr,
        _c: ValueAddr,
        _d: ValueAddr,
    ) -> ValueAddr {
        println!("builtin4");
        ValueAddr(0)
    }
}

fn count(a: &Value) -> Value {
    match a {
        Value::Array(ref v) => Value::Number(v.len().into()),
        Value::Object(ref v) => Value::Number(v.len().into()),
        Value::Set(ref v) => Value::Number(v.len().into()),
        _ => Value::Null,
    }
}
