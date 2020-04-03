#![allow(dead_code)]

use std::convert::TryFrom;
use std::os::raw::*;
use std::str;

use serde::de::{self, IntoDeserializer, Visitor};

use crate::opa::{Error, Result};
use crate::wasm::Instance;
use crate::ValueAddr;

use super::*;

pub struct Deserializer<'de> {
    instance: &'de Instance,
    addr: ValueAddr,
}

impl<'de> Deserializer<'de> {
    pub fn from_instance(addr: ValueAddr, instance: &'de Instance) -> Self {
        Self { instance, addr }
    }
}

pub fn from_instance<T>(addr: ValueAddr, instance: &Instance) -> Result<T>
where
    T: de::DeserializeOwned,
{
    let mut deserializer = Deserializer::from_instance(addr, instance);
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

impl<'de> Deserializer<'de> {
    fn peek_type(&self) -> Result<c_uchar> {
        self.instance
            .memory()
            .as_type::<opa_value>(self.addr)
            .map(|r| r.ty)
    }

    fn parse_bool(&self) -> Result<bool> {
        let ty = self.peek_type()?;
        if ty != OPA_BOOLEAN {
            return Err(Error::ExpectedBoolean(ty as u8));
        }

        let b = self.instance.memory().as_type::<opa_boolean_t>(self.addr)?;
        if b.v == 0 {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    fn parse_int<T: TryFrom<i64>>(&self) -> Result<T>
    where
        T: TryFrom<i64>,
        <T as TryFrom<i64>>::Error: Into<Error>,
    {
        let ty = self.peek_type()?;
        if ty != OPA_NUMBER {
            return Err(Error::ExpectedNumber(ty as u8));
        }

        let n = self.instance.memory().as_type::<opa_number_t>(self.addr)?;
        if n.repr != OPA_NUMBER_REPR_INT {
            return Err(Error::ExpectedInteger(n.repr as u8));
        }

        let i = unsafe { T::try_from(n.v.i).map_err(|e| e.into())? };
        Ok(i)
    }

    fn parse_float(&self) -> Result<f64> {
        let ty = self.peek_type()?;
        if ty != OPA_NUMBER {
            return Err(Error::ExpectedNumber(ty as u8));
        }

        let n = self.instance.memory().as_type::<opa_number_t>(self.addr)?;
        if n.repr != OPA_NUMBER_REPR_FLOAT {
            return Err(Error::ExpectedFloat(n.repr as u8));
        }

        let f = unsafe { n.v.f };
        Ok(f)
    }

    fn parse_string(&self) -> Result<&str> {
        let ty = self.peek_type()?;
        if ty != OPA_STRING {
            return Err(Error::ExpectedString(ty as u8));
        }
        let s = self.instance.memory().as_type::<opa_string_t>(self.addr)?;
        let s = unsafe {
            let start = s.v as usize;
            let end = start + s.len as usize;
            let slice = &self.instance.memory().data_unchecked()[start..end];
            str::from_utf8(slice).map_err(Error::InvalidUtf8)?
        };
        Ok(s)
    }
}

impl<'a, 'de> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.peek_type()? {
            OPA_NULL => self.deserialize_unit(visitor),
            OPA_BOOLEAN => self.deserialize_bool(visitor),
            OPA_NUMBER => self.deserialize_i64(visitor),
            OPA_STRING => self.deserialize_str(visitor),
            OPA_ARRAY => self.deserialize_seq(visitor),
            OPA_OBJECT => self.deserialize_map(visitor),
            OPA_SET => self.deserialize_seq(visitor),
            t => Err(Error::UnknownType(t as u8)),
        }
    }

    // Uses the `parse_bool` parsing function defined above to read the JSON
    // identifier `true` or `false` from the input.
    //
    // Parsing refers to looking at the input and deciding that it contains the
    // JSON value `true` or `false`.
    //
    // Deserialization refers to mapping that JSON value into Serde's data
    // model by invoking one of the `Visitor` methods. In the case of JSON and
    // bool that mapping is straightforward so the distinction may seem silly,
    // but in other cases Deserializers sometimes perform non-obvious mappings.
    // For example the TOML format has a Datetime type and Serde's data model
    // does not. In the `toml` crate, a Datetime in the input is deserialized by
    // mapping it to a Serde data model "struct" type with a special name and a
    // single field containing the Datetime represented as a string.
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    // The `parse_signed` function is generic over the integer type `T` so here
    // it is invoked with `T=i8`. The next 8 methods are similar.
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.parse_int()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.parse_int()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse_int()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_int()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse_int()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse_int()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse_int()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse_int()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.parse_float()? as f32)
    }

    // Float parsing is stupidly hard.
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_float()?)
    }

    // The `Serializer` implementation on the previous page serialized chars as
    // single-character strings so handle that representation here.
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let s = self.parse_string()?;
        s.chars()
            .next()
            .map_or(Err(Error::InvalidChar), |c| visitor.visit_char(c))
    }

    // Refer to the "Understanding deserializer lifetimes" page for information
    // about the three deserialization flavors of strings in Serde.
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(self.parse_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    // The `Serializer` implementation on the previous page serialized byte
    // arrays as JSON arrays of bytes. Handle that representation here.
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    // An absent optional is represented as the JSON `null` and a present
    // optional is represented as just the contained value.
    //
    // As commented in `Serializer` implementation, this is a lossy
    // representation. For example the values `Some(())` and `None` both
    // serialize as just `null`. Unfortunately this is typically what people
    // expect when working with JSON. Other formats are encouraged to behave
    // more intelligently if possible.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.peek_type()? == OPA_NULL {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    // In Serde, unit means an anonymous value containing no data.
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let ty = self.peek_type()?;
        if ty == OPA_NULL {
            visitor.visit_unit()
        } else {
            Err(Error::ExpectedNull(ty as u8))
        }
    }

    // Unit struct means a named value containing no data.
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain. That means not
    // parsing anything other than the contained value.
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    // Deserialization of compound types like sequences and maps happens by
    // passing the visitor an "Access" object that gives it the ability to
    // iterate through the data contained in the sequence.
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let ty = self.peek_type()?;
        if ty != OPA_ARRAY {
            return Err(Error::ExpectedArray(ty as u8));
        }

        let array = self.instance.memory().as_type::<opa_array_t>(self.addr)?;
        let access = ArrayAccess::new(self, &array);
        visitor.visit_seq(access)
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently.
    //
    // As indicated by the length parameter, the `Deserialize` implementation
    // for a tuple in the Serde data model is required to know the length of the
    // tuple before even looking at the input data.
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    // Tuple structs look just like sequences in JSON.
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    // Much like `deserialize_seq` but calls the visitors `visit_map` method
    // with a `MapAccess` implementation, rather than the visitor's `visit_seq`
    // method with a `SeqAccess` implementation.
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let ty = self.peek_type()?;
        if ty != OPA_OBJECT {
            return Err(Error::ExpectedObject(ty as u8));
        }

        let object = self.instance.memory().as_type::<opa_object_t>(self.addr)?;
        let access = ObjectAccess::new(self, &object);
        visitor.visit_map(access)
    }

    // Structs look just like maps in JSON.
    //
    // Notice the `fields` parameter - a "struct" in the Serde data model means
    // that the `Deserialize` implementation is required to know what the fields
    // are before even looking at the input data. Any key-value pairing in which
    // the fields cannot be known ahead of time is probably a map.
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.peek_type()? {
            OPA_STRING => visitor.visit_enum(self.parse_string()?.into_deserializer()),
            OPA_OBJECT => visitor.visit_enum(EnumAccess::new(self)),
            ty => Err(Error::ExpectedEnum(ty as u8)),
        }
    }

    // An identifier in Serde is the type that identifies a field of a struct or
    // the variant of an enum. In JSON, struct fields and enum variants are
    // represented as strings. In other formats they may be represented as
    // numeric indices.
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    // Like `deserialize_any` but indicates to the `Deserializer` that it makes
    // no difference which `Visitor` method is called because the data is
    // ignored.
    //
    // Some deserializers are able to implement this more efficiently than
    // `deserialize_any`, for example by rapidly skipping over matched
    // delimiters without paying close attention to the data in between.
    //
    // Some formats are not able to implement this at all. Formats that can
    // implement `deserialize_any` and `deserialize_ignored_any` are known as
    // self-describing.
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct ArrayAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    n: usize,
    len: usize,
    elems: ValueAddr,
}

impl<'a, 'de> ArrayAccess<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, array: &opa_array_t) -> Self {
        ArrayAccess {
            de,
            n: 0,
            len: array.len as usize,
            elems: ValueAddr(array.elems as i32),
        }
    }
}

impl<'de, 'a> de::SeqAccess<'de> for ArrayAccess<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.n == self.len {
            return Ok(None);
        }
        let addr = self.elems + self.n * mem::size_of::<opa_array_elem_t>();
        let elem = self
            .de
            .instance
            .memory()
            .as_type::<opa_array_elem_t>(addr)?;

        self.n = self.n + 1;
        self.de.addr = ValueAddr(elem.v as i32);
        seed.deserialize(&mut *self.de).map(Some)
    }
}

struct ObjectAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    next: Option<ValueAddr>,
}

impl<'a, 'de> ObjectAccess<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, map: &opa_object_t) -> Self {
        let next = if map.head == 0 {
            None
        } else {
            Some(ValueAddr(map.head as i32))
        };
        ObjectAccess { de, next }
    }
}

impl<'de, 'a> de::MapAccess<'de> for ObjectAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if let Some(next_addr) = self.next {
            let elem = self
                .de
                .instance
                .memory()
                .as_type::<opa_object_elem_t>(next_addr)?;
            self.de.addr = ValueAddr(elem.k as i32);
            seed.deserialize(&mut *self.de).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let Some(next_addr) = self.next {
            let elem = self
                .de
                .instance
                .memory()
                .as_type::<opa_object_elem_t>(next_addr)?;
            self.next = if elem.next != 0 {
                Some(elem.next.into())
            } else {
                None
            };

            self.de.addr = ValueAddr(elem.v as i32);
            seed.deserialize(&mut *self.de)
        } else {
            Err(Error::ExpectedNextAddr)
        }
    }
}

struct EnumAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> EnumAccess<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        EnumAccess { de }
    }
}

impl<'de, 'a> de::EnumAccess<'de> for EnumAccess<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        // we are looking at a map
        // read the first key
        let ty = self.de.peek_type()?;
        if ty != OPA_OBJECT {
            return Err(Error::ExpectedObject(ty as u8));
        }

        let object = self
            .de
            .instance
            .memory()
            .as_type::<opa_object_t>(self.de.addr)?;
        let elem = self
            .de
            .instance
            .memory()
            .as_type::<opa_object_elem_t>(ValueAddr(object.head as i32))?;
        self.de.addr = ValueAddr(elem.k as i32);
        let val = seed.deserialize(&mut *self.de)?;
        self.de.addr = ValueAddr(elem.v as i32);
        Ok((val, self))
    }
}

impl<'de, 'a> de::VariantAccess<'de> for EnumAccess<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        // If the `Visitor` expected this variant to be a unit variant, the input
        // should have been the plain string case handled in `deserialize_enum`.
        Err(Error::ExpectedString(0))
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self.de, visitor)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;

    use serde::{Deserialize, Serialize};

    use crate::opa::to_instance;
    use crate::wasm::{Instance, Memory, Module};

    use super::*;

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

    thread_local! {
        static EMPTY_MODULE: Module = {
            let bytes = fs::read("tests/empty.wasm").unwrap();
            let module = Module::from_bytes(bytes).unwrap();
            module
        };
    }

    macro_rules! type_roundtrip {
        ($name:ident, $ty:ty, $input:expr) => {
            #[test]
            fn $name() {
                EMPTY_MODULE.with(|module| {
                    let memory = Memory::from_module(module);
                    let instance = Instance::new(module, memory).unwrap();
                    let addr = to_instance(instance.clone(), &$input).unwrap();
                    let loaded = from_instance::<$ty>(addr, &instance).unwrap();
                    assert_eq!($input, loaded);
                })
            }
        };
    }

    type_roundtrip!(test_deserialize_bool, bool, true);
    type_roundtrip!(test_deserialize_i8, i8, 42_i8);
    type_roundtrip!(test_deserialize_i16, i16, 42_i16);
    type_roundtrip!(test_deserialize_i32, i32, 42_i32);
    type_roundtrip!(test_deserialize_i64, i64, 42_i64);
    type_roundtrip!(test_deserialize_u8, u8, 42_u8);
    type_roundtrip!(test_deserialize_u16, u16, 42_u16);
    type_roundtrip!(test_deserialize_u32, u32, 42_u32);
    type_roundtrip!(test_deserialize_u64, u64, 42_u64);
    type_roundtrip!(test_deserialize_f32, f32, 1.234_f32);
    type_roundtrip!(test_deserialize_f64, f64, 1.234_f64);

    type_roundtrip!(test_deserialize_string, String, "hello there".to_string());
    type_roundtrip!(test_deserialize_char, char, 'a');
    type_roundtrip!(test_deserialize_none, Option<i64>, Option::<i64>::None);
    type_roundtrip!(test_deserialize_some, Option<i64>, Some(56));
    type_roundtrip!(test_deserialize_unit_struct, UnitStruct, UnitStruct);
    type_roundtrip!(
        test_deserialize_newtype_struct,
        NewTypeStruct,
        NewTypeStruct(56)
    );
    type_roundtrip!(test_deserialize_unit_variant, TestEnum, TestEnum::Unit);
    type_roundtrip!(
        test_deserialize_newtype_variant,
        TestEnum,
        TestEnum::NewType(64)
    );
    type_roundtrip!(
        test_deserialize_tuple_variant,
        TestEnum,
        TestEnum::Tuple(64, "Hello".to_string())
    );
    type_roundtrip!(
        test_deserialize_struct_variant,
        TestEnum,
        TestEnum::Struct {
            age: 64,
            msg: "Hello".to_string()
        }
    );

    type_roundtrip!(
        test_deserialize_vec,
        Vec<String>,
        vec!["hello".to_string(), "there".to_string()]
    );
    type_roundtrip!(
        test_deserialize_tuple,
        (i64, String),
        (42, "hello".to_string())
    );
    type_roundtrip!(
        test_deserialize_tuple_struct,
        TupleStruct,
        TupleStruct(42, "hello".to_string())
    );

    #[test]
    fn test_deserialize_map() {
        EMPTY_MODULE.with(|module| {
            let memory = Memory::from_module(module);
            let instance = Instance::new(module, memory).unwrap();
            let mut input = HashMap::new();
            input.insert("key1".to_string(), 3);
            input.insert("key2".to_string(), 2);
            let addr = to_instance(instance.clone(), &input).unwrap();
            let loaded = from_instance(addr, &instance).unwrap();
            assert_eq!(input, loaded);
        })
    }

    #[test]
    fn test_deserialize_empty_map() {
        EMPTY_MODULE.with(|module| {
            let memory = Memory::from_module(module);
            let instance = Instance::new(module, memory).unwrap();
            let input: HashMap<String, i64> = HashMap::new();
            let addr = to_instance(instance.clone(), &input).unwrap();
            let loaded = from_instance(addr, &instance).unwrap();
            assert_eq!(input, loaded);
        })
    }

    #[test]
    fn test_deserialize_struct() {
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
            let loaded = from_instance(addr, &instance).unwrap();
            assert_eq!(person, loaded);
        })
    }

    #[test]
    fn test_deserialize_unit() {
        EMPTY_MODULE.with(|module| {
            let memory = Memory::from_module(module);
            let instance = Instance::new(module, memory).unwrap();
            let input = ();
            let addr = to_instance(instance.clone(), &input).unwrap();
            let loaded = from_instance(addr, &instance).unwrap();
            assert_eq!(input, loaded);
        })
    }
}
