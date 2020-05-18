use rego::CompiledQuery;
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Debug)]
pub enum Error {}

#[derive(Debug)]
pub struct Policy {
    query: CompiledQuery,
}

impl Policy {
    pub fn from_query(query: &str, modules: &[&str]) -> Result<Self, Error> {
        let query = rego::compile(query, modules).unwrap();
        let policy = Self { query };
        Ok(policy)
    }

    pub fn evaluate<T: Serialize, V: DeserializeOwned>(&mut self, input: T) -> Result<V, Error> {
        let input = rego::to_value(input).unwrap();
        let result = self.query.eval(input).unwrap();
        let result = rego::from_value(result).unwrap();
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
