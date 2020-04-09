use std::path::Path;
use std::{fmt, ops, process};

use serde::Serialize;
use tempfile::TempDir;

mod builtins;
mod error;
mod opa_serde;
mod runtime;
pub mod set;
pub mod value;

use runtime::{Instance, Memory, Module};

pub use error::Error;
pub use value::{Map, Number, Value};

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

impl ops::Add<usize> for ValueAddr {
    type Output = ValueAddr;

    fn add(self, rhs: usize) -> Self {
        ValueAddr(self.0 + rhs as i32)
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
        let initial = Value::Object(Map::new());
        let data_addr = opa_serde::to_instance(&instance, &initial)?;

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
        let input_addr = opa_serde::to_instance(&self.instance, input)?;

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
        let v = opa_serde::from_instance(&self.instance, result_addr)?;
        Ok(v)
    }

    pub fn set_data<T: Serialize>(&mut self, data: &T) -> Result<(), Error> {
        self.instance.functions().heap_ptr_set(self.base_heap_ptr)?;
        self.instance.functions().heap_top_set(self.base_heap_top)?;
        self.data_addr = opa_serde::to_instance(&self.instance, data)?;
        self.data_heap_ptr = self.instance.functions().heap_ptr_get()?;
        self.data_heap_top = self.instance.functions().heap_top_get()?;
        Ok(())
    }

    // TODO: add proper parsing here
    // pub fn builtins(&mut self) -> Result<String, Error> {
    //     let addr = self.instance.functions().builtins()?;
    //     let s = dump_json(&self.instance, addr)?;
    //     Ok(s)
    // }
}

fn abort(_a: i32) {
    println!("abort");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_types() {
        let mut policy = Policy::from_rego("tests/types.rego", "data.tests.types").unwrap();
        let result = policy.evaluate(&Value::Null).unwrap();
        assert_eq!(1, result.as_set().unwrap().len());
    }
}
