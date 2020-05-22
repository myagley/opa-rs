use std::fmt;

use rego::{CompiledQuery, ValueRef};
use serde::de::DeserializeOwned;

#[derive(Debug)]
pub enum Error {
    Compile(String),
    Runtime(rego::Error<'static>),
    Serialize(rego::Error<'static>),
    Deserialize(rego::Error<'static>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Compile(s) => write!(f, "Policy failed to compile: {}", s),
            Self::Runtime(_) => write!(f, "An error occurred while evaluating the policy."),
            Self::Serialize(_) => write!(f, "An error occurred while serializing the input."),
            Self::Deserialize(_) => write!(f, "An error occurred while deserializing the result."),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Runtime(e) => Some(e),
            Self::Serialize(e) => Some(e),
            Self::Deserialize(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Policy {
    query: CompiledQuery,
}

impl Policy {
    pub fn from_query(query: &str, modules: &[&str]) -> Result<Self, Error> {
        let query = rego::compile(query, modules).map_err(|e| Error::Compile(e.to_string()))?;
        let policy = Self { query };
        Ok(policy)
    }

    pub fn evaluate<T: ValueRef, V: DeserializeOwned>(&mut self, input: T) -> Result<V, Error> {
        let result = self.query.eval(&input).map_err(Error::Runtime)?;
        let result = rego::from_value(result).map_err(Error::Deserialize)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let module = r###"
        package test

        default allow = true
        "###;
        let query = "data.test.allow";
        let mut policy = Policy::from_query(query, &[module]).unwrap();
        let result = policy.evaluate(()).unwrap();
        assert_eq!(true, result);
    }
}
