use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

mod de;
mod from;
mod index;
pub(crate) mod number;
mod ser;

use crate::error::Error;

pub use self::index::Index;
pub use self::number::Number;

pub type Map<K, V> = BTreeMap<K, V>;
pub type Set<V> = BTreeSet<V>;

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(Map<String, Value>),
    Set(Set<Value>),
}

impl fmt::Debug for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Null => formatter.debug_tuple("Null").finish(),
            Value::Bool(v) => formatter.debug_tuple("Bool").field(&v).finish(),
            Value::Number(ref v) => fmt::Debug::fmt(v, formatter),
            Value::String(ref v) => formatter.debug_tuple("String").field(v).finish(),
            Value::Array(ref v) => formatter.debug_tuple("Array").field(v).finish(),
            Value::Object(ref v) => formatter.debug_tuple("Object").field(v).finish(),
            Value::Set(ref v) => formatter.debug_tuple("Set").field(v).finish(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Null => write!(f, "null"),
            Value::Bool(ref v) => fmt::Display::fmt(v, f),
            Value::Number(ref v) => fmt::Display::fmt(v, f),
            Value::String(ref v) => write!(f, "\"{}\"", v.escape_default()),
            Value::Array(ref v) => {
                write!(f, "[")?;
                let mut iter = v.iter();
                if let Some(first) = iter.next() {
                    fmt::Display::fmt(first, f)?;
                }
                while let Some(elem) = iter.next() {
                    write!(f, ",")?;
                    fmt::Display::fmt(elem, f)?;
                }
                write!(f, "]")
            }
            Value::Object(ref v) => {
                write!(f, "{{")?;
                let mut iter = v.iter();
                if let Some((k, v)) = iter.next() {
                    fmt::Display::fmt(k, f)?;
                    write!(f, ":")?;
                    fmt::Display::fmt(v, f)?;
                }
                while let Some((k, v)) = iter.next() {
                    write!(f, ",")?;
                    fmt::Display::fmt(k, f)?;
                    write!(f, ":")?;
                    fmt::Display::fmt(v, f)?;
                }
                write!(f, "}}")
            }
            Value::Set(ref v) => {
                write!(f, "{{")?;
                let mut iter = v.iter();
                if let Some(first) = iter.next() {
                    fmt::Display::fmt(first, f)?;
                }
                while let Some(elem) = iter.next() {
                    write!(f, ",")?;
                    fmt::Display::fmt(elem, f)?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl Default for Value {
    fn default() -> Value {
        Value::Null
    }
}

impl Value {
    pub fn get<I: Index>(&self, index: I) -> Option<&Value> {
        index.index_into(self)
    }

    pub fn get_mut<I: Index>(&mut self, index: I) -> Option<&mut Value> {
        index.index_into_mut(self)
    }

    pub fn try_into_set(self) -> Result<Set<Value>, Error> {
        match self {
            Value::Set(v) => Ok(v),
            v => Err(Error::InvalidType("set", v)),
        }
    }

    pub fn as_set(&self) -> Option<&Set<Value>> {
        match *self {
            Value::Set(ref set) => Some(set),
            _ => None,
        }
    }

    pub fn as_set_mut(&mut self) -> Option<&mut Set<Value>> {
        match *self {
            Value::Set(ref mut set) => Some(set),
            _ => None,
        }
    }

    pub fn is_set(&self) -> bool {
        self.as_set().is_some()
    }

    pub fn try_into_object(self) -> Result<Map<String, Value>, Error> {
        match self {
            Value::Object(map) => Ok(map),
            v => Err(Error::InvalidType("object", v)),
        }
    }

    pub fn as_object(&self) -> Option<&Map<String, Value>> {
        match *self {
            Value::Object(ref map) => Some(map),
            _ => None,
        }
    }

    pub fn as_object_mut(&mut self) -> Option<&mut Map<String, Value>> {
        match *self {
            Value::Object(ref mut map) => Some(map),
            _ => None,
        }
    }

    pub fn is_object(&self) -> bool {
        self.as_object().is_some()
    }

    pub fn try_into_array(self) -> Result<Vec<Value>, Error> {
        match self {
            Value::Array(array) => Ok(array),
            v => Err(Error::InvalidType("array", v)),
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match *self {
            Value::Array(ref array) => Some(array),
            _ => None,
        }
    }

    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        match *self {
            Value::Array(ref mut array) => Some(array),
            _ => None,
        }
    }

    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    pub fn try_into_string(self) -> Result<String, Error> {
        match self {
            Value::String(string) => Ok(string),
            v => Err(Error::InvalidType("string", v)),
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match *self {
            Value::String(ref string) => Some(string),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        self.as_str().is_some()
    }

    pub fn is_number(&self) -> bool {
        match *self {
            Value::Number(_) => true,
            _ => false,
        }
    }

    pub fn try_into_i64(self) -> Result<i64, Error> {
        match self {
            Value::Number(n) => n.try_into_i64(),
            v => Err(Error::InvalidType("i64", v)),
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Value::Number(ref n) => n.as_i64(),
            _ => None,
        }
    }

    pub fn is_i64(&self) -> bool {
        match *self {
            Value::Number(ref n) => n.is_i64(),
            _ => false,
        }
    }

    pub fn try_into_f64(self) -> Result<f64, Error> {
        match self {
            Value::Number(n) => n.try_into_f64(),
            v => Err(Error::InvalidType("f64", v)),
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Value::Number(ref n) => n.as_f64(),
            _ => None,
        }
    }

    pub fn is_f64(&self) -> bool {
        match *self {
            Value::Number(ref n) => n.is_f64(),
            _ => false,
        }
    }

    pub fn try_into_bool(self) -> Result<bool, Error> {
        match self {
            Value::Bool(b) => Ok(b),
            v => Err(Error::InvalidType("bool", v)),
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Value::Bool(b) => Some(b),
            _ => None,
        }
    }

    pub fn is_boolean(&self) -> bool {
        self.as_bool().is_some()
    }

    pub fn as_null(&self) -> Option<()> {
        match *self {
            Value::Null => Some(()),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        self.as_null().is_some()
    }
}
