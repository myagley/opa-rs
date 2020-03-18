use policy::Policy;
use wasmtime::{Module, Store};

fn main() -> Result<(), anyhow::Error> {
    let store = Store::default();
    let module = Module::from_file(&store, "/home/miyagley/Code/opa/policy.wasm")?;

    let input = "{\"message\":\"world\"}";
    let data = "{\"world\":\"world\"}";

    let mut policy = Policy::from_wasm(&module)?;
    policy.set_data(data)?;
    let result = policy.evaluate(input)?;
    println!("result: {}", result);
    Ok(())
}
