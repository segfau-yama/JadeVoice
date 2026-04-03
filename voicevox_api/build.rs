use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set"));
    let workspace_root = manifest_dir
        .parent()
        .expect("voicevox_api must be placed under workspace root");

    let c_api_lib_dir = workspace_root.join("voicevox_core").join("c_api").join("lib");
    let onnxruntime_lib_dir = workspace_root.join("voicevox_core").join("onnxruntime").join("lib");

    println!("cargo:rerun-if-changed={}", c_api_lib_dir.display());
    println!("cargo:rerun-if-changed={}", onnxruntime_lib_dir.display());

    println!("cargo:rustc-link-search=native={}", c_api_lib_dir.display());
    println!("cargo:rustc-link-search=native={}", onnxruntime_lib_dir.display());
    println!("cargo:rustc-link-lib=dylib=voicevox_core");
}
