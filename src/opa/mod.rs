mod de;
mod error;
mod ser;

pub use de::{from_instance, Deserializer};
pub use error::{Error, Result};
pub use ser::{to_instance, Serializer};

use std::os::raw::*;

use crate::ValueAddr;

const OPA_NULL: c_uchar = 1;
const OPA_BOOLEAN: c_uchar = 2;
const OPA_NUMBER: c_uchar = 3;
const OPA_STRING: c_uchar = 4;
const OPA_ARRAY: c_uchar = 5;
const OPA_OBJECT: c_uchar = 6;
const OPA_SET: c_uchar = 7;

const OPA_NUMBER_REPR_INT: c_uchar = 1;
const OPA_NUMBER_REPR_FLOAT: c_uchar = 2;
const OPA_NUMBER_REPR_REF: c_uchar = 3;

// wasm is 32-bit and doesn't support unsigned ints
type size_t = c_int;
type intptr_t = c_int;

const NULL: opa_value = opa_value { ty: OPA_NULL };

pub trait ToBytes: Sized {
    fn as_slice(&self) -> &[Self] {
        unsafe { std::slice::from_raw_parts(self as *const Self, 1) }
    }

    fn as_bytes(&self) -> &[u8] {
        let slice = self.as_slice();
        unsafe {
            std::slice::from_raw_parts(
                slice.as_ptr() as *const _,
                slice.len() * std::mem::size_of::<Self>(),
            )
        }
    }
}

impl ToBytes for opa_value {}
impl ToBytes for opa_boolean_t {}
impl ToBytes for opa_number_t {}
impl ToBytes for opa_string_t {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct opa_value {
    pub ty: c_uchar,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct opa_boolean_t {
    pub hdr: opa_value,
    pub v: c_int,
}

impl opa_boolean_t {
    pub fn new(b: bool) -> Self {
        let v = if b { 1 } else { 0 };
        let hdr = opa_value { ty: OPA_BOOLEAN };
        Self { hdr, v }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct opa_number_ref_t {
    pub s: intptr_t,
    pub len: size_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union opa_number_variant_t {
    pub i: c_longlong,
    pub f: c_double,
    pub r: opa_number_ref_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opa_number_t {
    pub hdr: opa_value,
    pub repr: c_uchar,
    pub v: opa_number_variant_t,
}

impl opa_number_t {
    pub fn from_i64(i: i64) -> Self {
        let hdr = opa_value { ty: OPA_NUMBER };
        let v = opa_number_variant_t { i };
        opa_number_t {
            hdr,
            repr: OPA_NUMBER_REPR_INT,
            v,
        }
    }

    pub fn from_f64(f: f64) -> Self {
        let hdr = opa_value { ty: OPA_NUMBER };
        let v = opa_number_variant_t { f };
        opa_number_t {
            hdr,
            repr: OPA_NUMBER_REPR_FLOAT,
            v,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct opa_string_t {
    pub hdr: opa_value,
    pub free: c_uchar,
    pub len: size_t,
    pub v: intptr_t,
}

impl opa_string_t {
    pub fn from_str(s: &str, data: ValueAddr) -> Self {
        let hdr = opa_value { ty: OPA_STRING };
        let free = 0 as c_uchar;
        let len = s.len() as size_t;
        opa_string_t {
            hdr,
            free,
            len,
            v: data.0 as intptr_t,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct opa_array_elem_t {
    pub i: intptr_t,
    pub v: intptr_t,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct opa_array_t {
    pub hdr: opa_value,
    pub elems: intptr_t,
    pub len: size_t,
    pub cap: size_t,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct opa_object_elem_t {
    pub k: intptr_t,
    pub v: intptr_t,
    pub next: intptr_t,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct opa_object_t {
    pub hdr: opa_value,
    pub head: intptr_t,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct opa_set_elem_t {
    pub v: intptr_t,
    pub next: intptr_t,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct opa_set_t {
    pub hdr: opa_value,
    pub head: intptr_t,
}

#[cfg(test)]
mod tests {
    use std::mem;

    use super::*;

    #[test]
    fn test_bool_size() {
        assert_eq!(8, mem::size_of::<opa_boolean_t>());
    }

    #[test]
    fn test_number_ref_size() {
        assert_eq!(8, mem::size_of::<opa_number_ref_t>());
    }
}
