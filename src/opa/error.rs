use std::{convert, fmt, num, str};

use serde::{de, ser};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Error, PartialEq)]
pub enum Error {
    #[error("{0}")]
    Message(String),
    #[error("Failed to alloc memory.")]
    Alloc,
    #[error("Failed to set memory.")]
    MemSet,
    #[error("Expected sequence length. Serializer does not support serializing sequences without lengths.")]
    ExpectedSeqLen,
    #[error("Invalid serialized length. Expected len {0}, serialized {1}")]
    InvalidSeqLen(usize, usize),
    #[error("Invalid buffer length when casting to struct. Expected {0}, got {1}.")]
    NotEnoughData(usize, usize),
    #[error("Unknown type: {0}")]
    UnknownType(u8),
    #[error("Expected boolean value. Found type {0}")]
    ExpectedBoolean(u8),
    #[error("Expected number value. Found type {0}")]
    ExpectedNumber(u8),
    #[error("Expected integer value. Found repr {0}")]
    ExpectedInteger(u8),
    #[error("Expected float value. Found repr {0}")]
    ExpectedFloat(u8),
    #[error("Integer conversion failed.")]
    IntegerConversion(#[source] num::TryFromIntError),
    #[error("Expected string value. Found type {0}")]
    ExpectedString(u8),
    #[error("Invalid utf8 string.")]
    InvalidUtf8(#[source] str::Utf8Error),
    #[error("Invalid char. Expected a string of length one.")]
    InvalidChar,
    #[error("Expected null value. Found type {0}")]
    ExpectedNull(u8),
    #[error("Expected array value. Found type {0}")]
    ExpectedArray(u8),
    #[error("Expected object value. Found type {0}")]
    ExpectedObject(u8),
    #[error("Expected next address when parsing object element value")]
    ExpectedNextAddr,
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl From<num::TryFromIntError> for Error {
    fn from(error: num::TryFromIntError) -> Self {
        Error::IntegerConversion(error)
    }
}

impl From<convert::Infallible> for Error {
    fn from(_error: convert::Infallible) -> Error {
        unreachable!()
    }
}
