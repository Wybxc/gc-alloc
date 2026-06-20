use std::path::PathBuf;

fn main() {
    let is_docs_rs = std::env::var("DOCS_RS").is_ok();

    let dst = if !is_docs_rs {
        let dst = cmake::Config::new("vendored/bdwgc")
            .define("BUILD_SHARED_LIBS", "OFF")
            .define("build_cord", "OFF")
            .define("build_tests", "OFF")
            .define("enable_docs", "OFF")
            .define("enable_gcj_support", "OFF")
            .define("enable_java_finalization", "OFF")
            .define("enable_throw_bad_alloc_library", "OFF")
            .define("enable_disclaim", "OFF")
            .define("enable_handle_fork", "OFF")
            .profile("Release")
            .build();

        println!("cargo:rustc-link-search=native={}/lib", dst.display());
        println!("cargo:rustc-link-lib=static=gc");

        Some(dst)
    } else {
        None
    };

    let mut builder = bindgen::Builder::default()
        .header("src/wrapper.h")
        .clang_arg("-Ivendored/bdwgc/include");

    if let Some(ref dst) = dst {
        builder = builder.clang_arg(format!("-I{}/include", dst.display()));
    }

    let bindings = builder
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
