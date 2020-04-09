mod de;
mod error;
mod ser;

pub use de::{from_instance, Deserializer};
pub use error::{Error, Result};
pub use ser::{to_instance, Serializer};

use std::mem;
use std::os::raw::*;

use crate::runtime::{AsBytes, FromBytes};
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

const NULL: opa_value = opa_value { ty: OPA_NULL };

// wasm is 32-bit and doesn't support unsigned ints
#[allow(non_camel_case_types)]
type size_t = c_int;
#[allow(non_camel_case_types)]
type intptr_t = c_int;

macro_rules! as_bytes {
    ($ty:ty) => {
        impl AsBytes for $ty {
            fn as_bytes(&self) -> &[u8] {
                unsafe {
                    let slice = std::slice::from_raw_parts(self as *const Self, 1);
                    std::slice::from_raw_parts(
                        slice.as_ptr() as *const _,
                        slice.len() * std::mem::size_of::<Self>(),
                    )
                }
            }
        }
    };
}

as_bytes!(opa_value);
as_bytes!(opa_boolean_t);
as_bytes!(opa_number_t);
as_bytes!(opa_string_t);
as_bytes!(opa_array_t);
as_bytes!(opa_array_elem_t);
as_bytes!(opa_object_t);
as_bytes!(opa_object_elem_t);
as_bytes!(opa_set_t);
as_bytes!(opa_set_elem_t);

unsafe impl FromBytes for opa_value {}
unsafe impl FromBytes for opa_boolean_t {}
unsafe impl FromBytes for opa_number_t {}
unsafe impl FromBytes for opa_string_t {}
unsafe impl FromBytes for opa_array_t {}
unsafe impl FromBytes for opa_array_elem_t {}
unsafe impl FromBytes for opa_object_t {}
unsafe impl FromBytes for opa_object_elem_t {}
unsafe impl FromBytes for opa_set_t {}
unsafe impl FromBytes for opa_set_elem_t {}

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

    pub fn from_str(s: &str, data: ValueAddr) -> Self {
        let hdr = opa_value { ty: OPA_NUMBER };
        let len = s.len() as size_t;
        let r = opa_number_ref_t {
            s: data.0 as intptr_t,
            len,
        };
        let v = opa_number_variant_t { r };
        opa_number_t {
            hdr,
            repr: OPA_NUMBER_REPR_REF,
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

impl opa_array_t {
    pub fn new(elems: ValueAddr, len: usize) -> Self {
        let hdr = opa_value { ty: OPA_ARRAY };
        Self {
            hdr,
            elems: elems.0 as intptr_t,
            len: len as size_t,
            cap: 0 as size_t,
        }
    }
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

impl opa_object_t {
    pub fn new(head: ValueAddr) -> Self {
        let hdr = opa_value { ty: OPA_OBJECT };
        Self {
            hdr,
            head: head.0 as i32,
        }
    }
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

impl opa_set_t {
    pub fn new(head: ValueAddr) -> Self {
        let hdr = opa_value { ty: OPA_SET };
        Self {
            hdr,
            head: head.0 as i32,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::mem;

    use proptest::prelude::*;
    use serde::{Deserialize, Serialize};

    use crate::opa_serde::to_instance;
    use crate::runtime::{Instance, Memory, Module};
    use crate::value::{self, Number, Value};

    use super::*;

    thread_local! {
        static EMPTY_MODULE: Module = {
            let bytes = fs::read("tests/empty.wasm").unwrap();
            let module = Module::from_bytes(bytes).unwrap();
            module
        };
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct UnitStruct;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct NewTypeStruct(i64);

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TupleStruct(i64, String);

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum TestEnum {
        Unit,
        NewType(i64),
        Tuple(i64, String),
        Struct { age: i64, msg: String },
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Person {
        name: String,
        age: u8,
        properties: HashMap<String, String>,
    }

    #[test]
    fn test_bool_size() {
        assert_eq!(8, mem::size_of::<opa_boolean_t>());
    }

    #[test]
    fn test_number_ref_size() {
        assert_eq!(8, mem::size_of::<opa_number_ref_t>());
    }

    macro_rules! type_roundtrip {
        ($name:ident, $ty:ty, $input:expr) => {
            #[test]
            fn $name() {
                EMPTY_MODULE.with(|module| {
                    let memory = Memory::from_module(module);
                    let instance = Instance::new(module, memory).unwrap();
                    let addr = to_instance(&instance, &$input).unwrap();
                    let loaded = from_instance::<$ty>(&instance, addr).unwrap();
                    assert_eq!($input, loaded);
                })
            }
        };
    }

    type_roundtrip!(test_roundtrip_bool, bool, true);
    type_roundtrip!(test_roundtrip_i8, i8, 42_i8);
    type_roundtrip!(test_roundtrip_i16, i16, 42_i16);
    type_roundtrip!(test_roundtrip_i32, i32, 42_i32);
    type_roundtrip!(test_roundtrip_i64, i64, 42_i64);
    type_roundtrip!(test_roundtrip_u8, u8, 42_u8);
    type_roundtrip!(test_roundtrip_u16, u16, 42_u16);
    type_roundtrip!(test_roundtrip_u32, u32, 42_u32);
    type_roundtrip!(test_roundtrip_u64, u64, 42_u64);
    type_roundtrip!(test_roundtrip_f32, f32, 1.234_f32);
    type_roundtrip!(test_roundtrip_f64, f64, 1.234_f64);

    type_roundtrip!(test_roundtrip_string, String, "hello there".to_string());
    type_roundtrip!(test_roundtrip_char, char, 'a');
    type_roundtrip!(test_roundtrip_none, Option<i64>, Option::<i64>::None);
    type_roundtrip!(test_roundtrip_some, Option<i64>, Some(56));
    type_roundtrip!(test_roundtrip_unit_struct, UnitStruct, UnitStruct);
    type_roundtrip!(
        test_roundtrip_newtype_struct,
        NewTypeStruct,
        NewTypeStruct(56)
    );
    type_roundtrip!(test_roundtrip_unit_variant, TestEnum, TestEnum::Unit);
    type_roundtrip!(
        test_roundtrip_newtype_variant,
        TestEnum,
        TestEnum::NewType(64)
    );
    type_roundtrip!(
        test_roundtrip_tuple_variant,
        TestEnum,
        TestEnum::Tuple(64, "Hello".to_string())
    );
    type_roundtrip!(
        test_roundtrip_struct_variant,
        TestEnum,
        TestEnum::Struct {
            age: 64,
            msg: "Hello".to_string()
        }
    );

    type_roundtrip!(
        test_roundtrip_vec,
        Vec<String>,
        vec!["hello".to_string(), "there".to_string()]
    );
    type_roundtrip!(
        test_roundtrip_tuple,
        (i64, String),
        (42, "hello".to_string())
    );
    type_roundtrip!(
        test_roundtrip_tuple_struct,
        TupleStruct,
        TupleStruct(42, "hello".to_string())
    );

    #[test]
    fn test_roundtrip_map() {
        EMPTY_MODULE.with(|module| {
            let memory = Memory::from_module(module);
            let instance = Instance::new(module, memory).unwrap();
            let mut input = HashMap::new();
            input.insert("key1".to_string(), 3);
            input.insert("key2".to_string(), 2);
            let addr = to_instance(&instance, &input).unwrap();
            let loaded = from_instance(&instance, addr).unwrap();
            assert_eq!(input, loaded);
        })
    }

    #[test]
    fn test_roundtrip_empty_map() {
        EMPTY_MODULE.with(|module| {
            let memory = Memory::from_module(module);
            let instance = Instance::new(module, memory).unwrap();
            let input: HashMap<String, i64> = HashMap::new();
            let addr = to_instance(&instance, &input).unwrap();
            let loaded = from_instance(&instance, addr).unwrap();
            assert_eq!(input, loaded);
        })
    }

    #[test]
    fn test_roundtrip_struct() {
        EMPTY_MODULE.with(|module| {
            let memory = Memory::from_module(module);
            let instance = Instance::new(module, memory).unwrap();
            let mut properties = HashMap::new();
            properties.insert("height".to_string(), "50".to_string());
            properties.insert("mood".to_string(), "happy".to_string());
            let person = Person {
                name: "thename".to_string(),
                age: 42,
                properties,
            };
            let addr = to_instance(&instance, &person).unwrap();
            let loaded = from_instance(&instance, addr).unwrap();
            assert_eq!(person, loaded);
        })
    }

    #[test]
    fn test_roundtrip_unit() {
        EMPTY_MODULE.with(|module| {
            let memory = Memory::from_module(module);
            let instance = Instance::new(module, memory).unwrap();
            let input = ();
            let addr = to_instance(&instance, &input).unwrap();
            let loaded = from_instance(&instance, addr).unwrap();
            assert_eq!(input, loaded);
        })
    }

    // Value tests
    #[test]
    fn test_roundtrip_value_object() {
        EMPTY_MODULE.with(|module| {
            let memory = Memory::from_module(module);
            let instance = Instance::new(module, memory).unwrap();
            let mut input = value::Map::new();
            input.insert("key1".to_string(), Value::Number(3.into()));
            input.insert("key2".to_string(), Value::Bool(true));
            let input = Value::Object(input);
            let addr = to_instance(&instance, &input).unwrap();
            let loaded = from_instance(&instance, addr).unwrap();
            assert_eq!(input, loaded);
        })
    }

    #[test]
    fn test_roundtrip_value_set() {
        EMPTY_MODULE.with(|module| {
            let memory = Memory::from_module(module);
            let instance = Instance::new(module, memory).unwrap();
            let mut input = value::Set::new();
            input.insert(Value::String("key1".to_string()));
            input.insert(Value::String("key2".to_string()));
            let input = Value::Set(input);
            let addr = to_instance(&instance, &input).unwrap();
            let loaded = from_instance(&instance, addr).unwrap();
            assert_eq!(input, loaded);
        })
    }

    fn arb_number() -> impl Strategy<Value = Number> {
        prop_oneof![
            prop::num::i64::ANY.prop_map(Number::from),
            prop::num::i64::ANY.prop_map(|i| Number::from(i.to_string())),
            prop::num::f64::ANY.prop_map(Number::from),
            prop::num::f64::ANY.prop_map(|f| Number::from(f.to_string())),
        ]
    }

    fn arb_value() -> impl Strategy<Value = Value> {
        let leaf = prop_oneof![
            Just(Value::Null),
            any::<bool>().prop_map(Value::Bool),
            arb_number().prop_map(Value::Number),
            ".*".prop_map(Value::String),
        ];
        leaf.prop_recursive(
            8,   // 8 levels deep
            256, // Shoot for maximum size of 256 nodes
            10,  // We put up to 10 items per collection
            |inner| {
                prop_oneof![
                    prop::collection::vec(inner.clone(), 0..10).prop_map(Value::Array),
                    prop::collection::btree_map(".*", inner.clone(), 0..10).prop_map(Value::Object),
                    prop::collection::btree_set(inner.clone(), 0..10).prop_map(Value::Set),
                ]
            },
        )
    }

    proptest! {
        #[test]
        fn test_roundtrip_value(input in arb_value()) {
            EMPTY_MODULE.with(|module| {
                let memory = Memory::from_module(module);
                let instance = Instance::new(module, memory).unwrap();
                let addr = to_instance(&instance, &input).unwrap();
                let loaded = from_instance(&instance, addr).unwrap();
                assert_eq!(input, loaded);
            })
        }
    }
}
