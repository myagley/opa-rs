use std::path::Path;
use std::{fmt, process};

use tempfile::TempDir;
use wasmtime::*;

mod error;
mod value;

pub use error::Error;
pub use value::{Number, Value};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ValueAddr(i32);

impl fmt::Display for ValueAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ValueAddr({})", self.0)
    }
}

impl From<i32> for ValueAddr {
    fn from(addr: i32) -> Self {
        Self(addr)
    }
}

pub struct Policy {
    memory: Memory,
    data_addr: ValueAddr,
    base_heap_ptr: ValueAddr,
    base_heap_top: ValueAddr,
    data_heap_ptr: ValueAddr,
    data_heap_top: ValueAddr,

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

impl Policy {
    pub fn from_rego<P: AsRef<Path>>(path: P, query: &str) -> Result<Self, Error> {
        let dir = TempDir::new().map_err(Error::DirOpen)?;
        let wasm = dir.path().join("policy.wasm");
        let output = process::Command::new("opa")
            .arg("build")
            .args(&["-d".as_ref(), path.as_ref().as_os_str()])
            .args(&["-o".as_ref(), wasm.as_os_str()])
            .arg(query)
            .output()
            .map_err(Error::OpaCommand)?;

        if !output.status.success() {
            return Err(Error::OpaCompiler(
                String::from_utf8_lossy(&output.stdout).to_string(),
            ));
        }

        let store = Store::default();
        let module = Module::from_file(&store, &wasm).map_err(Error::Wasm)?;
        Self::from_wasm(&module)
    }

    pub fn from_wasm(module: &Module) -> Result<Self, Error> {
        let memorytype = MemoryType::new(Limits::new(5, None));
        let memory = Memory::new(module.store(), memorytype);

        let imports = [
            Extern::Memory(memory.clone()),
            Extern::Func(Func::wrap1(module.store(), abort)),
            Extern::Func(Func::wrap2(module.store(), builtin0)),
            Extern::Func(Func::wrap3(module.store(), builtin1)),
            Extern::Func(Func::wrap4(module.store(), builtin2)),
            Extern::Func(Func::wrap5(module.store(), builtin3)),
            Extern::Func(Func::wrap6(module.store(), builtin4)),
        ];

        let instance = Instance::new(module, &imports).map_err(|e| Error::Wasm(e))?;

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

        // Load the data
        let data = "{}";
        let raw_addr = opa_malloc(data.as_bytes().len() as i32)?;
        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr(),
                memory.data_ptr().offset(raw_addr as isize),
                data.as_bytes().len(),
            );
        }

        let data_addr = opa_json_parse(raw_addr, data.as_bytes().len() as i32)?.into();
        let base_heap_ptr = opa_heap_ptr_get()?.into();
        let base_heap_top = opa_heap_top_get()?.into();
        let data_heap_ptr = base_heap_ptr;
        let data_heap_top = base_heap_top;

        let mut policy = Policy {
            memory,
            data_addr,
            base_heap_ptr,
            base_heap_top,
            data_heap_ptr,
            data_heap_top,

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
        let builtins = policy.builtins()?;
        println!("builtins: {}", builtins);

        Ok(policy)
    }

    pub fn evaluate(&mut self, input: &str) -> Result<String, Error> {
        // Reset the heap pointers
        self.heap_ptr_set(self.data_heap_ptr)?;
        self.heap_top_set(self.data_heap_top)?;

        // Load input data
        let input_addr = self.load_json(input)?;

        // setup the context
        let ctx_addr = self.eval_ctx_new()?;
        self.eval_ctx_set_input(ctx_addr, input_addr)?;
        self.eval_ctx_set_data(ctx_addr, self.data_addr)?;

        // Eval
        self.eval(ctx_addr)?;

        let result_addr = self.eval_ctx_get_result(ctx_addr)?;
        let s = self.dump_json(result_addr)?;
        Ok(s)
    }

    pub fn set_data(&mut self, data: &str) -> Result<(), Error> {
        self.heap_ptr_set(self.base_heap_ptr)?;
        self.heap_top_set(self.base_heap_top)?;
        self.data_addr = self.load_json(data)?;
        self.data_heap_ptr = self.heap_ptr_get()?;
        self.data_heap_top = self.heap_top_get()?;
        Ok(())
    }

    fn eval_ctx_new(&mut self) -> Result<ValueAddr, Error> {
        let addr = (self.opa_eval_ctx_new)()?;
        Ok(addr.into())
    }

    fn eval_ctx_set_input(&mut self, ctx: ValueAddr, input: ValueAddr) -> Result<(), Error> {
        (self.opa_eval_ctx_set_input)(ctx.0, input.0)?;
        Ok(())
    }

    fn eval_ctx_set_data(&mut self, ctx: ValueAddr, data: ValueAddr) -> Result<(), Error> {
        (self.opa_eval_ctx_set_data)(ctx.0, data.0)?;
        Ok(())
    }

    fn builtins(&mut self) -> Result<String, Error> {
        let addr = (self.builtins)()?;
        let s = self.dump_json(addr.into())?;
        Ok(s)
    }

    fn eval(&mut self, ctx: ValueAddr) -> Result<(), Error> {
        (self.eval)(ctx.0)?;
        Ok(())
    }

    fn eval_ctx_get_result(&mut self, ctx: ValueAddr) -> Result<ValueAddr, Error> {
        let addr = (self.opa_eval_ctx_get_result)(ctx.0)?;
        Ok(addr.into())
    }

    fn heap_ptr_get(&mut self) -> Result<ValueAddr, Error> {
        let addr = (self.opa_heap_ptr_get)()?;
        Ok(addr.into())
    }

    fn heap_ptr_set(&mut self, addr: ValueAddr) -> Result<(), Error> {
        (self.opa_heap_ptr_set)(addr.0)?;
        Ok(())
    }

    fn heap_top_get(&mut self) -> Result<ValueAddr, Error> {
        let addr = (self.opa_heap_top_get)()?;
        Ok(addr.into())
    }

    fn heap_top_set(&mut self, addr: ValueAddr) -> Result<(), Error> {
        (self.opa_heap_top_set)(addr.0)?;
        Ok(())
    }

    fn malloc(&mut self, len: usize) -> Result<ValueAddr, Error> {
        let addr = (self.opa_malloc)(len as i32)?;
        Ok(addr.into())
    }

    fn json_parse(&mut self, addr: ValueAddr, len: usize) -> Result<ValueAddr, Error> {
        let parsed_addr = (self.opa_json_parse)(addr.0, len as i32)?;
        if parsed_addr == 0 {
            return Err(Error::JsonParse(addr));
        }
        Ok(parsed_addr.into())
    }

    fn json_dump(&mut self, addr: ValueAddr) -> Result<ValueAddr, Error> {
        let raw_addr = (self.opa_json_dump)(addr.0)?;
        Ok(raw_addr.into())
    }

    fn load_json(&mut self, value: &str) -> Result<ValueAddr, Error> {
        let raw_addr = self.malloc(value.as_bytes().len())?;
        unsafe {
            std::ptr::copy_nonoverlapping(
                value.as_ptr(),
                self.memory.data_ptr().offset(raw_addr.0 as isize),
                value.as_bytes().len(),
            );
        }
        let parsed_addr = self.json_parse(raw_addr, value.as_bytes().len())?;
        Ok(parsed_addr)
    }

    fn dump_json(&mut self, addr: ValueAddr) -> Result<String, Error> {
        let raw_addr = self.json_dump(addr)?;
        let s = unsafe {
            let p = self.memory.data_ptr().offset(raw_addr.0 as isize);
            let cstr = std::ffi::CStr::from_ptr(p as *const i8);
            let s = cstr.to_str().map_err(Error::CStr)?;
            s.to_string()
        };
        Ok(s)
    }
}

fn abort(_a: i32) {
    println!("abort");
}

fn builtin0(_a: i32, _b: i32) -> i32 {
    println!("builtin0");
    0
}

fn builtin1(_a: i32, _b: i32, _c: i32) -> i32 {
    println!("builtin1");
    0
}

fn builtin2(_a: i32, _b: i32, _c: i32, _d: i32) -> i32 {
    println!("builtin2");
    0
}

fn builtin3(_a: i32, _b: i32, _c: i32, _d: i32, _e: i32) -> i32 {
    println!("builtin3");
    0
}

fn builtin4(_a: i32, _b: i32, _c: i32, _d: i32, _e: i32, _f: i32) -> i32 {
    println!("builtin4");
    0
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
