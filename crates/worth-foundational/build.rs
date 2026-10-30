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
    let compiler_field = |name: &str| {
        version
            .lines()
            .find_map(|line| line.strip_prefix(name))
            .expect("configured Rust compiler supplies verbose identity")
    };
    println!(
        "cargo:rustc-env=WORTH_CANONICAL_SORT_COMPILER_RELEASE={}",
        compiler_field("release: ")
    );
    println!(
        "cargo:rustc-env=WORTH_CANONICAL_SORT_COMPILER_COMMIT={}",
        compiler_field("commit-hash: ")
    );
    // Admission's sort bound is tied to the reviewed standard-library
    // implementations. Qualification never changes ordinary canonical ordering
    // or its comparison count, which participates in existing digest material.
    let verified = [
        (
            "release: 1.94.0",
            "commit-hash: 4a4ef493e3a1488c6e321570238084b38948f6db",
        ),
        (
            "release: 1.98.0",
            "commit-hash: 88d9e12ae178fab0fb5cc050a94da85685d449ea",
        ),
        (
            "release: 1.99.1",
            "commit-hash: 58cf8f9c35b355199727627d07ae34a4e2846d67",
        ),
    ]
    .into_iter()
    .any(|(release, commit)| {
        version.lines().any(|line| line == release) && version.lines().any(|line| line == commit)
    });
    if verified {
        println!("cargo:rustc-cfg=canonical_verified_std_sort");
    }
}
