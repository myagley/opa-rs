use std::{fmt, io};

use serde::{de, ser};
use thiserror::Error;

#[cfg(target_arch = "x86_64")]
use wasmtime::Trap;

use crate::{opa_serde, Value};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Policy is not initialized properly. This is a bug.")]
    Initialization,
    #[cfg(target_arch = "x86_64")]
    #[error("An occurred from wasmtime.")]
    Wasmtime(#[source] anyhow::Error),
    #[cfg(not(target_arch = "x86_64"))]
    #[error("An occurred from wasmi.")]
    Wasmi(#[source] wasmi::Error),
    #[error("Expected exported function {0}")]
    MissingExport(&'static str),
    #[cfg(target_arch = "x86_64")]
    #[error("A wasm function call trapped.")]
    Trap(
        #[source]
        #[from]
        Trap,
    ),
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
    #[error("Invalid function return. Expected {0}")]
    InvalidResult(&'static str),
    #[error("Failed to serialize value to instance.")]
    InstanceSerde(#[source] opa_serde::Error),
    #[error("Invalid buffer length when casting to struct. Expected {0}, got {1}.")]
    NotEnoughData(usize, usize),
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

impl From<opa_serde::Error> for Error {
    fn from(error: opa_serde::Error) -> Error {
        Error::InstanceSerde(error)
    }
}
