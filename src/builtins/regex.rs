use regex::Regex;

use crate::{Error, Value};

// TODO - memoize the compilation of the regex
pub fn re_match(pattern: Value, value: Value) -> Result<Value, Error> {
    let pattern = format!("^{}$", pattern.try_into_string()?);
    let regex = Regex::new(&pattern).map_err(Error::InvalidRegex)?;
    let value = value.try_into_string()?;
    let b = regex.is_match(&value);
    Ok(b.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_re_match() {
        let result = re_match("[a-z]*".into(), "hello".into())
            .unwrap()
            .as_bool()
            .unwrap();
        assert_eq!(true, result);

        let result = re_match("[a-z]*".into(), "Hello".into())
            .unwrap()
            .as_bool()
            .unwrap();
        assert_eq!(false, result);
    }
}
