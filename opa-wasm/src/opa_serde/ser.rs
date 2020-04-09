#![allow(dead_code)]

use std::mem;

use serde::{ser, Serialize};

use crate::opa_serde::{Error, Result};
use crate::runtime::Instance;
use crate::value::number;
use crate::{set, ValueAddr};

use super::*;

pub fn to_instance<T>(instance: &Instance, value: &T) -> Result<ValueAddr>
where
    T: ?Sized + ser::Serialize,
{
    let mut serializer = Serializer { instance };
    let addr = value.serialize(&mut serializer)?;
    Ok(addr)
}

pub struct Serializer<'i> {
    instance: &'i Instance,
}

impl<'i> Serializer<'i> {
    fn alloc(&self, size: usize) -> Result<ValueAddr> {
        self.instance
            .functions()
            .malloc(size)
            .map_err(|e| Error::Alloc(Box::new(e)))
    }

    fn memset(&self, addr: ValueAddr, bytes: &[u8]) -> Result<()> {
        self.instance
            .memory()
            .set(addr, &bytes)
            .map_err(|e| Error::MemSet(Box::new(e)))
    }

    fn store<T: AsBytes + ?Sized>(&self, value: &T) -> Result<ValueAddr> {
        let addr = self.alloc(value.as_bytes().len())?;
        self.memset(addr, value.as_bytes())?;
        Ok(addr)
    }
}

impl<'a, 'i> ser::Serializer for &'a mut Serializer<'i> {
    type Ok = ValueAddr;
    type Error = Error;

    type SerializeSeq = ArraySerializer<'a, 'i>;
    type SerializeTuple = ArraySerializer<'a, 'i>;
    type SerializeTupleStruct = ArraySerializer<'a, 'i>;
    type SerializeTupleVariant = TupleVariantSerializer<'a, 'i>;
    type SerializeMap = ObjectSerializer<'a, 'i>;
    type SerializeStruct = StructSerializer<'a, 'i>;
    type SerializeStructVariant = StructVariantSerializer<'a, 'i>;

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
            let serializer = ArraySerializer::from_serializer(self, len)?;
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
        let serializer = TupleVariantSerializer::from_serializer(self, variant, len)?;
        Ok(serializer)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        let serializer = ObjectSerializer::from_serializer(self)?;
        Ok(serializer)
    }

    fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        let serializer = if name == set::TOKEN {
            StructSerializer::Set(self, None)
        } else if name == number::TOKEN {
            StructSerializer::NumberRef(self, None)
        } else {
            StructSerializer::Object(ObjectSerializer::from_serializer(self)?)
        };
        Ok(serializer)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        let serializer = StructVariantSerializer::from_serializer(self, variant, len)?;
        Ok(serializer)
    }
}

pub struct ArraySerializer<'a, 'i: 'a> {
    ser: &'a mut Serializer<'i>,
    count: usize,
    len: usize,
    addr: ValueAddr,
    elems_addr: ValueAddr,
}

impl<'a, 'i: 'a> ArraySerializer<'a, 'i> {
    pub fn from_serializer(ser: &'a mut Serializer<'i>, len: usize) -> Result<Self> {
        let elems_addr = ser.alloc(len * mem::size_of::<opa_array_elem_t>())?;
        let array = opa_array_t::new(elems_addr, len);
        let addr = ser.store(&array)?;

        let serializer = ArraySerializer {
            ser,
            count: 0,
            len,
            addr,
            elems_addr,
        };
        Ok(serializer)
    }
}

impl<'i, 'a> ser::SerializeSeq for ArraySerializer<'a, 'i> {
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
impl<'i, 'a> ser::SerializeTuple for ArraySerializer<'a, 'i> {
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
impl<'i, 'a> ser::SerializeTupleStruct for ArraySerializer<'a, 'i> {
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

pub struct SetSerializer<'a, 'i: 'a> {
    ser: &'a mut Serializer<'i>,
    addr: ValueAddr,
    prev_elem: ValueAddr,
    first: bool,
}

impl<'a, 'i: 'a> SetSerializer<'a, 'i> {
    pub fn from_serializer(ser: &'a mut Serializer<'i>) -> Result<Self> {
        let obj = opa_set_t::new(ValueAddr(0));
        let addr = ser.store(&obj)?;

        let serializer = SetSerializer {
            ser,
            addr,
            prev_elem: addr,
            first: true,
        };
        Ok(serializer)
    }
}

impl<'i, 'a> ser::SerializeSeq for SetSerializer<'a, 'i> {
    type Ok = ValueAddr;

    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        // store the value
        let v_addr = value.serialize(&mut *self.ser)?;

        // store the elem
        let elem = opa_set_elem_t {
            v: v_addr.0 as intptr_t,
            next: 0,
        };

        // store the elem
        let elem_addr = self.ser.store(&elem)?;

        if self.first {
            let mut prev_elem = self
                .ser
                .instance
                .memory()
                .get::<opa_set_t>(self.prev_elem)?;
            prev_elem.head = elem_addr.0 as intptr_t;
            self.ser.instance.memory().set(self.prev_elem, &prev_elem)?;
        } else {
            let mut prev_elem = self
                .ser
                .instance
                .memory()
                .get::<opa_set_elem_t>(self.prev_elem)?;
            prev_elem.next = elem_addr.0 as intptr_t;
            self.ser.instance.memory().set(self.prev_elem, &prev_elem)?;
        }

        self.first = false;
        self.prev_elem = elem_addr;
        Ok(())
    }

    fn end(self) -> Result<ValueAddr> {
        Ok(self.addr)
    }
}

pub struct TupleVariantSerializer<'a, 'i> {
    seq: ArraySerializer<'a, 'i>,
    addr: ValueAddr,
    elem_addr: ValueAddr,
}

impl<'a, 'i: 'a> TupleVariantSerializer<'a, 'i> {
    pub fn from_serializer(
        ser: &'a mut Serializer<'i>,
        variant: &'static str,
        len: usize,
    ) -> Result<Self> {
        use serde::ser::Serializer;

        let variant_addr = variant.serialize(&mut *ser)?;

        let elem = opa_object_elem_t {
            k: variant_addr.0 as intptr_t,
            v: 0,
            next: 0,
        };
        let elem_addr = ser.store(&elem)?;

        let obj = opa_object_t::new(elem_addr);
        let addr = ser.store(&obj)?;

        let seq = ser.serialize_seq(Some(len))?;

        let serializer = TupleVariantSerializer {
            seq,
            addr,
            elem_addr,
        };
        Ok(serializer)
    }
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
impl<'i, 'a> ser::SerializeTupleVariant for TupleVariantSerializer<'a, 'i> {
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
        let instance = self.seq.ser.instance.clone();
        let seq_addr = self.seq.end()?;
        let mut elem = instance.memory().get::<opa_object_elem_t>(self.elem_addr)?;
        elem.v = seq_addr.0 as intptr_t;
        instance.memory().set(self.elem_addr, &elem)?;
        Ok(self.addr)
    }
}

pub struct ObjectSerializer<'a, 'i: 'a> {
    ser: &'a mut Serializer<'i>,
    addr: ValueAddr,
    elem: opa_object_elem_t,
    prev_elem: ValueAddr,
    first: bool,
}

impl<'a, 'i: 'a> ObjectSerializer<'a, 'i> {
    pub fn from_serializer(ser: &'a mut Serializer<'i>) -> Result<Self> {
        let obj = opa_object_t::new(ValueAddr(0));
        let addr = ser.store(&obj)?;

        let elem = opa_object_elem_t {
            k: 0,
            v: 0,
            next: 0,
        };
        let serializer = ObjectSerializer {
            ser,
            addr,
            elem,
            prev_elem: addr,
            first: true,
        };
        Ok(serializer)
    }
}

// Some `Serialize` types are not able to hold a key and value in memory at the
// same time so `SerializeMap` implementations are required to support
// `serialize_key` and `serialize_value` individually.
//
// There is a third optional method on the `SerializeMap` trait. The
// `serialize_entry` method allows serializers to optimize for the case where
// key and value are both available simultaneously. In JSON it doesn't make a
// difference so the default behavior for `serialize_entry` is fine.
impl<'i, 'a> ser::SerializeMap for ObjectSerializer<'a, 'i> {
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
            let mut prev_elem = self
                .ser
                .instance
                .memory()
                .get::<opa_object_t>(self.prev_elem)?;
            prev_elem.head = elem_addr.0 as intptr_t;
            self.ser.instance.memory().set(self.prev_elem, &prev_elem)?;
        } else {
            let mut prev_elem = self
                .ser
                .instance
                .memory()
                .get::<opa_object_elem_t>(self.prev_elem)?;
            prev_elem.next = elem_addr.0 as intptr_t;
            self.ser.instance.memory().set(self.prev_elem, &prev_elem)?;
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
impl<'i, 'a> ser::SerializeStruct for ObjectSerializer<'a, 'i> {
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

pub struct StructVariantSerializer<'a, 'i: 'a> {
    obj: ObjectSerializer<'a, 'i>,
    addr: ValueAddr,
    elem_addr: ValueAddr,
}

impl<'a, 'i: 'a> StructVariantSerializer<'a, 'i> {
    pub fn from_serializer(
        ser: &'a mut Serializer<'i>,
        variant: &'static str,
        len: usize,
    ) -> Result<Self> {
        use serde::ser::Serializer;

        let variant_addr = variant.serialize(&mut *ser)?;
        let elem = opa_object_elem_t {
            k: variant_addr.0 as intptr_t,
            v: 0,
            next: 0,
        };
        let elem_addr = ser.store(&elem)?;

        let obj = opa_object_t::new(elem_addr);
        let addr = ser.store(&obj)?;

        let obj = ser.serialize_map(Some(len))?;
        let serializer = StructVariantSerializer {
            obj,
            addr,
            elem_addr,
        };
        Ok(serializer)
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'i, 'a> ser::SerializeStructVariant for StructVariantSerializer<'a, 'i> {
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
        let instance = self.obj.ser.instance.clone();
        let obj_addr = self.obj.end()?;
        let mut elem = instance.memory().get::<opa_object_elem_t>(self.elem_addr)?;
        elem.v = obj_addr.0 as intptr_t;
        instance.memory().set(self.elem_addr, &elem)?;
        Ok(self.addr)
    }
}

pub enum StructSerializer<'a, 'i: 'a> {
    Set(&'a mut Serializer<'i>, Option<ValueAddr>),
    Object(ObjectSerializer<'a, 'i>),
    NumberRef(&'a mut Serializer<'i>, Option<ValueAddr>),
}

impl<'a, 'i> ser::SerializeStruct for StructSerializer<'a, 'i> {
    type Ok = ValueAddr;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match *self {
            StructSerializer::Set(ref mut ser, ref mut a) => {
                if key == set::TOKEN {
                    let addr = value.serialize(SetEmitter(ser))?;
                    a.replace(addr);
                    Ok(())
                } else {
                    return Err(Error::SetInvalid);
                }
            }
            StructSerializer::NumberRef(ref mut ser, ref mut a) => {
                if key == number::TOKEN {
                    let addr = value.serialize(NumberRefEmitter(ser))?;
                    a.replace(addr);
                    Ok(())
                } else {
                    return Err(Error::NumberRefInvalid);
                }
            }
            StructSerializer::Object(ref mut obj) => {
                ser::SerializeStruct::serialize_field(obj, key, value)
            }
        }
    }

    fn end(self) -> Result<ValueAddr> {
        match self {
            StructSerializer::Set(_s, addr) => addr.ok_or_else(|| Error::ExpectedField(set::TOKEN)),
            StructSerializer::NumberRef(_n, addr) => {
                addr.ok_or_else(|| Error::ExpectedField(number::TOKEN))
            }
            StructSerializer::Object(obj) => ser::SerializeStruct::end(obj),
        }
    }
}

struct SetEmitter<'a, 'i: 'a>(&'a mut Serializer<'i>);

impl<'a, 'i> ser::Serializer for SetEmitter<'a, 'i> {
    type Ok = ValueAddr;
    type Error = Error;

    type SerializeSeq = SetSerializer<'a, 'i>;
    type SerializeTuple = ser::Impossible<ValueAddr, Error>;
    type SerializeTupleStruct = ser::Impossible<ValueAddr, Error>;
    type SerializeTupleVariant = ser::Impossible<ValueAddr, Error>;
    type SerializeMap = ser::Impossible<ValueAddr, Error>;
    type SerializeStruct = ser::Impossible<ValueAddr, Error>;
    type SerializeStructVariant = ser::Impossible<ValueAddr, Error>;

    fn serialize_bool(self, _v: bool) -> Result<ValueAddr> {
        Err(Error::SetInvalid)
    }

    fn serialize_i8(self, _v: i8) -> Result<ValueAddr> {
        Err(Error::SetInvalid)
    }

    fn serialize_i16(self, _v: i16) -> Result<ValueAddr> {
        Err(Error::SetInvalid)
    }

    fn serialize_i32(self, _v: i32) -> Result<ValueAddr> {
        Err(Error::SetInvalid)
    }

    fn serialize_i64(self, _v: i64) -> Result<ValueAddr> {
        Err(Error::SetInvalid)
    }

    fn serialize_u8(self, _v: u8) -> Result<ValueAddr> {
        Err(Error::SetInvalid)
    }

    fn serialize_u16(self, _v: u16) -> Result<ValueAddr> {
        Err(Error::SetInvalid)
    }

    fn serialize_u32(self, _v: u32) -> Result<ValueAddr> {
        Err(Error::SetInvalid)
    }

    fn serialize_u64(self, _v: u64) -> Result<ValueAddr> {
        Err(Error::SetInvalid)
    }

    fn serialize_f32(self, _v: f32) -> Result<ValueAddr> {
        Err(Error::SetInvalid)
    }

    fn serialize_f64(self, _v: f64) -> Result<ValueAddr> {
        Err(Error::SetInvalid)
    }

    fn serialize_char(self, _v: char) -> Result<ValueAddr> {
        Err(Error::SetInvalid)
    }

    fn serialize_str(self, _v: &str) -> Result<ValueAddr> {
        Err(Error::SetInvalid)
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<ValueAddr> {
        Err(Error::SetInvalid)
    }

    fn serialize_none(self) -> Result<ValueAddr> {
        Err(Error::SetInvalid)
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<ValueAddr>
    where
        T: ser::Serialize,
    {
        Err(Error::SetInvalid)
    }

    fn serialize_unit(self) -> Result<ValueAddr> {
        Err(Error::SetInvalid)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<ValueAddr> {
        Err(Error::SetInvalid)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<ValueAddr> {
        Err(Error::SetInvalid)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<ValueAddr>
    where
        T: ser::Serialize,
    {
        Err(Error::SetInvalid)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<ValueAddr>
    where
        T: ser::Serialize,
    {
        Err(Error::SetInvalid)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        let serializer = SetSerializer::from_serializer(self.0)?;
        Ok(serializer)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::SetInvalid)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::SetInvalid)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::SetInvalid)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::SetInvalid)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(Error::SetInvalid)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::SetInvalid)
    }
}

struct NumberRefEmitter<'a, 'i: 'a>(&'a mut Serializer<'i>);

impl<'a, 'i> ser::Serializer for NumberRefEmitter<'a, 'i> {
    type Ok = ValueAddr;
    type Error = Error;

    type SerializeSeq = ser::Impossible<ValueAddr, Error>;
    type SerializeTuple = ser::Impossible<ValueAddr, Error>;
    type SerializeTupleStruct = ser::Impossible<ValueAddr, Error>;
    type SerializeTupleVariant = ser::Impossible<ValueAddr, Error>;
    type SerializeMap = ser::Impossible<ValueAddr, Error>;
    type SerializeStruct = ser::Impossible<ValueAddr, Error>;
    type SerializeStructVariant = ser::Impossible<ValueAddr, Error>;

    fn serialize_bool(self, _v: bool) -> Result<ValueAddr> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_i8(self, _v: i8) -> Result<ValueAddr> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_i16(self, _v: i16) -> Result<ValueAddr> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_i32(self, _v: i32) -> Result<ValueAddr> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_i64(self, _v: i64) -> Result<ValueAddr> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_u8(self, _v: u8) -> Result<ValueAddr> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_u16(self, _v: u16) -> Result<ValueAddr> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_u32(self, _v: u32) -> Result<ValueAddr> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_u64(self, _v: u64) -> Result<ValueAddr> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_f32(self, _v: f32) -> Result<ValueAddr> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_f64(self, _v: f64) -> Result<ValueAddr> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_char(self, _v: char) -> Result<ValueAddr> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_str(self, v: &str) -> Result<ValueAddr> {
        let data_addr = self.0.store(v)?;
        let n = opa_number_t::from_str(v, data_addr);
        self.0.store(&n)
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<ValueAddr> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_none(self) -> Result<ValueAddr> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<ValueAddr>
    where
        T: ser::Serialize,
    {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_unit(self) -> Result<ValueAddr> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<ValueAddr> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<ValueAddr> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<ValueAddr>
    where
        T: ser::Serialize,
    {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<ValueAddr>
    where
        T: ser::Serialize,
    {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(Error::NumberRefInvalid)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::NumberRefInvalid)
    }
}
