use serde::de;

use crate::opa::Result;
use crate::wasm::Instance;

pub fn from_instance<T>(_instance: Instance) -> Result<T>
where
    T: de::DeserializeOwned,
{
    unimplemented!()
}

pub struct Deserializer {
    instance: Instance,
}
