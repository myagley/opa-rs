use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use lazy_static::lazy_static;
use wasmtime::Memory;

use crate::{dump_json, load_json, Error, Functions, Value, ValueAddr};

mod aggregates;
mod arrays;
mod numbers;
mod objects;
mod sets;

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
        b.insert("all", aggregates::all);
        b.insert("any", aggregates::any);
        b.insert("count", aggregates::count);
        b.insert("max", aggregates::max);
        b.insert("min", aggregates::min);
        b.insert("product", aggregates::product);
        b.insert("sort", aggregates::sort);
        b.insert("sum", aggregates::sum);

        b.insert("abs", numbers::abs);
        b.insert("round", numbers::round);

        b.insert("intersection", sets::intersection1);
        b.insert("union", sets::union1);
        b
    };
    static ref BUILTIN2: HashMap<&'static str, Arity2> = {
        let mut b: HashMap<&'static str, Arity2> = HashMap::new();
        b.insert("array.concat", arrays::concat);

        b.insert("plus", numbers::plus);
        b.insert("minus", numbers::minus);
        b.insert("mul", numbers::mul);
        b.insert("div", numbers::div);
        b.insert("rem", numbers::rem);

        b.insert("object.remove", objects::remove);

        b.insert("and", sets::intersection2);
        b.insert("or", sets::union2);
        b
    };
    static ref BUILTIN3: HashMap<&'static str, Arity3> = {
        let mut b: HashMap<&'static str, Arity3> = HashMap::new();
        b.insert("array.slice", arrays::slice);

        b.insert("object.get", objects::get);

        b.insert("intersection", sets::intersection3);
        b.insert("union", sets::union3);
        b
    };
    static ref BUILTIN4: HashMap<&'static str, Arity4> = {
        let mut b: HashMap<&'static str, Arity4> = HashMap::new();
        b.insert("intersection", sets::intersection4);
        b.insert("union", sets::union4);
        b
    };
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

        let mut lookup = HashMap::new();
        for (k, v) in val.try_into_object()?.into_iter() {
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
