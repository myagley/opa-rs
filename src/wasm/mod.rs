#[cfg(target_arch = "x86_64")]
mod wasmtime;

#[cfg(not(target_arch = "x86_64"))]
mod wasmi;

#[cfg(target_arch = "x86_64")]
pub use self::wasmtime::{FunctionsImpl, Instance, Memory, Module};

#[cfg(not(target_arch = "x86_64"))]
pub use self::wasmi::{FunctionsImpl, Instance, Memory, Module};
