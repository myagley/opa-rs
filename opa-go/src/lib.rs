use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::{error, fmt};

use opa_go_sys::*;
use serde::Serialize;

pub mod wasm;

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error compiling to wasm: {}", self.message)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

struct GoError {
    ptr: *const c_char,
}

impl Drop for GoError {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { Free(self.ptr as *mut c_void) }
        }
    }
}

impl From<GoError> for Error {
    fn from(error: GoError) -> Self {
        let message = unsafe { CStr::from_ptr(error.ptr).to_string_lossy().into_owned() };
        Self { message }
    }
}

pub struct Rego {
    id: u64,
}

impl Rego {
    pub fn new(query: &str, module_name: &str, module_contents: &str) -> Result<Self, Error> {
        let query = GoString {
            p: query.as_ptr() as *const c_char,
            n: query.len() as isize,
        };

        let module_name = GoString {
            p: module_name.as_ptr() as *const c_char,
            n: module_name.len() as isize,
        };

        let module_contents = GoString {
            p: module_contents.as_ptr() as *const c_char,
            n: module_contents.len() as isize,
        };

        let result = unsafe { RegoNew(query, module_name, module_contents) };
        if !result.r1.is_null() {
            let e = GoError {
                ptr: result.r1 as *const c_char,
            };
            return Err(Error::from(e));
        }

        let rego = Self { id: result.r0 };
        Ok(rego)
    }

    pub fn eval_bool<T: Serialize>(&self, input: &T) -> Result<bool, Error> {
        let serialized = serde_json::to_string(input).map_err(|e| Error::new(e.to_string()))?;
        let input = GoString {
            p: serialized.as_ptr() as *const c_char,
            n: serialized.len() as isize,
        };
        let result = unsafe { RegoEvalBool(self.id, input) };
        if !result.r1.is_null() {
            let e = GoError {
                ptr: result.r1 as *const c_char,
            };
            return Err(Error::from(e));
        }
        Ok(result.r0 != 0)
    }
}

impl Drop for Rego {
    fn drop(&mut self) {
        unsafe { RegoDrop(self.id) }
    }
}
