use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::{error, fmt};

pub mod wasm;

use opa_go_sys::Free;

#[derive(Debug)]
pub struct Error {
    message: String,
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
