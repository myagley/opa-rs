use std::env;
use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = root.join("opa.go");
    gobuild::Build::new().file(&file).compile("opa");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let header = out_path.join("libopa.h");
    let bindings = bindgen::Builder::default()
        .header(header.display().to_string())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .whitelist_function("Free")
        .whitelist_function("RegoNew")
        .whitelist_function("RegoDrop")
        .whitelist_function("RegoEval")
        .whitelist_function("RegoEvalBool")
        .whitelist_function("WasmBuild")
        .clang_arg("-I/usr/arm-linux-gnueabihf/include")
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
