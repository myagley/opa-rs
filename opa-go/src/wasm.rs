use std::os::raw::{c_char, c_void};
use std::path::Path;
use std::{slice, str};

use opa_go_sys::{Free, GoInt, GoSlice, GoString, WasmBuild};

use crate::{Error, GoError};

struct WasmBuildReturn {
    ptr: *const u8,
    len: usize,
}

impl WasmBuildReturn {
    fn into_bytes(self) -> Vec<u8> {
        let bytes = unsafe {
            if self.ptr.is_null() {
                vec![]
            } else {
                let b = slice::from_raw_parts(self.ptr, self.len);
                Vec::from(b)
            }
        };
        bytes
    }
}

impl Drop for WasmBuildReturn {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { Free(self.ptr as *mut c_void) }
        }
    }
}

pub fn compile<P: AsRef<Path>>(query: &str, data: P) -> Result<Vec<u8>, Error> {
    let query = GoString {
        p: query.as_ptr() as *const c_char,
        n: query.len() as isize,
    };

    let data = data.as_ref().to_str().unwrap();
    let mut data = GoString {
        p: data.as_ptr() as *const c_char,
        n: data.len() as isize,
    };
    let data = slice::from_mut(&mut data);
    let data = GoSlice {
        data: data.as_mut_ptr() as *mut c_void,
        len: data.len() as GoInt,
        cap: data.len() as GoInt,
    };

    let bundles = GoSlice {
        data: std::ptr::null_mut() as *mut c_void,
        len: 0,
        cap: 0,
    };

    let ignore = GoSlice {
        data: std::ptr::null_mut() as *mut c_void,
        len: 0,
        cap: 0,
    };

    let bytes = build(query, data, bundles, ignore)?.into_bytes();
    Ok(bytes)
}

fn build(
    query: GoString,
    data: GoSlice,
    bundles: GoSlice,
    ignore: GoSlice,
) -> Result<WasmBuildReturn, Error> {
    let result = unsafe { WasmBuild(query, data, bundles, ignore) };
    if !result.r0.is_null() && !result.r2.is_null() {
        let r = WasmBuildReturn {
            ptr: result.r0 as *const u8,
            len: result.r1 as usize,
        };
        let goe = GoError {
            ptr: result.r2 as *const c_char,
        };
        drop(goe);
        Ok(r)
    } else if !result.r2.is_null() {
        let goe = GoError {
            ptr: result.r2 as *const c_char,
        };
        Err(Error::from(goe))
    } else if !result.r0.is_null() {
        let r = WasmBuildReturn {
            ptr: result.r0 as *const u8,
            len: result.r1 as usize,
        };
        Ok(r)
    } else {
        let message = "Result and error pointers are both null.".to_string();
        let e = Error { message };
        Err(e)
    }
}
