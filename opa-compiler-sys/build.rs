use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed={}", "opa.go");
    gobuild::Build::new().file("opa.go").compile("opa");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let header = out_path.join("libopa.h");
    let bindings = bindgen::Builder::default()
        .header(header.display().to_string())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .whitelist_function("Build")
        .whitelist_function("Free")
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
