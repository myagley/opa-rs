#![allow(dead_code)]

use std::mem;

use serde::{ser, Serialize};

use crate::opa::{Error, Result};
use crate::wasm::Instance;
use crate::ValueAddr;

use super::*;

pub fn to_instance<T>(instance: Instance, value: &T) -> Result<ValueAddr>
where
    T: ?Sized + ser::Serialize,
{
    let mut serializer = Serializer { instance };
    let addr = value.serialize(&mut serializer)?;
    Ok(addr)
}

pub struct Serializer {
    instance: Instance,
}

impl Serializer {
    fn alloc(&self, size: usize) -> Result<ValueAddr> {
        self.instance
            .functions()
            .malloc(size)
            .map_err(|_| Error::Alloc)
    }

    fn memset(&self, addr: ValueAddr, bytes: &[u8]) -> Result<()> {
        self.instance
            .memory()
            .set(addr, bytes)
            .map_err(|_| Error::MemSet)
    }

    fn store<T: AsBytes + ?Sized>(&self, value: &T) -> Result<ValueAddr> {
        let addr = self
            .instance
            .functions()
            .malloc(value.as_bytes().len())
            .map_err(|_| Error::Alloc)?;
        self.instance
            .memory()
            .set(addr, value.as_bytes())
            .map_err(|_| Error::MemSet)?;
        Ok(addr)
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ValueAddr;
    type Error = Error;

    type SerializeSeq = ArraySerializer<'a>;
    type SerializeTuple = ArraySerializer<'a>;
    type SerializeTupleStruct = ArraySerializer<'a>;
    type SerializeTupleVariant = TupleVariantSerializer<'a>;
    type SerializeMap = ObjectSerializer<'a>;
    type SerializeStruct = ObjectSerializer<'a>;
    type SerializeStructVariant = StructVariantSerializer<'a>;

    fn serialize_bool(self, v: bool) -> Result<ValueAddr> {
        self.store(&opa_boolean_t::new(v))
    }

    fn serialize_i8(self, v: i8) -> Result<ValueAddr> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<ValueAddr> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<ValueAddr> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<ValueAddr> {
        self.store(&opa_number_t::from_i64(v))
    }

    fn serialize_u8(self, v: u8) -> Result<ValueAddr> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<ValueAddr> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<ValueAddr> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<ValueAddr> {
        self.serialize_i64(v as i64)
    }

    fn serialize_f32(self, v: f32) -> Result<ValueAddr> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<ValueAddr> {
        self.store(&opa_number_t::from_f64(v))
    }

    fn serialize_char(self, v: char) -> Result<ValueAddr> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<ValueAddr> {
        let data_addr = self.store(v)?;
        let s = opa_string_t::from_str(v, data_addr);
        self.store(&s)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<ValueAddr> {
        use serde::ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    fn serialize_none(self) -> Result<ValueAddr> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<ValueAddr>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<ValueAddr> {
        self.store(&NULL)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<ValueAddr> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<ValueAddr> {
        variant.serialize(self)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<ValueAddr>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<ValueAddr>
    where
        T: ?Sized + Serialize,
    {
        use serde::ser::SerializeMap;
        let mut mapser = self.serialize_map(Some(1))?;
        mapser.serialize_entry(variant, value)?;
        let addr = mapser.end()?;
        Ok(addr)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        if let Some(len) = len {
            let elems_addr = self.alloc(len * mem::size_of::<opa_array_elem_t>())?;
            let array = opa_array_t::new(elems_addr, len);
            let addr = self.store(&array)?;

            let serializer = ArraySerializer {
                ser: self,
                count: 0,
                len,
                addr,
                elems_addr,
            };
            Ok(serializer)
        } else {
            Err(Error::ExpectedSeqLen)
        }
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        let instance = self.instance.clone();
        let variant_addr = variant.serialize(&mut *self)?;

        let elem = opa_object_elem_t {
            k: variant_addr.0 as intptr_t,
            v: 0,
            next: 0,
        };
        let elem_addr = self.store(&elem)?;

        let obj = opa_object_t::new(elem_addr);
        let addr = self.store(&obj)?;

        let seq = self.serialize_seq(Some(len))?;

        let serializer = TupleVariantSerializer {
            instance,
            seq,
            addr,
            elem_addr,
        };
        Ok(serializer)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        let obj = opa_object_t::new(ValueAddr(0));
        let addr = self.store(&obj)?;

        let elem = opa_object_elem_t {
            k: 0,
            v: 0,
            next: 0,
        };
        let serializer = ObjectSerializer {
            ser: self,
            addr,
            elem,
            prev_elem: addr,
            first: true,
        };
        Ok(serializer)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        let instance = self.instance.clone();
        let variant_addr = variant.serialize(&mut *self)?;
        let elem = opa_object_elem_t {
            k: variant_addr.0 as intptr_t,
            v: 0,
            next: 0,
        };
        let elem_addr = self.store(&elem)?;

        let obj = opa_object_t::new(elem_addr);
        let addr = self.store(&obj)?;

        let obj = self.serialize_map(Some(len))?;
        let serializer = StructVariantSerializer {
            instance,
            obj,
            addr,
            elem_addr,
        };
        Ok(serializer)
    }
}

pub struct ArraySerializer<'a> {
    ser: &'a mut Serializer,
    count: usize,
    len: usize,
    addr: ValueAddr,
    elems_addr: ValueAddr,
}

impl<'a> ser::SerializeSeq for ArraySerializer<'a> {
    type Ok = ValueAddr;

    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        // store the index
        let i_addr = self.count.serialize(&mut *self.ser)?;

        // store the value
        let v_addr = value.serialize(&mut *self.ser)?;

        // store the elem
        let elem = opa_array_elem_t {
            i: i_addr.0 as intptr_t,
            v: v_addr.0 as intptr_t,
        };
        self.ser.memset(
            self.elems_addr + self.count * mem::size_of::<opa_array_elem_t>(),
            elem.as_bytes(),
        )?;

        // bump the count for the next element
        self.count = self.count + 1;
        Ok(())
    }

    fn end(self) -> Result<ValueAddr> {
        if self.count != self.len {
            return Err(Error::InvalidSeqLen(self.len, self.count));
        }
        Ok(self.addr)
    }
}

// Same thing but for tuples.
impl<'a> ser::SerializeTuple for ArraySerializer<'a> {
    type Ok = ValueAddr;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<ValueAddr> {
        ser::SerializeSeq::end(self)
    }
}

// Same thing but for tuple structs
impl<'a> ser::SerializeTupleStruct for ArraySerializer<'a> {
    type Ok = ValueAddr;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<ValueAddr> {
        ser::SerializeSeq::end(self)
    }
}

pub struct TupleVariantSerializer<'a> {
    instance: Instance,
    seq: ArraySerializer<'a>,
    addr: ValueAddr,
    elem_addr: ValueAddr,
}

// Tuple variants are a little different. Refer back to the
// `serialize_tuple_variant` method above:
//
//    self.output += "{";
//    variant.serialize(&mut *self)?;
//    self.output += ":[";
//
// So the `end` method in this impl is responsible for closing both the `]` and
// the `}`.
impl<'a> ser::SerializeTupleVariant for TupleVariantSerializer<'a> {
    type Ok = ValueAddr;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        use serde::ser::SerializeSeq;
        self.seq.serialize_element(value)
    }

    fn end(self) -> Result<ValueAddr> {
        use serde::ser::SerializeSeq;
        let seq_addr = self.seq.end()?;
        self.instance
            .memory()
            .as_type_mut::<opa_object_elem_t>(self.elem_addr)?
            .v = seq_addr.0 as intptr_t;
        Ok(self.addr)
    }
}

pub struct ObjectSerializer<'a> {
    ser: &'a mut Serializer,
    addr: ValueAddr,
    elem: opa_object_elem_t,
    prev_elem: ValueAddr,
    first: bool,
}

// Some `Serialize` types are not able to hold a key and value in memory at the
// same time so `SerializeMap` implementations are required to support
// `serialize_key` and `serialize_value` individually.
//
// There is a third optional method on the `SerializeMap` trait. The
// `serialize_entry` method allows serializers to optimize for the case where
// key and value are both available simultaneously. In JSON it doesn't make a
// difference so the default behavior for `serialize_entry` is fine.
impl<'a> ser::SerializeMap for ObjectSerializer<'a> {
    type Ok = ValueAddr;
    type Error = Error;

    // The Serde data model allows map keys to be any serializable type. JSON
    // only allows string keys so the implementation below will produce invalid
    // JSON if the key serializes as something other than a string.
    //
    // A real JSON serializer would need to validate that map keys are strings.
    // This can be done by using a different Serializer to serialize the key
    // (instead of `&mut **self`) and having that other serializer only
    // implement `serialize_str` and return an error on any other data type.
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        // store the key
        let k_addr = key.serialize(&mut *self.ser)?;

        // update the current entry's pointer to this key
        self.elem.k = k_addr.0 as intptr_t;
        Ok(())
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        // store the value
        let v_addr = value.serialize(&mut *self.ser)?;

        // update the current entry's pointer to this value
        self.elem.v = v_addr.0 as intptr_t;

        // store this entry
        let elem_addr = self.ser.store(&self.elem)?;

        if self.first {
            self.ser
                .instance
                .memory()
                .as_type_mut::<opa_object_t>(self.prev_elem)?
                .head = elem_addr.0 as intptr_t;
        } else {
            self.ser
                .instance
                .memory()
                .as_type_mut::<opa_object_elem_t>(self.prev_elem)?
                .next = elem_addr.0 as intptr_t;
        }

        self.first = false;
        self.prev_elem = elem_addr;
        self.elem.k = 0;
        self.elem.v = 0;
        self.elem.next = 0;
        Ok(())
    }

    fn end(self) -> Result<ValueAddr> {
        Ok(self.addr)
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a> ser::SerializeStruct for ObjectSerializer<'a> {
    type Ok = ValueAddr;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<ValueAddr> {
        ser::SerializeMap::end(self)
    }
}

pub struct StructVariantSerializer<'a> {
    instance: Instance,
    obj: ObjectSerializer<'a>,
    addr: ValueAddr,
    elem_addr: ValueAddr,
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a> ser::SerializeStructVariant for StructVariantSerializer<'a> {
    type Ok = ValueAddr;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        use serde::ser::SerializeMap;
        self.obj.serialize_entry(key, value)
    }

    fn end(self) -> Result<ValueAddr> {
        use serde::ser::SerializeMap;
        let obj_addr = self.obj.end()?;
        self.instance
            .memory()
            .as_type_mut::<opa_object_elem_t>(self.elem_addr)?
            .v = obj_addr.0 as intptr_t;
        Ok(self.addr)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;

    use serde::{Deserialize, Serialize};

    use crate::dump_json;
    use crate::opa::to_instance;
    use crate::wasm::{Instance, Memory, Module};

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

    thread_local! {
        static EMPTY_MODULE: Module = {
            let bytes = fs::read("tests/empty.wasm").unwrap();
            let module = Module::from_bytes(bytes).unwrap();
            module
        };
    }

    macro_rules! type_roundtrip {
        ($name:ident, $input:expr, $expected:expr) => {
            #[test]
            fn $name() {
                EMPTY_MODULE.with(|module| {
                    let memory = Memory::from_module(module);
                    let instance = Instance::new(module, memory).unwrap();
                    let addr = to_instance(instance.clone(), &$input).unwrap();
                    let loaded = dump_json(&instance, addr).unwrap();
                    assert_eq!($expected, loaded);
                })
            }
        };
    }

    type_roundtrip!(test_serialize_bool, true, "true");
    type_roundtrip!(test_serialize_i8, 42_i8, "42");
    type_roundtrip!(test_serialize_i16, 42_i16, "42");
    type_roundtrip!(test_serialize_i32, 42_i32, "42");
    type_roundtrip!(test_serialize_i64, 42_i64, "42");
    type_roundtrip!(test_serialize_u8, 42_u8, "42");
    type_roundtrip!(test_serialize_u16, 42_u16, "42");
    type_roundtrip!(test_serialize_u32, 42_u32, "42");
    type_roundtrip!(test_serialize_u64, 42_u64, "42");

    type_roundtrip!(test_serialize_f32, 1.234_f32, "1.23400");
    type_roundtrip!(test_serialize_f64, 1.234_f64, "1.23400");

    type_roundtrip!(test_serialize_str, "hello there", "\"hello there\"");

    type_roundtrip!(test_serialize_unit, (), "null");
    type_roundtrip!(test_serialize_none, Option::<i8>::None, "null");
    type_roundtrip!(test_serialize_unit_variant, TestEnum::Unit, "\"Unit\"");
    type_roundtrip!(
        test_serialize_newtype_variant,
        TestEnum::NewType(64),
        "{\"NewType\":64}"
    );
    type_roundtrip!(
        test_serialize_tuple_variant,
        TestEnum::Tuple(64, "Hello".to_string()),
        "{\"Tuple\":[64,\"Hello\"]}"
    );

    type_roundtrip!(
        test_serialize_vec,
        vec!["hello", "there"],
        "[\"hello\",\"there\"]"
    );

    #[test]
    fn test_serialize_map() {
        EMPTY_MODULE.with(|module| {
            let memory = Memory::from_module(module);
            let instance = Instance::new(module, memory).unwrap();
            let mut input = HashMap::new();
            input.insert("key1", 3);
            input.insert("key2", 2);
            let addr = to_instance(instance.clone(), &input).unwrap();
            let loaded = dump_json(&instance, addr).unwrap();
            let result = serde_json::from_str(&loaded).unwrap();
            assert_eq!(input, result);
        })
    }

    #[test]
    fn test_serialize_empty_map() {
        EMPTY_MODULE.with(|module| {
            let memory = Memory::from_module(module);
            let instance = Instance::new(module, memory).unwrap();
            let input: HashMap<String, i64> = HashMap::new();
            let addr = to_instance(instance.clone(), &input).unwrap();
            let loaded = dump_json(&instance, addr).unwrap();
            let result = serde_json::from_str(&loaded).unwrap();
            assert_eq!(input, result);
        })
    }

    #[test]
    fn test_serialize_struct() {
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
            let addr = to_instance(instance.clone(), &person).unwrap();
            let loaded = dump_json(&instance, addr).unwrap();
            let result = serde_json::from_str(&loaded).unwrap();
            assert_eq!(person, result);
        })
    }

    #[test]
    fn test_serialize_struct_variant() {
        EMPTY_MODULE.with(|module| {
            let memory = Memory::from_module(module);
            let instance = Instance::new(module, memory).unwrap();
            let variant = TestEnum::Struct {
                age: 64,
                msg: "Hello".to_string(),
            };
            let addr = to_instance(instance.clone(), &variant).unwrap();
            let loaded = dump_json(&instance, addr).unwrap();
            let result = serde_json::from_str(&loaded).unwrap();
            assert_eq!(variant, result);
        })
    }
}
