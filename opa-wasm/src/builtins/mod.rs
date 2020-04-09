use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use lazy_static::lazy_static;
use tracing::{debug, error};

use crate::runtime::Instance;
use crate::{opa_serde, Error, Value, ValueAddr};

mod aggregates;
mod arrays;
mod net;
mod numbers;
mod objects;
mod regex;
mod sets;
mod strings;
mod time;
mod types;

macro_rules! btry {
    ($expr:expr) => {
        match $expr {
            ::std::result::Result::Ok(val) => val,
            ::std::result::Result::Err(err) => {
                error!(msg = "error processing builtin function", error = %err);
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
    static ref BUILTIN0: HashMap<&'static str, Arity0> = {
        let mut b: HashMap<&'static str, Arity0> = HashMap::new();
        b.insert("time.now_ns", time::now_ns);
        b
    };
    static ref BUILTIN1: HashMap<&'static str, Arity1> = {
        let mut b: HashMap<&'static str, Arity1> = HashMap::new();
        b.insert("trace", trace);

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

        b.insert("net.cidr_expand", net::cidr_expand);

        b.insert("upper", strings::upper);

        b.insert("time.clock", time::clock);
        b.insert("time.date", time::date);
        b.insert("time.parse_rfc3339_ns", time::parse_rfc3339_ns);
        b.insert("time.weekday", time::weekday);

        b.insert("is_array", types::is_array);
        b.insert("is_boolean", types::is_boolean);
        b.insert("is_null", types::is_null);
        b.insert("is_number", types::is_number);
        b.insert("is_object", types::is_object);
        b.insert("is_set", types::is_set);
        b.insert("is_string", types::is_string);
        b.insert("type_name", types::type_name);
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

        b.insert("net.cidr_contains", net::cidr_contains);
        b.insert("net.cidr_intersects", net::cidr_intersects);

        b.insert("object.remove", objects::remove);

        b.insert("re_match", regex::re_match);

        b.insert("and", sets::and);
        b.insert("or", sets::or);
        b
    };
    static ref BUILTIN3: HashMap<&'static str, Arity3> = {
        let mut b: HashMap<&'static str, Arity3> = HashMap::new();
        b.insert("array.slice", arrays::slice);

        b.insert("object.get", objects::get);
        b
    };
    static ref BUILTIN4: HashMap<&'static str, Arity4> = {
        let b: HashMap<&'static str, Arity4> = HashMap::new();
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

#[derive(Clone, Debug, Default)]
pub struct Builtins {
    inner: Arc<RefCell<Option<Inner>>>,
}

impl Builtins {
    pub fn replace(&self, instance: Instance) -> Result<(), Error> {
        let inner = Inner::new(instance)?;
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

#[derive(Debug)]
struct Inner {
    instance: Instance,
    lookup: HashMap<i32, String>,
}

impl Inner {
    fn new(instance: Instance) -> Result<Self, Error> {
        let builtins_addr = instance.functions().builtins()?;
        let val: Value = opa_serde::from_instance(&instance, builtins_addr)?;

        let mut lookup = HashMap::new();
        for (k, v) in val.try_into_object()?.into_iter() {
            if !BUILTIN_NAMES.contains(k.as_str()) {
                return Err(Error::UnknownBuiltin(k));
            }
            let v = v.try_into_i64()?;
            lookup.insert(v as i32, k);
        }

        let inner = Inner { instance, lookup };
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
        debug!(name = %name, arity = 0, "calling builtin function...");
        let result = btry!(func());
        debug!(name = %name, arity = 0, result = ?result, "called builtin function.");

        btry!(opa_serde::to_instance(&self.instance, &result))
    }

    fn builtin1(&self, id: i32, _ctx_addr: ValueAddr, value: ValueAddr) -> ValueAddr {
        let name = btry!(self
            .lookup
            .get(&id)
            .ok_or_else(|| Error::UnknownBuiltinId(id)));
        let func = btry!(BUILTIN1
            .get(name.as_str())
            .ok_or_else(|| Error::UnknownBuiltin(name.to_string())));

        let val = btry!(opa_serde::from_instance(&self.instance, value));

        debug!(name = %name, arity = 1, arg0 = ?val, "calling builtin function...");
        let result = btry!(func(val));
        debug!(name = %name, arity = 1, result = ?result, "called builtin function.");

        btry!(opa_serde::to_instance(&self.instance, &result))
    }

    fn builtin2(&self, id: i32, _ctx_addr: ValueAddr, a: ValueAddr, b: ValueAddr) -> ValueAddr {
        let name = btry!(self
            .lookup
            .get(&id)
            .ok_or_else(|| Error::UnknownBuiltinId(id)));
        let func = btry!(BUILTIN2
            .get(name.as_str())
            .ok_or_else(|| Error::UnknownBuiltin(name.to_string())));

        let val1 = btry!(opa_serde::from_instance(&self.instance, a));
        let val2 = btry!(opa_serde::from_instance(&self.instance, b));

        debug!(name = %name, arity = 2, arg0 = ?val1, arg1 = ?val2, "calling builtin function...");
        let result = btry!(func(val1, val2));
        debug!(name = %name, arity = 2, result = ?result, "called builtin function.");

        btry!(opa_serde::to_instance(&self.instance, &result))
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

        let val1 = btry!(opa_serde::from_instance(&self.instance, a));
        let val2 = btry!(opa_serde::from_instance(&self.instance, b));
        let val3 = btry!(opa_serde::from_instance(&self.instance, c));

        debug!(name = %name, arity = 3, arg0 = ?val1, arg1 = ?val2, arg2 = ?val3, "calling builtin function...");
        let result = btry!(func(val1, val2, val3));
        debug!(name = %name, arity = 3, result = ?result, "called builtin function.");

        btry!(opa_serde::to_instance(&self.instance, &result))
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

        let val1 = btry!(opa_serde::from_instance(&self.instance, a));
        let val2 = btry!(opa_serde::from_instance(&self.instance, b));
        let val3 = btry!(opa_serde::from_instance(&self.instance, c));
        let val4 = btry!(opa_serde::from_instance(&self.instance, d));

        debug!(name = %name, arity = 4, arg0 = ?val1, arg1 = ?val2, arg2 = ?val3, arg3 = ?val4, "calling builtin function...");
        let result = btry!(func(val1, val2, val3, val4));
        debug!(name = %name, arity = 4, result = ?result, "called builtin function.");

        btry!(opa_serde::to_instance(&self.instance, &result))
    }
}

fn trace(value: Value) -> Result<Value, Error> {
    debug!("TRACE: {:?}", value);
    value.try_into_string().map(|_| true.into())
}
