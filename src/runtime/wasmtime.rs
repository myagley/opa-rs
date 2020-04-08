use std::fmt;
use std::path::Path;

use wasmtime::{Extern, Func, Limits, MemoryType, Store, Trap};

use crate::builtins::Builtins;
use crate::error::Error;
use crate::ValueAddr;

use super::{AsBytes, FromBytes, Functions};

#[derive(Clone)]
pub struct Instance {
    memory: Memory,
    functions: Functions,
}

impl Instance {
    pub fn new(module: &Module, memory: Memory) -> Result<Self, Error> {
        // Builtins are tricky to handle.
        // We need to setup the functions as imports before creating
        // the instance. However, these functions require an instance to be called.
        // This is a circular dependency, which needless to say poses problems for
        // rust.
        //
        // To workaround this, we create an empty Builtins struct that we pass to the
        // imports so they can get a reference. Then, the instance is created and the
        // Builtins struct is updated with the instance. This is safe because none of
        // the builtins are called before the instance is created. It makes the Builtins
        // struct annoyingly complex because we need to use an Arc for shared references
        // as well as mutate the contents, requiring a RefCell.
        let builtins = Builtins::default();

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
        let fimpl = FunctionsImpl::from_instance(instance)?;
        let functions = Functions::from_impl(fimpl)?;

        let instance = Instance { memory, functions };
        builtins.replace(instance.clone())?;

        Ok(instance)
    }

    pub fn functions(&self) -> &Functions {
        &self.functions
    }

    pub fn memory(&self) -> &Memory {
        &self.memory
    }
}

impl fmt::Debug for Instance {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Instance")
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

    pub fn get<T: FromBytes>(&self, addr: ValueAddr) -> Result<T, Error> {
        let start = addr.0 as usize;
        let t = unsafe { T::from_bytes(&self.0.data_unchecked()[start..])? };
        Ok(t)
    }

    pub fn get_bytes(&self, addr: ValueAddr, len: usize) -> Result<Vec<u8>, Error> {
        let start = addr.0 as usize;
        let end = start + len;
        let t = unsafe { Vec::from(&self.0.data_unchecked()[start..end]) };
        Ok(t)
    }

    pub fn set<T: AsBytes>(&self, addr: ValueAddr, value: &T) -> Result<(), Error> {
        let bytes = value.as_bytes();
        unsafe {
            let start = addr.0 as usize;
            let end = start + bytes.len();
            self.0.data_unchecked_mut()[start..end].copy_from_slice(bytes);
        }
        Ok(())
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Memory")
    }
}

#[derive(Clone)]
pub struct Module(wasmtime::Module);

impl Module {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Module, Error> {
        let store = Store::default();
        let module = wasmtime::Module::from_file(&store, &path).map_err(Error::Wasmtime)?;
        Ok(Module(module))
    }

    pub fn from_bytes<B: AsRef<[u8]>>(bytes: B) -> Result<Module, Error> {
        let store = Store::default();
        let module = wasmtime::Module::new(&store, bytes).map_err(Error::Wasmtime)?;
        Ok(Module(module))
    }
}

#[allow(dead_code)]
pub struct FunctionsImpl {
    instance: wasmtime::Instance,
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
    fn from_instance(instance: wasmtime::Instance) -> Result<Self, Error> {
        let opa_malloc = instance
            .get_export("opa_malloc")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_malloc"))
            .and_then(|f| f.get1::<i32, i32>().map_err(|e| Error::Wasmtime(e)))?;

        let opa_json_parse = instance
            .get_export("opa_json_parse")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_json_parse"))
            .and_then(|f| f.get2::<i32, i32, i32>().map_err(|e| Error::Wasmtime(e)))?;

        let opa_json_dump = instance
            .get_export("opa_json_dump")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_json_dump"))
            .and_then(|f| f.get1::<i32, i32>().map_err(|e| Error::Wasmtime(e)))?;

        let opa_heap_ptr_get = instance
            .get_export("opa_heap_ptr_get")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_heap_ptr_get"))
            .and_then(|f| f.get0::<i32>().map_err(|e| Error::Wasmtime(e)))?;

        let opa_heap_ptr_set = instance
            .get_export("opa_heap_ptr_set")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_heap_ptr_set"))
            .and_then(|f| f.get1::<i32, ()>().map_err(|e| Error::Wasmtime(e)))?;

        let opa_heap_top_get = instance
            .get_export("opa_heap_top_get")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_heap_top_get"))
            .and_then(|f| f.get0::<i32>().map_err(|e| Error::Wasmtime(e)))?;

        let opa_heap_top_set = instance
            .get_export("opa_heap_top_set")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_heap_top_set"))
            .and_then(|f| f.get1::<i32, ()>().map_err(|e| Error::Wasmtime(e)))?;

        let opa_eval_ctx_new = instance
            .get_export("opa_eval_ctx_new")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_eval_ctx_new"))
            .and_then(|f| f.get0::<i32>().map_err(|e| Error::Wasmtime(e)))?;

        let opa_eval_ctx_set_input = instance
            .get_export("opa_eval_ctx_set_input")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_eval_ctx_set_input"))
            .and_then(|f| f.get2::<i32, i32, ()>().map_err(|e| Error::Wasmtime(e)))?;

        let opa_eval_ctx_set_data = instance
            .get_export("opa_eval_ctx_set_data")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_eval_ctx_set_data"))
            .and_then(|f| f.get2::<i32, i32, ()>().map_err(|e| Error::Wasmtime(e)))?;

        let opa_eval_ctx_get_result = instance
            .get_export("opa_eval_ctx_get_result")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("opa_eval_ctx_get_result"))
            .and_then(|f| f.get1::<i32, i32>().map_err(|e| Error::Wasmtime(e)))?;

        let builtins = instance
            .get_export("builtins")
            .and_then(|ext| ext.func())
            .ok_or_else(|| Error::MissingExport("builtins"))
            .and_then(|f| f.get0::<i32>().map_err(|e| Error::Wasmtime(e)))?;

        let eval = instance
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
