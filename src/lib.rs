use std::path::Path;
use std::{fmt, process};

use tempfile::TempDir;
use wasmtime::*;

mod error;
mod functions;
mod value;

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
        let functions = Functions::default();

        let func0 = functions.clone();
        let func1 = functions.clone();
        let func2 = functions.clone();
        let func3 = functions.clone();
        let func4 = functions.clone();
        let mem0 = memory.clone();
        let mem1 = memory.clone();
        let mem2 = memory.clone();
        let mem3 = memory.clone();
        let mem4 = memory.clone();

        let imports = [
            Extern::Memory(memory.clone()),
            Extern::Func(Func::wrap1(module.store(), abort)),
            Extern::Func(Func::wrap2(module.store(), move |id, ctx| {
                builtin0(&func0, &mem0, ValueAddr(id), ValueAddr(ctx))
            })),
            Extern::Func(Func::wrap3(module.store(), move |id, ctx, a| {
                builtin1(&func1, &mem1, id, ValueAddr(ctx), ValueAddr(a))
            })),
            Extern::Func(Func::wrap4(module.store(), move |id, ctx, a, b| {
                builtin2(
                    &func2,
                    &mem2,
                    id,
                    ValueAddr(ctx),
                    ValueAddr(a),
                    ValueAddr(b),
                )
            })),
            Extern::Func(Func::wrap5(module.store(), move |id, ctx, a, b, c| {
                builtin3(&func3, &mem3, id, ValueAddr(ctx), ValueAddr(a), ValueAddr(b), ValueAddr(c))
            })),
            Extern::Func(Func::wrap6(module.store(), move |id, ctx, a, b, c, d | {
                builtin4(&func4, &mem4, id, ValueAddr(ctx), ValueAddr(a), ValueAddr(b), ValueAddr(c), ValueAddr(d))
            })),
        ];

        let instance = Instance::new(module, &imports).map_err(|e| Error::Wasm(e))?;
        functions.replace(instance)?;

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

        let mut policy = Policy {
            functions,
            memory,
            data_addr,
            base_heap_ptr,
            base_heap_top,
            data_heap_ptr,
            data_heap_top,
        };

        let builtins = policy.builtins()?;
        println!("builtins: {}", builtins);

        Ok(policy)
    }

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

fn dump_json(functions: &Functions, memory: &Memory, addr: ValueAddr) -> Result<String, Error> {
    let raw_addr = functions.json_dump(addr)?;
    let s = unsafe {
        let p = memory.data_ptr().offset(raw_addr.0 as isize);
        let cstr = std::ffi::CStr::from_ptr(p as *const i8);
        let s = cstr.to_str().map_err(Error::CStr)?;
        s.to_string()
    };
    Ok(s)
}

fn load_json(functions: &Functions, memory: &Memory, value: &str) -> Result<ValueAddr, Error> {
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

macro_rules! btry {
    ($expr:expr) => {
        match $expr {
            ::std::result::Result::Ok(val) => val,
            ::std::result::Result::Err(err) => {
                println!("builtin error: {}", err);
                return 0;
            }
        }
    };
}

fn builtin0(_functions: &Functions, _memory: &Memory, _id: ValueAddr, _ctx_addr: ValueAddr) -> i32 {
    println!("builtin0");
    0
}

fn builtin1(
    functions: &Functions,
    memory: &Memory,
    _id: i32,
    _ctx_addr: ValueAddr,
    value: ValueAddr,
) -> i32 {
    let val = btry!(dump_json(functions, memory, value)
        .and_then(|s| serde_json::from_str(&s).map_err(Error::DeserializeJson)));

    let len = match val {
        Value::Array(ref v) => v.len(),
        Value::Object(ref v) => v.len(),
        Value::Set(ref v) => v.len(),
        _ => return 0,
    };

    let serialized = btry!(serde_json::to_string(&Value::Number(len.into())));
    let addr = btry!(load_json(functions, memory, &serialized));
    addr.0
}

fn builtin2(
    functions: &Functions,
    memory: &Memory,
    _id: i32,
    _ctx_addr: ValueAddr,
    a: ValueAddr,
    b: ValueAddr,
) -> i32 {
    let val1: Value = btry!(dump_json(functions, memory, a)
        .and_then(|s| serde_json::from_str(&s).map_err(Error::DeserializeJson)));
    let val2: Value = btry!(dump_json(functions, memory, b)
        .and_then(|s| serde_json::from_str(&s).map_err(Error::DeserializeJson)));

    let num1 = btry!(val1
        .as_i64()
        .ok_or_else(|| Error::InvalidType("Number", val1)));
    let num2 = btry!(val2
        .as_i64()
        .ok_or_else(|| Error::InvalidType("Number", val2)));
    let sum = num1 + num2;
    let serialized = btry!(serde_json::to_string(&Value::Number(sum.into())));
    let addr = btry!(load_json(functions, memory, &serialized));
    addr.0
}

fn builtin3(
    _functions: &Functions,
    _memory: &Memory,
    _id: i32,
    _ctx_addr: ValueAddr,
    _a: ValueAddr,
    _b: ValueAddr,
    _c: ValueAddr,
) -> i32 {
    println!("builtin3");
    0
}

fn builtin4(
    _functions: &Functions,
    _memory: &Memory,
    _id: i32,
    _ctx_addr: ValueAddr,
    _a: ValueAddr,
    _b: ValueAddr,
    _c: ValueAddr,
    _d: ValueAddr,
) -> i32 {
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
