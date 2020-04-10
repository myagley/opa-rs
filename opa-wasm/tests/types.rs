use opa_wasm::{Policy, Value};

#[test]
fn test_types() {
    let module = opa_compiler::compile("data.tests.types", "tests/types.rego").unwrap();
    let mut policy = Policy::from_wasm(&module).unwrap();
    let result = policy.evaluate(&Value::Null).unwrap();
    assert_eq!(1, result.as_set().unwrap().len());
}
