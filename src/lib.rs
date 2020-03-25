use std::path::Path;
use std::{fmt, process};

use tempfile::TempDir;
use wasmtime::*;

mod builtins;
mod error;
mod functions;
mod value;

use builtins::Builtins;
use functions::Functions;

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

impl From<ValueAddr> for i32 {
    fn from(v: ValueAddr) -> Self {
        v.0
    }
}

#[allow(dead_code)]
pub struct Policy {
    functions: Functions,
    memory: Memory,
    data_addr: ValueAddr,
    base_heap_ptr: ValueAddr,
    base_heap_top: ValueAddr,
    data_heap_ptr: ValueAddr,
    data_heap_top: ValueAddr,
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
            Extern::Memory(memory.clone()),
            Extern::Func(Func::wrap1(module.store(), abort)),
            Extern::Func(Func::wrap2(module.store(), move |id, ctx| {
                i32::from(b0.builtin0(id, ValueAddr(ctx)))
            })),
            Extern::Func(Func::wrap3(module.store(), move |id, ctx, a| {
                i32::from(b1.builtin1(id, ValueAddr(ctx), ValueAddr(a)))
            })),
            Extern::Func(Func::wrap4(module.store(), move |id, ctx, a, b| {
                i32::from(b2.builtin2(id, ValueAddr(ctx), ValueAddr(a), ValueAddr(b)))
            })),
            Extern::Func(Func::wrap5(module.store(), move |id, ctx, a, b, c| {
                i32::from(b3.builtin3(id, ValueAddr(ctx), ValueAddr(a), ValueAddr(b), ValueAddr(c)))
            })),
            Extern::Func(Func::wrap6(module.store(), move |id, ctx, a, b, c, d| {
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

        let instance = Instance::new(module, &imports).map_err(|e| Error::Wasm(e))?;
        let functions = Functions::from_instance(instance.clone())?;
        builtins.replace(functions.clone(), memory.clone())?;

        // Load the data
        let data = "{}";
        let data_addr = functions.malloc(data.as_bytes().len())?;
        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr(),
                memory.data_ptr().offset(data_addr.0 as isize),
                data.as_bytes().len(),
            );
        }

        let data_addr = functions.json_parse(data_addr, data.as_bytes().len())?;
        let base_heap_ptr = functions.heap_ptr_get()?;
        let base_heap_top = functions.heap_top_get()?;
        let data_heap_ptr = base_heap_ptr;
        let data_heap_top = base_heap_top;

        let policy = Policy {
            functions,
            memory,
            data_addr,
            base_heap_ptr,
            base_heap_top,
            data_heap_ptr,
            data_heap_top,
        };

        Ok(policy)
    }

    // This takes a &mut self because calling it potentially mutates the
    // memory. We could make this take &self, if we add a mutex.
    pub fn evaluate(&mut self, input: &str) -> Result<Value, Error> {
        // Reset the heap pointers
        self.functions.heap_ptr_set(self.data_heap_ptr)?;
        self.functions.heap_top_set(self.data_heap_top)?;

        // Load input data
        let input_addr = self.load_json(input)?;

        // setup the context
        let ctx_addr = self.functions.eval_ctx_new()?;
        self.functions.eval_ctx_set_input(ctx_addr, input_addr)?;
        self.functions.eval_ctx_set_data(ctx_addr, self.data_addr)?;

        // Eval
        self.functions.eval(ctx_addr)?;

        let result_addr = self.functions.eval_ctx_get_result(ctx_addr)?;
        let s = self.dump_json(result_addr)?;
        let v = serde_json::from_str(&s).map_err(Error::DeserializeJson)?;
        Ok(v)
    }

    pub fn set_data(&mut self, data: &str) -> Result<(), Error> {
        self.functions.heap_ptr_set(self.base_heap_ptr)?;
        self.functions.heap_top_set(self.base_heap_top)?;
        self.data_addr = self.load_json(data)?;
        self.data_heap_ptr = self.functions.heap_ptr_get()?;
        self.data_heap_top = self.functions.heap_top_get()?;
        Ok(())
    }

    pub fn builtins(&mut self) -> Result<String, Error> {
        let addr = self.functions.builtins()?;
        let s = dump_json(&self.functions, &self.memory, addr)?;
        Ok(s)
    }

    fn load_json(&self, value: &str) -> Result<ValueAddr, Error> {
        load_json(&self.functions, &self.memory, value)
    }

    fn dump_json(&self, addr: ValueAddr) -> Result<String, Error> {
        dump_json(&self.functions, &self.memory, addr)
    }
}

pub(crate) fn dump_json(
    functions: &Functions,
    memory: &Memory,
    addr: ValueAddr,
) -> Result<String, Error> {
    let raw_addr = functions.json_dump(addr)?;
    let s = unsafe {
        let p = memory.data_ptr().offset(raw_addr.0 as isize);
        let cstr = std::ffi::CStr::from_ptr(p as *const i8);
        let s = cstr.to_str().map_err(Error::CStr)?;
        s.to_string()
    };
    Ok(s)
}

pub(crate) fn load_json(
    functions: &Functions,
    memory: &Memory,
    value: &str,
) -> Result<ValueAddr, Error> {
    let raw_addr = functions.malloc(value.as_bytes().len())?;
    unsafe {
        std::ptr::copy_nonoverlapping(
            value.as_ptr(),
            memory.data_ptr().offset(raw_addr.0 as isize),
            value.as_bytes().len(),
        );
    }
    let parsed_addr = functions.json_parse(raw_addr, value.as_bytes().len())?;
    Ok(parsed_addr)
}

fn abort(_a: i32) {
    println!("abort");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
