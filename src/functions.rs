use std::fmt;
use std::sync::Arc;

use wasmtime::{Instance, Trap};

use crate::{Error, ValueAddr};

#[derive(Clone)]
pub struct Functions {
    inner: Arc<Inner>,
}

impl Functions {
    pub fn from_instance(instance: Instance) -> Result<Self, Error> {
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

        let f = Self {
            inner: Arc::new(inner),
        };
        Ok(f)
    }

    pub fn builtins(&self) -> Result<ValueAddr, Error> {
        let addr = (self.inner.builtins)()?;
        Ok(addr.into())
    }

    pub fn eval_ctx_new(&self) -> Result<ValueAddr, Error> {
        let addr = (self.inner.opa_eval_ctx_new)()?;
        Ok(addr.into())
    }

    pub fn eval_ctx_set_input(&self, ctx: ValueAddr, input: ValueAddr) -> Result<(), Error> {
        (self.inner.opa_eval_ctx_set_input)(ctx.0, input.0)?;
        Ok(())
    }

    pub fn eval_ctx_set_data(&self, ctx: ValueAddr, data: ValueAddr) -> Result<(), Error> {
        (self.inner.opa_eval_ctx_set_data)(ctx.0, data.0)?;
        Ok(())
    }

    pub fn eval(&self, ctx: ValueAddr) -> Result<(), Error> {
        (self.inner.eval)(ctx.0)?;
        Ok(())
    }

    pub fn eval_ctx_get_result(&self, ctx: ValueAddr) -> Result<ValueAddr, Error> {
        let addr = (self.inner.opa_eval_ctx_get_result)(ctx.0)?;
        Ok(addr.into())
    }

    pub fn heap_ptr_get(&self) -> Result<ValueAddr, Error> {
        let addr = (self.inner.opa_heap_ptr_get)()?;
        Ok(addr.into())
    }

    pub fn heap_ptr_set(&self, addr: ValueAddr) -> Result<(), Error> {
        (self.inner.opa_heap_ptr_set)(addr.0)?;
        Ok(())
    }

    pub fn heap_top_get(&self) -> Result<ValueAddr, Error> {
        let addr = (self.inner.opa_heap_top_get)()?;
        Ok(addr.into())
    }

    pub fn heap_top_set(&self, addr: ValueAddr) -> Result<(), Error> {
        (self.inner.opa_heap_top_set)(addr.0)?;
        Ok(())
    }

    pub fn malloc(&self, len: usize) -> Result<ValueAddr, Error> {
        let addr = (self.inner.opa_malloc)(len as i32)?;
        Ok(addr.into())
    }

    pub fn json_parse(&self, addr: ValueAddr, len: usize) -> Result<ValueAddr, Error> {
        let parsed_addr = (self.inner.opa_json_parse)(addr.0, len as i32)?;
        if parsed_addr == 0 {
            return Err(Error::JsonParse(addr));
        }
        Ok(parsed_addr.into())
    }

    pub fn json_dump(&self, addr: ValueAddr) -> Result<ValueAddr, Error> {
        let raw_addr = (self.inner.opa_json_dump)(addr.0)?;
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
