use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let target = env::var("TARGET").unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();

    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg(&target)
        // Build shim as standalone package to avoid workspace lock.
        .arg("--manifest-path")
        .arg(
            workspace_root
                .join("crates")
                .join("shim")
                .join("Cargo.toml"),
        )
        .arg("--target-dir")
        .arg(workspace_root.join("intermediate"))
        .status()
        .expect("failed to build shim");

    assert!(status.success());

    let shim_name = if target.contains("windows") {
        "shim.exe"
    } else {
        "shim"
    };
    let shim_source = workspace_root
        .join("intermediate")
        .join(&target)
        .join("release")
        .join(shim_name);
    let shim_dest = out_dir.join(shim_name);
    fs::copy(&shim_source, &shim_dest).expect("failed to copy shim binary");

    println!("cargo:rerun-if-changed=../shim/src/main.rs");
    println!("cargo:rerun-if-changed=../shim/Cargo.toml");
}
