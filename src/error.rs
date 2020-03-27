use std::str::Utf8Error;
use std::{fmt, io};

use serde::{de, ser};
use thiserror::Error;
use wasmtime::Trap;

use crate::{Value, ValueAddr};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Policy is not initialized properly. This is a bug.")]
    Initialization,
    #[error("An occurred from wasmtime")]
    Wasm(#[source] anyhow::Error),
    #[error("Expected exported function {0}")]
    MissingExport(&'static str),
    #[error("A wasm function call trapped.")]
    Trap(
        #[source]
        #[from]
        Trap,
    ),
    #[error("Failed to parse json at addr \"{0}\".")]
    JsonParse(ValueAddr),
    #[error("Failed to create CStr.")]
    CStr(#[source] Utf8Error),
    #[error("Failed to open a directory.")]
    DirOpen(#[source] io::Error),
    #[error("Failed to open a file.")]
    FileOpen(#[source] io::Error),
    #[error("Failed to read file.")]
    FileRead(#[source] io::Error),
    #[error("Failed to call opa compiler.")]
    OpaCommand(#[source] io::Error),
    #[error("Failed to compile rego file: {0}")]
    OpaCompiler(String),
    #[error("Failed to deserialize: {0}")]
    DeserializeValue(String),
    #[error("Failed to serialize: {0}")]
    SerializeValue(String),
    #[error("Failed to deserialize JSON.")]
    DeserializeJson(#[source] serde_json::Error),
    #[error("Failed to serialize JSON.")]
    SerializeJson(#[source] serde_json::Error),
    #[error("Invalid type in builtin function: expected {0}, got {1:?}")]
    InvalidType(&'static str, Value),
    #[error("Invalid type conversion in builtin function: expected {0}")]
    InvalidConversion(&'static str),
    #[error("Unknown builtin required: {0}")]
    UnknownBuiltin(String),
    #[error("Unknown builtin id: {0}")]
    UnknownBuiltinId(i32),
    #[error("Unknown timezone: {0}")]
    UnknownTimezone(String),
    #[error("Failed to parse datetime.")]
    ParseDatetime(#[source] chrono::ParseError),
    #[error("Invalid ip network.")]
    InvalidIpNetwork(#[source] ipnetwork::IpNetworkError),
    #[error("Invalid regex.")]
    InvalidRegex(#[source] regex::Error),
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Error {
        Error::DeserializeValue(msg.to_string())
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Error {
        Error::SerializeValue(msg.to_string())
    }
}
