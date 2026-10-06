use std::{env, process::Command};

fn main() {
    println!("cargo:rerun-if-env-changed=RUSTC");
    println!("cargo:rustc-check-cfg=cfg(canonical_verified_std_sort)");
    let compiler = env::var_os("RUSTC").expect("Cargo supplies its configured Rust compiler");
    let version = Command::new(compiler)
        .args(["--version", "--verbose"])
        .output()
        .expect("configured Rust compiler reports its version");
    assert!(
        version.status.success(),
        "configured Rust compiler version failed"
    );
    let version = String::from_utf8(version.stdout).expect("Rust version is UTF-8");
    let verified_release = version.lines().any(|line| line == "release: 1.94.0");
    let verified_commit = version
        .lines()
        .any(|line| line == "commit-hash: 4a4ef493e3a1488c6e321570238084b38948f6db");
    if verified_release && verified_commit {
        println!("cargo:rustc-cfg=canonical_verified_std_sort");
    }
}
