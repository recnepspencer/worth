use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
#[path = "build_support/storage_profile.rs"]
mod storage_profile;

fn main() {
    // Storage envelopes are qualified against the actual standard library,
    // not merely the package's minimum supported Rust version.
    println!("cargo:rerun-if-env-changed=RUSTC");
    let compiler =
        std::process::Command::new(std::env::var_os("RUSTC").expect("Cargo supplies the compiler"))
            .arg("--version")
            .output()
            .expect("the build compiler must report its version");
    assert!(compiler.status.success(), "compiler version query failed");
    let compiler = String::from_utf8(compiler.stdout).expect("compiler version is UTF-8");
    let compiler = compiler.trim();
    println!("cargo:rustc-env=WORTH_STORE_MEDIA_COMPILER={compiler}");
    let lock_path = Path::new("../../Cargo.lock");
    println!("cargo:rerun-if-changed={}", lock_path.display());
    println!("cargo:rerun-if-changed=build_support");
    let lock = storage_profile::read_lock(lock_path);
    let storage_inputs: Vec<_> = storage_profile::PROFILE_INPUTS
        .iter()
        .map(|name| {
            println!("cargo:rerun-if-env-changed={name}");
            let value = if *name == "CARGO_CFG_RACY_ASSERTS" {
                if std::env::var_os(name).is_some() {
                    "present".into()
                } else {
                    String::new()
                }
            } else {
                match std::env::var(name) {
                    Ok(value) => value,
                    Err(std::env::VarError::NotPresent) => String::new(),
                    Err(std::env::VarError::NotUnicode(_)) => "unsupported-non-unicode".into(),
                }
            };
            (*name, value)
        })
        .collect();
    println!(
        "cargo:rustc-env=WORTH_STORE_MEDIA_PATH_PROFILE={}",
        if storage_profile::is_supported(compiler, &storage_inputs, &lock) {
            "qualified"
        } else {
            "unsupported"
        }
    );
    let mut files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("build.rs")];
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=build.rs");
    let media_sources = Path::new("src/filesystem_media");
    println!("cargo:rerun-if-changed={}", media_sources.display());
    collect_rust_sources(media_sources, &mut files);
    collect_rust_sources(Path::new("build_support"), &mut files);
    files.sort();
    let mut digest = Sha256::new();
    digest.update(compiler.as_bytes());
    digest.update(lock.as_bytes());
    for (name, value) in &storage_inputs {
        digest.update(name.as_bytes());
        digest.update((value.len() as u64).to_le_bytes());
        digest.update(value.as_bytes());
    }
    for path in files {
        let bytes = std::fs::read(&path).expect("filesystem media source must be readable");
        digest.update(path.to_string_lossy().as_bytes());
        digest.update((bytes.len() as u64).to_le_bytes());
        digest.update(bytes);
    }
    for variable in [
        "TARGET",
        "PROFILE",
        "OPT_LEVEL",
        "DEBUG",
        "CARGO_CFG_TARGET_FEATURE",
        "CARGO_FEATURE_CERTIFICATION_TEST_AUTHORITY",
    ] {
        let value = std::env::var(variable).unwrap_or_default();
        digest.update(variable.as_bytes());
        digest.update((value.len() as u64).to_le_bytes());
        digest.update(value.as_bytes());
    }
    println!(
        "cargo:rustc-env=WORTH_STORE_MEDIA_BUILD_ID={:x}",
        digest.finalize()
    );
}

fn collect_rust_sources(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).expect("filesystem media directory must exist") {
        let path = entry
            .expect("filesystem media entry must be readable")
            .path();
        if path.is_dir() {
            if path.file_name().and_then(|value| value.to_str()) != Some("tests") {
                collect_rust_sources(&path, files);
            }
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}
