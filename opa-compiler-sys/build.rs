use std::env;
use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = root.join("opa.go");
    let mut go = gobuild::Build::new();
    go.file(&file);

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if target_arch == "arm" {
        go.env("CC", "arm-linux-gnueabihf-gcc");
        go.env("GOOS", "linux");
        go.env("GOARCH", "arm");
    }
    go.env("CGO_ENABLED", "1");
    go.compile("opa");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let header = out_path.join("libopa.h");
    let bindings = bindgen::Builder::default()
        .header(header.display().to_string())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .whitelist_function("Build")
        .whitelist_function("Free")
        .clang_arg("-I/usr/arm-linux-gnueabihf/include")
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
