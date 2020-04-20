use std::collections::{HashMap, HashSet};
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tracing::Level;
use tracing_subscriber::{fmt, EnvFilter};

use opa_wasm::Policy;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct UnitStruct;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct NewtypeStruct(u8);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Struct {
    a: u8,
    b: u8,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TestEnum {
    Unit,
    NewType(i64),
    Tuple(i64, String),
    Struct { age: i64, msg: String },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct TestStruct {
    byte: i8,
    short: i16,
    int: i32,
    long: i64,

    ubyte: u8,
    ushort: u16,
    uint: u32,
    ulong: u64,

    float: f32,
    double: f64,

    string: String,

    unit: (),
    unit_struct: UnitStruct,
    newtype_struct: NewtypeStruct,
    struc: Struct,

    unit_variant: TestEnum,
    newtype_variant: TestEnum,
    tuple_variant: TestEnum,
    struct_variant: TestEnum,

    some: Option<String>,
    none: Option<String>,

    map: HashMap<u8, u8>,
    list: Vec<i16>,
    #[serde(with = "opa_wasm::set")]
    set: HashSet<String>,
}

#[test]
fn test_eval() {
    let subscriber = fmt::Subscriber::builder()
        .with_ansi(atty::is(atty::Stream::Stderr))
        .with_max_level(Level::TRACE)
        .with_writer(io::stderr)
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let mut rego = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    rego.push("tests/eval_struct_test.rego");

    let mut map = HashMap::new();
    map.insert(1, 2);
    map.insert(2, 3);

    let mut set = HashSet::new();
    set.insert("a".to_string());
    set.insert("b".to_string());

    let module = opa_go::wasm::compile("data.tests.eval_struct", &rego).unwrap();
    let mut policy = Policy::from_wasm(&module).unwrap();
    let input = TestStruct {
        byte: -1,
        short: -257,
        int: -65_600,
        long: -3_000_000_000,

        ubyte: 1,
        ushort: 257,
        uint: 65_600,
        ulong: 3_000_000_000,

        float: 1.0499999523162842,
        double: 2.34,

        string: "this is a string".to_string(),

        unit: (),
        unit_struct: UnitStruct,
        newtype_struct: NewtypeStruct(3),
        struc: Struct { a: 1, b: 2 },

        unit_variant: TestEnum::Unit,
        newtype_variant: TestEnum::NewType(64),
        tuple_variant: TestEnum::Tuple(42, "hello".to_string()),
        struct_variant: TestEnum::Struct {
            age: 72,
            msg: "goodbye".to_string(),
        },

        some: Some("there's something here".to_string()),
        none: None,

        map,
        list: vec![1, 2, 3],
        set,
    };
    let result = policy.evaluate(&input).unwrap();
    assert_eq!(1, result.as_set().unwrap().len());
}
