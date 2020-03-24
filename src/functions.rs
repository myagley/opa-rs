use std::cell::RefCell;
use std::fmt;
use std::sync::Arc;

use wasmtime::{Instance, Trap};

use crate::{Error, ValueAddr};

#[derive(Clone, Default)]
pub struct Functions {
    inner: Arc<RefCell<Option<Inner>>>,
}

impl Functions {
    pub fn replace(&self, instance: Instance) -> Result<(), Error> {
        let inner = Inner::from_instance(instance)?;
        self.inner.replace(Some(inner));
        Ok(())
    }

    pub fn builtins(&self) -> Result<ValueAddr, Error> {
        let maybe_inner = self.inner.borrow();
        let inner = maybe_inner.as_ref().ok_or(Error::Initialization)?;
        let addr = inner.builtins()?;
        Ok(addr.into())
    }

    pub fn eval_ctx_new(&self) -> Result<ValueAddr, Error> {
        let maybe_inner = self.inner.borrow();
        let inner = maybe_inner.as_ref().ok_or(Error::Initialization)?;
        let addr = inner.eval_ctx_new()?;
        Ok(addr.into())
    }

    pub fn eval_ctx_set_input(&self, ctx: ValueAddr, input: ValueAddr) -> Result<(), Error> {
        let maybe_inner = self.inner.borrow();
        let inner = maybe_inner.as_ref().ok_or(Error::Initialization)?;
        inner.eval_ctx_set_input(ctx, input)?;
        Ok(())
    }

    pub fn eval_ctx_set_data(&self, ctx: ValueAddr, data: ValueAddr) -> Result<(), Error> {
        let maybe_inner = self.inner.borrow();
        let inner = maybe_inner.as_ref().ok_or(Error::Initialization)?;
        inner.eval_ctx_set_data(ctx, data)?;
        Ok(())
    }

    pub fn eval(&self, ctx: ValueAddr) -> Result<(), Error> {
        let maybe_inner = self.inner.borrow();
        let inner = maybe_inner.as_ref().ok_or(Error::Initialization)?;
        inner.eval(ctx)?;
        Ok(())
    }

    pub fn eval_ctx_get_result(&self, ctx: ValueAddr) -> Result<ValueAddr, Error> {
        let maybe_inner = self.inner.borrow();
        let inner = maybe_inner.as_ref().ok_or(Error::Initialization)?;
        let addr = inner.eval_ctx_get_result(ctx)?;
        Ok(addr.into())
    }

    pub fn heap_ptr_get(&self) -> Result<ValueAddr, Error> {
        let maybe_inner = self.inner.borrow();
        let inner = maybe_inner.as_ref().ok_or(Error::Initialization)?;
        let addr = inner.heap_ptr_get()?;
        Ok(addr.into())
    }

    pub fn heap_ptr_set(&self, addr: ValueAddr) -> Result<(), Error> {
        let maybe_inner = self.inner.borrow();
        let inner = maybe_inner.as_ref().ok_or(Error::Initialization)?;
        inner.heap_ptr_set(addr)?;
        Ok(())
    }

    pub fn heap_top_get(&self) -> Result<ValueAddr, Error> {
        let maybe_inner = self.inner.borrow();
        let inner = maybe_inner.as_ref().ok_or(Error::Initialization)?;
        let addr = inner.heap_top_get()?;
        Ok(addr.into())
    }

    pub fn heap_top_set(&self, addr: ValueAddr) -> Result<(), Error> {
        let maybe_inner = self.inner.borrow();
        let inner = maybe_inner.as_ref().ok_or(Error::Initialization)?;
        inner.heap_top_set(addr)?;
        Ok(())
    }

    pub fn malloc(&self, len: usize) -> Result<ValueAddr, Error> {
        let maybe_inner = self.inner.borrow();
        let inner = maybe_inner.as_ref().ok_or(Error::Initialization)?;
        let addr = inner.malloc(len)?;
        Ok(addr.into())
    }

    pub fn json_parse(&self, addr: ValueAddr, len: usize) -> Result<ValueAddr, Error> {
        let maybe_inner = self.inner.borrow();
        let inner = maybe_inner.as_ref().ok_or(Error::Initialization)?;
        let parsed_addr = inner.json_parse(addr, len)?;
        Ok(parsed_addr.into())
    }

    pub fn json_dump(&self, addr: ValueAddr) -> Result<ValueAddr, Error> {
        let maybe_inner = self.inner.borrow();
        let inner = maybe_inner.as_ref().ok_or(Error::Initialization)?;
        let raw_addr = inner.json_dump(addr)?;
        Ok(raw_addr.into())
    }
}

#[allow(dead_code)]
struct Inner {
    instance: Instance,
    opa_malloc: Box<dyn Fn(i32) -> Result<i32, Trap>>,
    opa_json_parse: Box<dyn Fn(i32, i32) -> Result<i32, Trap>>,
    opa_json_dump: Box<dyn Fn(i32) -> Result<i32, Trap>>,
    opa_heap_ptr_get: Box<dyn Fn() -> Result<i32, Trap>>,
    opa_heap_ptr_set: Box<dyn Fn(i32) -> Result<(), Trap>>,
    opa_heap_top_get: Box<dyn Fn() -> Result<i32, Trap>>,
    opa_heap_top_set: Box<dyn Fn(i32) -> Result<(), Trap>>,
    opa_eval_ctx_new: Box<dyn Fn() -> Result<i32, Trap>>,
    opa_eval_ctx_set_input: Box<dyn Fn(i32, i32) -> Result<(), Trap>>,
    opa_eval_ctx_set_data: Box<dyn Fn(i32, i32) -> Result<(), Trap>>,
    opa_eval_ctx_get_result: Box<dyn Fn(i32) -> Result<i32, Trap>>,
    builtins: Box<dyn Fn() -> Result<i32, Trap>>,
    eval: Box<dyn Fn(i32) -> Result<i32, Trap>>,
}

impl fmt::Debug for Inner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Inner")
    }
}

impl Inner {
    fn from_instance(instance: Instance) -> Result<Self, Error> {
        let opa_malloc = instance
            .get_export("opa_malloc")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_malloc"))
            .and_then(|f| f.get1::<i32, i32>().map_err(|e| Error::Wasm(e)))?;

        let opa_json_parse = instance
            .get_export("opa_json_parse")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_json_parse"))
            .and_then(|f| f.get2::<i32, i32, i32>().map_err(|e| Error::Wasm(e)))?;

        let opa_json_dump = instance
            .get_export("opa_json_dump")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_json_dump"))
            .and_then(|f| f.get1::<i32, i32>().map_err(|e| Error::Wasm(e)))?;

        let opa_heap_ptr_get = instance
            .get_export("opa_heap_ptr_get")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_heap_ptr_get"))
            .and_then(|f| f.get0::<i32>().map_err(|e| Error::Wasm(e)))?;

        let opa_heap_ptr_set = instance
            .get_export("opa_heap_ptr_set")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_heap_ptr_set"))
            .and_then(|f| f.get1::<i32, ()>().map_err(|e| Error::Wasm(e)))?;

        let opa_heap_top_get = instance
            .get_export("opa_heap_top_get")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_heap_top_get"))
            .and_then(|f| f.get0::<i32>().map_err(|e| Error::Wasm(e)))?;

        let opa_heap_top_set = instance
            .get_export("opa_heap_top_set")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_heap_top_set"))
            .and_then(|f| f.get1::<i32, ()>().map_err(|e| Error::Wasm(e)))?;

        let opa_eval_ctx_new = instance
            .get_export("opa_eval_ctx_new")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_eval_ctx_new"))
            .and_then(|f| f.get0::<i32>().map_err(|e| Error::Wasm(e)))?;

        let opa_eval_ctx_set_input = instance
            .get_export("opa_eval_ctx_set_input")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_eval_ctx_set_input"))
            .and_then(|f| f.get2::<i32, i32, ()>().map_err(|e| Error::Wasm(e)))?;

        let opa_eval_ctx_set_data = instance
            .get_export("opa_eval_ctx_set_data")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_eval_ctx_set_data"))
            .and_then(|f| f.get2::<i32, i32, ()>().map_err(|e| Error::Wasm(e)))?;

        let opa_eval_ctx_get_result = instance
            .get_export("opa_eval_ctx_get_result")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_eval_ctx_get_result"))
            .and_then(|f| f.get1::<i32, i32>().map_err(|e| Error::Wasm(e)))?;

        let builtins = instance
            .get_export("builtins")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("builtins"))
            .and_then(|f| f.get0::<i32>().map_err(|e| Error::Wasm(e)))?;

        let eval = instance
            .get_export("eval")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("eval"))
            .and_then(|f| f.get1::<i32, i32>().map_err(|e| Error::Wasm(e)))?;

        let inner = Inner {
            instance,
            opa_malloc: Box::new(opa_malloc),
            opa_json_parse: Box::new(opa_json_parse),
            opa_json_dump: Box::new(opa_json_dump),
            opa_heap_ptr_get: Box::new(opa_heap_ptr_get),
            opa_heap_ptr_set: Box::new(opa_heap_ptr_set),
            opa_heap_top_get: Box::new(opa_heap_top_get),
            opa_heap_top_set: Box::new(opa_heap_top_set),
            opa_eval_ctx_new: Box::new(opa_eval_ctx_new),
            opa_eval_ctx_set_input: Box::new(opa_eval_ctx_set_input),
            opa_eval_ctx_set_data: Box::new(opa_eval_ctx_set_data),
            opa_eval_ctx_get_result: Box::new(opa_eval_ctx_get_result),
            builtins: Box::new(builtins),
            eval: Box::new(eval),
        };
        Ok(inner)
    }

    fn builtins(&self) -> Result<ValueAddr, Error> {
        let addr = (self.builtins)()?;
        Ok(addr.into())
    }

    fn eval_ctx_new(&self) -> Result<ValueAddr, Error> {
        let addr = (self.opa_eval_ctx_new)()?;
        Ok(addr.into())
    }

    fn eval_ctx_set_input(&self, ctx: ValueAddr, input: ValueAddr) -> Result<(), Error> {
        (self.opa_eval_ctx_set_input)(ctx.0, input.0)?;
        Ok(())
    }

    fn eval_ctx_set_data(&self, ctx: ValueAddr, data: ValueAddr) -> Result<(), Error> {
        (self.opa_eval_ctx_set_data)(ctx.0, data.0)?;
        Ok(())
    }

    fn eval(&self, ctx: ValueAddr) -> Result<(), Error> {
        (self.eval)(ctx.0)?;
        Ok(())
    }

    fn eval_ctx_get_result(&self, ctx: ValueAddr) -> Result<ValueAddr, Error> {
        let addr = (self.opa_eval_ctx_get_result)(ctx.0)?;
        Ok(addr.into())
    }

    fn heap_ptr_get(&self) -> Result<ValueAddr, Error> {
        let addr = (self.opa_heap_ptr_get)()?;
        Ok(addr.into())
    }

    fn heap_ptr_set(&self, addr: ValueAddr) -> Result<(), Error> {
        (self.opa_heap_ptr_set)(addr.0)?;
        Ok(())
    }

    fn heap_top_get(&self) -> Result<ValueAddr, Error> {
        let addr = (self.opa_heap_top_get)()?;
        Ok(addr.into())
    }

    fn heap_top_set(&self, addr: ValueAddr) -> Result<(), Error> {
        (self.opa_heap_top_set)(addr.0)?;
        Ok(())
    }

    fn malloc(&self, len: usize) -> Result<ValueAddr, Error> {
        let addr = (self.opa_malloc)(len as i32)?;
        Ok(addr.into())
    }

    fn json_parse(&self, addr: ValueAddr, len: usize) -> Result<ValueAddr, Error> {
        let parsed_addr = (self.opa_json_parse)(addr.0, len as i32)?;
        if parsed_addr == 0 {
            return Err(Error::JsonParse(addr));
        }
        Ok(parsed_addr.into())
    }

    fn json_dump(&self, addr: ValueAddr) -> Result<ValueAddr, Error> {
        let raw_addr = (self.opa_json_dump)(addr.0)?;
        Ok(raw_addr.into())
    }
}
