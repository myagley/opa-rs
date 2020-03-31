use std::ffi::{CStr, CString};
use std::fmt;
use std::os::raw::c_char;
use std::path::Path;

use wasmtime::{Extern, Func, Limits, MemoryType, Store, Trap};

use crate::builtins::Builtins;
use crate::error::Error;
use crate::ValueAddr;

pub struct Instance(wasmtime::Instance);

impl Instance {
    pub fn new(module: &Module, memory: &Memory, builtins: &Builtins) -> Result<Self, Error> {
        let b0 = builtins.clone();
        let b1 = builtins.clone();
        let b2 = builtins.clone();
        let b3 = builtins.clone();
        let b4 = builtins.clone();

        let imports = [
            Extern::Memory(memory.clone().0),
            Extern::Func(Func::wrap1(module.0.store(), crate::abort)),
            Extern::Func(Func::wrap2(module.0.store(), move |id, ctx| {
                i32::from(b0.builtin0(id, ValueAddr(ctx)))
            })),
            Extern::Func(Func::wrap3(module.0.store(), move |id, ctx, a| {
                i32::from(b1.builtin1(id, ValueAddr(ctx), ValueAddr(a)))
            })),
            Extern::Func(Func::wrap4(module.0.store(), move |id, ctx, a, b| {
                i32::from(b2.builtin2(id, ValueAddr(ctx), ValueAddr(a), ValueAddr(b)))
            })),
            Extern::Func(Func::wrap5(module.0.store(), move |id, ctx, a, b, c| {
                i32::from(b3.builtin3(id, ValueAddr(ctx), ValueAddr(a), ValueAddr(b), ValueAddr(c)))
            })),
            Extern::Func(Func::wrap6(module.0.store(), move |id, ctx, a, b, c, d| {
                i32::from(b4.builtin4(
                    id,
                    ValueAddr(ctx),
                    ValueAddr(a),
                    ValueAddr(b),
                    ValueAddr(c),
                    ValueAddr(d),
                ))
            })),
        ];

        let instance =
            wasmtime::Instance::new(&module.0, &imports).map_err(|e| Error::Wasmtime(e))?;
        Ok(Instance(instance))
    }
}

#[derive(Clone)]
pub struct Memory(wasmtime::Memory);

impl Memory {
    pub fn from_module(module: &Module) -> Self {
        let memorytype = MemoryType::new(Limits::new(5, None));
        let memory = wasmtime::Memory::new(module.0.store(), memorytype);
        Memory(memory)
    }

    pub fn cstring_at(&self, addr: ValueAddr) -> Result<CString, Error> {
        let s = unsafe {
            let p = self.0.data_ptr().offset(addr.0 as isize);
            CStr::from_ptr(p as *const c_char).to_owned()
        };
        Ok(s)
    }

    pub fn set(&self, addr: ValueAddr, value: &[u8]) -> Result<(), Error> {
        unsafe {
            std::ptr::copy_nonoverlapping(
                value.as_ptr(),
                self.0.data_ptr().offset(addr.0 as isize),
                value.len(),
            );
        }
        Ok(())
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Memory")
    }
}

pub struct Module(wasmtime::Module);

impl Module {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Module, Error> {
        let store = Store::default();
        let module = wasmtime::Module::from_file(&store, &path).map_err(Error::Wasmtime)?;
        Ok(Module(module))
    }
}

#[allow(dead_code)]
pub struct FunctionsImpl {
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

impl FunctionsImpl {
    pub fn from_instance(instance: Instance) -> Result<Self, Error> {
        let opa_malloc = instance
            .0
            .get_export("opa_malloc")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_malloc"))
            .and_then(|f| f.get1::<i32, i32>().map_err(|e| Error::Wasmtime(e)))?;

        let opa_json_parse = instance
            .0
            .get_export("opa_json_parse")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_json_parse"))
            .and_then(|f| f.get2::<i32, i32, i32>().map_err(|e| Error::Wasmtime(e)))?;

        let opa_json_dump = instance
            .0
            .get_export("opa_json_dump")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_json_dump"))
            .and_then(|f| f.get1::<i32, i32>().map_err(|e| Error::Wasmtime(e)))?;

        let opa_heap_ptr_get = instance
            .0
            .get_export("opa_heap_ptr_get")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_heap_ptr_get"))
            .and_then(|f| f.get0::<i32>().map_err(|e| Error::Wasmtime(e)))?;

        let opa_heap_ptr_set = instance
            .0
            .get_export("opa_heap_ptr_set")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_heap_ptr_set"))
            .and_then(|f| f.get1::<i32, ()>().map_err(|e| Error::Wasmtime(e)))?;

        let opa_heap_top_get = instance
            .0
            .get_export("opa_heap_top_get")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_heap_top_get"))
            .and_then(|f| f.get0::<i32>().map_err(|e| Error::Wasmtime(e)))?;

        let opa_heap_top_set = instance
            .0
            .get_export("opa_heap_top_set")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_heap_top_set"))
            .and_then(|f| f.get1::<i32, ()>().map_err(|e| Error::Wasmtime(e)))?;

        let opa_eval_ctx_new = instance
            .0
            .get_export("opa_eval_ctx_new")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_eval_ctx_new"))
            .and_then(|f| f.get0::<i32>().map_err(|e| Error::Wasmtime(e)))?;

        let opa_eval_ctx_set_input = instance
            .0
            .get_export("opa_eval_ctx_set_input")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_eval_ctx_set_input"))
            .and_then(|f| f.get2::<i32, i32, ()>().map_err(|e| Error::Wasmtime(e)))?;

        let opa_eval_ctx_set_data = instance
            .0
            .get_export("opa_eval_ctx_set_data")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_eval_ctx_set_data"))
            .and_then(|f| f.get2::<i32, i32, ()>().map_err(|e| Error::Wasmtime(e)))?;

        let opa_eval_ctx_get_result = instance
            .0
            .get_export("opa_eval_ctx_get_result")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_eval_ctx_get_result"))
            .and_then(|f| f.get1::<i32, i32>().map_err(|e| Error::Wasmtime(e)))?;

        let builtins = instance
            .0
            .get_export("builtins")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("builtins"))
            .and_then(|f| f.get0::<i32>().map_err(|e| Error::Wasmtime(e)))?;

        let eval = instance
            .0
            .get_export("eval")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("eval"))
            .and_then(|f| f.get1::<i32, i32>().map_err(|e| Error::Wasmtime(e)))?;

        let inner = FunctionsImpl {
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

    pub fn builtins(&self) -> Result<i32, Error> {
        let addr = (self.builtins)().map_err(Error::Trap)?;
        Ok(addr)
    }

    pub fn opa_eval_ctx_new(&self) -> Result<i32, Error> {
        let addr = (self.opa_eval_ctx_new)().map_err(Error::Trap)?;
        Ok(addr)
    }

    pub fn opa_eval_ctx_set_input(&self, ctx: i32, input: i32) -> Result<(), Error> {
        (self.opa_eval_ctx_set_input)(ctx, input).map_err(Error::Trap)?;
        Ok(())
    }

    pub fn opa_eval_ctx_set_data(&self, ctx: i32, data: i32) -> Result<(), Error> {
        (self.opa_eval_ctx_set_data)(ctx, data).map_err(Error::Trap)?;
        Ok(())
    }

    pub fn eval(&self, ctx: i32) -> Result<(), Error> {
        (self.eval)(ctx).map_err(Error::Trap)?;
        Ok(())
    }

    pub fn opa_eval_ctx_get_result(&self, ctx: i32) -> Result<i32, Error> {
        let addr = (self.opa_eval_ctx_get_result)(ctx).map_err(Error::Trap)?;
        Ok(addr)
    }

    pub fn opa_heap_ptr_get(&self) -> Result<i32, Error> {
        let addr = (self.opa_heap_ptr_get)().map_err(Error::Trap)?;
        Ok(addr)
    }

    pub fn opa_heap_ptr_set(&self, addr: i32) -> Result<(), Error> {
        (self.opa_heap_ptr_set)(addr).map_err(Error::Trap)?;
        Ok(())
    }

    pub fn opa_heap_top_get(&self) -> Result<i32, Error> {
        let addr = (self.opa_heap_top_get)().map_err(Error::Trap)?;
        Ok(addr)
    }

    pub fn opa_heap_top_set(&self, addr: i32) -> Result<(), Error> {
        (self.opa_heap_top_set)(addr).map_err(Error::Trap)?;
        Ok(())
    }

    pub fn opa_malloc(&self, len: i32) -> Result<i32, Error> {
        let addr = (self.opa_malloc)(len).map_err(Error::Trap)?;
        Ok(addr)
    }

    pub fn opa_json_parse(&self, addr: i32, len: i32) -> Result<i32, Error> {
        let parsed_addr = (self.opa_json_parse)(addr, len)?;
        Ok(parsed_addr)
    }

    pub fn opa_json_dump(&self, addr: i32) -> Result<i32, Error> {
        let raw_addr = (self.opa_json_dump)(addr).map_err(Error::Trap)?;
        Ok(raw_addr)
    }
}

impl fmt::Debug for FunctionsImpl {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "FunctionsImpl")
    }
}