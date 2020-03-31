use crate::{Error, ValueAddr};
use std::sync::Arc;

#[cfg(target_arch = "x86_64")]
mod wasmtime;

#[cfg(not(target_arch = "x86_64"))]
mod wasmi;

#[cfg(target_arch = "x86_64")]
pub use self::wasmtime::{Instance, Memory, Module};

#[cfg(not(target_arch = "x86_64"))]
pub use self::wasmi::{Instance, Memory, Module};

#[cfg(target_arch = "x86_64")]
use self::wasmtime::FunctionsImpl;

#[cfg(not(target_arch = "x86_64"))]
use self::wasmi::FunctionsImpl;

#[derive(Clone, Debug)]
pub struct Functions {
    inner: Arc<FunctionsImpl>,
}

impl Functions {
    pub fn from_impl(inner: FunctionsImpl) -> Result<Self, Error> {
        let f = Self {
            inner: Arc::new(inner),
        };
        Ok(f)
    }

    pub fn builtins(&self) -> Result<ValueAddr, Error> {
        let addr = self.inner.builtins()?;
        Ok(addr.into())
    }

    pub fn eval_ctx_new(&self) -> Result<ValueAddr, Error> {
        let addr = self.inner.opa_eval_ctx_new()?;
        Ok(addr.into())
    }

    pub fn eval_ctx_set_input(&self, ctx: ValueAddr, input: ValueAddr) -> Result<(), Error> {
        self.inner.opa_eval_ctx_set_input(ctx.0, input.0)?;
        Ok(())
    }

    pub fn eval_ctx_set_data(&self, ctx: ValueAddr, data: ValueAddr) -> Result<(), Error> {
        self.inner.opa_eval_ctx_set_data(ctx.0, data.0)?;
        Ok(())
    }

    pub fn eval(&self, ctx: ValueAddr) -> Result<(), Error> {
        self.inner.eval(ctx.0)?;
        Ok(())
    }

    pub fn eval_ctx_get_result(&self, ctx: ValueAddr) -> Result<ValueAddr, Error> {
        let addr = self.inner.opa_eval_ctx_get_result(ctx.0)?;
        Ok(addr.into())
    }

    pub fn heap_ptr_get(&self) -> Result<ValueAddr, Error> {
        let addr = self.inner.opa_heap_ptr_get()?;
        Ok(addr.into())
    }

    pub fn heap_ptr_set(&self, addr: ValueAddr) -> Result<(), Error> {
        self.inner.opa_heap_ptr_set(addr.0)?;
        Ok(())
    }

    pub fn heap_top_get(&self) -> Result<ValueAddr, Error> {
        let addr = self.inner.opa_heap_top_get()?;
        Ok(addr.into())
    }

    pub fn heap_top_set(&self, addr: ValueAddr) -> Result<(), Error> {
        self.inner.opa_heap_top_set(addr.0)?;
        Ok(())
    }

    pub fn malloc(&self, len: usize) -> Result<ValueAddr, Error> {
        let addr = self.inner.opa_malloc(len as i32)?;
        Ok(addr.into())
    }

    pub fn json_parse(&self, addr: ValueAddr, len: usize) -> Result<ValueAddr, Error> {
        let parsed_addr = self.inner.opa_json_parse(addr.0, len as i32)?;
        if parsed_addr == 0 {
            return Err(Error::JsonParse(addr));
        }
        Ok(parsed_addr.into())
    }

    pub fn json_dump(&self, addr: ValueAddr) -> Result<ValueAddr, Error> {
        let raw_addr = self.inner.opa_json_dump(addr.0)?;
        Ok(raw_addr.into())
    }
}
