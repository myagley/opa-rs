use std::mem;
use std::sync::Arc;

use crate::{Error, ValueAddr};

// #[cfg(target_arch = "x86_64")]
// mod wasmtime;

// #[cfg(not(target_arch = "x86_64"))]
mod wasmi;

// #[cfg(target_arch = "x86_64")]
// pub use self::wasmtime::{Instance, Memory, Module};

// #[cfg(not(target_arch = "x86_64"))]
pub use self::wasmi::{Instance, Memory, Module};

// #[cfg(target_arch = "x86_64")]
// use self::wasmtime::FunctionsImpl;

// #[cfg(not(target_arch = "x86_64"))]
use self::wasmi::FunctionsImpl;

pub trait AsBytes {
    fn as_bytes(&self) -> &[u8];
}

impl AsBytes for [u8] {
    fn as_bytes(&self) -> &[u8] {
        &self
    }
}

impl<'a> AsBytes for &'a [u8] {
    fn as_bytes(&self) -> &[u8] {
        *self
    }
}

impl AsBytes for str {
    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<'a> AsBytes for &'a str {
    fn as_bytes(&self) -> &[u8] {
        str::as_bytes(&*self)
    }
}

pub unsafe trait FromBytes: Sized + Copy {
    fn len() -> usize {
        mem::size_of::<Self>()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < mem::size_of::<Self>() {
            return Err(Error::NotEnoughData(mem::size_of::<Self>(), bytes.len()));
        }

        let bytes_ptr = bytes.as_ptr();
        let struct_ptr = bytes_ptr as *const Self;
        let struct_ref = unsafe { *struct_ptr };
        Ok(struct_ref)
    }
}

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
}
