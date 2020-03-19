use policy::Policy;
use wasmtime::{Module, Store};

fn main() -> Result<(), anyhow::Error> {
    let store = Store::default();
    let module = Module::from_file(&store, "/home/miyagley/Code/opa/policy.wasm")?;

    // let input = r#"{"servers":[{"id":"app","protocols":["https","ssh"],"ports":["p1","p2","p3"]},{"id":"db","protocols":["mysql"],"ports":["p3"]},{"id":"cache","protocols":["memcache"],"ports":["p3"]},{"id":"ci","protocols":["http"],"ports":["p1","p2"]},{"id":"busybox","protocols":["telnet"],"ports":["p1"]}],"networks":[{"id":"net1","public":false},{"id":"net2","public":false},{"id":"net3","public":true},{"id":"net4","public":true}],"ports":[{"id":"p1","network":"n1"},{"id":"p2","network":"n3"},{"id":"p3","network":"n2"}]}"#;
    let input = r#"{"servers":[{"id":"app","protocols":["https","ssh"],"ports":["p1","p2","p3"]},{"id":"db","protocols":["mysql"],"ports":["p3"]},{"id":"cache","protocols":["memcache"],"ports":["p3"]},{"id":"ci","protocols":["http"],"ports":["p1","p2"]},{"id":"busybox","protocols":["telnet"],"ports":["p1"]}],"networks":[{"id":"net1","public":false},{"id":"net2","public":false},{"id":"net3","public":true},{"id":"net4","public":true}],"ports":[{"id":"p1","network":"net1"},{"id":"p2","network":"net3"},{"id":"p3","network":"net2"}]}"#;
    // let input = "{}";

    let mut policy = Policy::from_wasm(&module)?;
    let result = policy.evaluate(&input)?;
    println!("result: {}", result);
    Ok(())
}
