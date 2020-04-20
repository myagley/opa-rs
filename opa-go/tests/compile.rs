use std::fs;
use std::path::PathBuf;

#[test]
fn test_opa_compiler_compile() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let bytes = opa_go::wasm::compile("data.tests.allow", &root.join("tests/empty.rego")).unwrap();
    let expected = fs::read(&root.join("tests/empty.wasm")).unwrap();
    assert_eq!(expected, bytes);
}
