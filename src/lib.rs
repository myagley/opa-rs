use std::path::Path;
use std::{fmt, process};

use serde::Serialize;
use tempfile::TempDir;

mod builtins;
mod error;
pub mod value;
mod wasm;

use wasm::{Instance, Memory, Module};

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
    instance: Instance,
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

        let module = Module::from_file(&wasm)?;
        Self::from_wasm(&module)
    }

    pub fn from_wasm(module: &Module) -> Result<Self, Error> {
        let memory = Memory::from_module(module);
        let instance = Instance::new(module, memory)?;

        // Load initial data
        let data = "{}";
        let data_addr = instance.functions().malloc(data.as_bytes().len())?;
        instance.memory().set(data_addr, data.as_bytes())?;

        let data_addr = instance
            .functions()
            .json_parse(data_addr, data.as_bytes().len())?;
        let base_heap_ptr = instance.functions().heap_ptr_get()?;
        let base_heap_top = instance.functions().heap_top_get()?;
        let data_heap_ptr = base_heap_ptr;
        let data_heap_top = base_heap_top;

        let policy = Policy {
            instance,
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
    pub fn evaluate<T: Serialize>(&mut self, input: &T) -> Result<Value, Error> {
        // Reset the heap pointers
        self.instance.functions().heap_ptr_set(self.data_heap_ptr)?;
        self.instance.functions().heap_top_set(self.data_heap_top)?;

        // Load input data
        let serialized = serde_json::to_string(input).map_err(Error::SerializeJson)?;
        let input_addr = self.load_json(&serialized)?;

        // setup the context
        let ctx_addr = self.instance.functions().eval_ctx_new()?;
        self.instance
            .functions()
            .eval_ctx_set_input(ctx_addr, input_addr)?;
        self.instance
            .functions()
            .eval_ctx_set_data(ctx_addr, self.data_addr)?;

        // Eval
        self.instance.functions().eval(ctx_addr)?;

        let result_addr = self.instance.functions().eval_ctx_get_result(ctx_addr)?;
        let s = self.dump_json(result_addr)?;
        let v = serde_json::from_str(&s).map_err(Error::DeserializeJson)?;
        Ok(v)
    }

    pub fn set_data(&mut self, data: &str) -> Result<(), Error> {
        self.instance.functions().heap_ptr_set(self.base_heap_ptr)?;
        self.instance.functions().heap_top_set(self.base_heap_top)?;
        self.data_addr = self.load_json(data)?;
        self.data_heap_ptr = self.instance.functions().heap_ptr_get()?;
        self.data_heap_top = self.instance.functions().heap_top_get()?;
        Ok(())
    }

    pub fn builtins(&mut self) -> Result<String, Error> {
        let addr = self.instance.functions().builtins()?;
        let s = dump_json(&self.instance, addr)?;
        Ok(s)
    }

    fn load_json(&self, value: &str) -> Result<ValueAddr, Error> {
        load_json(&self.instance, value)
    }

    fn dump_json(&self, addr: ValueAddr) -> Result<String, Error> {
        dump_json(&self.instance, addr)
    }
}

pub(crate) fn dump_json(instance: &Instance, addr: ValueAddr) -> Result<String, Error> {
    let raw_addr = instance.functions().json_dump(addr)?;
    let s = instance
        .memory()
        .cstring_at(raw_addr)?
        .into_string()
        .map_err(|e| Error::CStr(e.utf8_error()))?;
    Ok(s)
}

pub(crate) fn load_json(instance: &Instance, value: &str) -> Result<ValueAddr, Error> {
    let raw_addr = instance.functions().malloc(value.as_bytes().len())?;
    instance.memory().set(raw_addr, value.as_bytes())?;
    let parsed_addr = instance
        .functions()
        .json_parse(raw_addr, value.as_bytes().len())?;
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
