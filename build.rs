use std::path::PathBuf;

fn main() {
    let deps = system_deps::Config::new().probe().unwrap();

    let bindings = bindgen::Builder::default()
        .header("src/wrapper.h")
        .clang_args(
            deps.all_include_paths()
                .iter()
                .map(|path| format!("-I{}", path.to_string_lossy())),
        )
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
