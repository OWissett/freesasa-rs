extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    if let Some(freesasa_location) = option_env!("FREESASA_STATIC_LIB")
    {
        println!(
            "cargo:rustc-link-search=native={}",
            freesasa_location
        );
    } else {
        println!(
            "cargo:warning=FREESASA_STATIC_LIB not set, assuming /usr/local/lib. \
            If this is not correct, please set FREESASA_STATIC_LIB to the directory \
            containing libfreesasa.a"
        );
        println!("cargo:rustc-link-search=native=/usr/local/lib");
    }

    println!("cargo:rustc-link-lib=static=freesasa");

    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=build.rs");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .merge_extern_blocks(true)
        .rustfmt_bindings(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
