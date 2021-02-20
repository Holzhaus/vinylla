extern crate cbindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    if check_env_var("CARGO_GENERATE_C_HEADER") {
        generate_c_header();
    }
}

/// Returns true if the environment variable is set to "1", "true", "on" or "yes".
fn check_env_var(variable_name: &str) -> bool {
    println!("cargo:rerun-if-env-changed={}", variable_name);
    if let Ok(value) = env::var(variable_name) {
        matches!(value.to_lowercase().as_str(), "1" | "true" | "on" | "yes")
    } else {
        false
    }
}

/// Generate a C header file in the target directory.
fn generate_c_header() {
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let target_dir = target_dir();
    let package_name = env::var("CARGO_PKG_NAME").unwrap();

    let config = cbindgen::Config::from_file("cbindgen.toml")
        .expect("Unable to find cbindgen.toml configuration file");

    cbindgen::generate_with_config(&crate_dir, config)
        .expect("Unable to generate bindings")
        .write_to_file(target_dir.join(format!("{}.h", package_name)));
}

/// Find the location of the `target/` directory. Note that this may be
/// overridden by `cmake`, so we also need to check the `CARGO_TARGET_DIR`
/// variable.
fn target_dir() -> PathBuf {
    if let Ok(target) = env::var("CARGO_TARGET_DIR") {
        PathBuf::from(target)
    } else {
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("target")
    }
}
