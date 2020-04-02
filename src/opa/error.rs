use std::fmt;

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
