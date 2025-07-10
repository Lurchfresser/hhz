use std::env;

fn main() {
    // Read the engine name from an environment variable set by the build script.
    // If it's not set (e.g., during a normal `cargo run`), use a default value.
    let engine_name = env::var("HHZ_ENGINE_NAME").unwrap_or_else(|_| "hhz_dev_build".to_string());

    // Pass the engine name to the Rust compiler.
    println!("cargo:rustc-env=HHZ_ENGINE_NAME={}", engine_name);

    // Tell Cargo to rerun this script if the environment variable changes.
    println!("cargo:rerun-if-env-changed=HHZ_ENGINE_NAME");
}