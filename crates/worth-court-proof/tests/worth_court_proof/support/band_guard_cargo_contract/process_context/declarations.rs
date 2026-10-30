pub(crate) use std::fs;
pub(crate) use {std::path::Path, std::path::PathBuf};
pub(crate) use {std::process::Command, std::process::Output};
pub(crate) use {std::time::SystemTime, std::time::UNIX_EPOCH};

pub(crate) fn worth_proof_root() -> PathBuf {
    std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-proof"))
}

pub(crate) fn fixture_manifest() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "tests/worth_court_proof/sources/worth_proof/fixtures/band_guard_cargo_contract/Cargo.toml",
    )
}

pub(crate) fn isolated_root(label: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock must be after Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "worth-proof-{label}-{}-{nonce}",
        std::process::id()
    ))
}

pub(crate) fn run_cargo(arguments: &[&str], manifest: &Path, target_dir: &Path) -> Output {
    std::process::Command::new("cargo")
        .args(arguments)
        .arg("--manifest-path")
        .arg(manifest)
        .arg("--offline")
        .env("CARGO_TARGET_DIR", target_dir)
        .output()
        .unwrap_or_else(|error| panic!("failed to run cargo for {}: {error}", manifest.display()))
}

pub(crate) fn compiler_output(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

pub(crate) fn assert_package_compiles(package: &str, target_dir: &Path) {
    let output = run_cargo(
        &["check", "--package", package],
        &fixture_manifest(),
        target_dir,
    );
    assert!(
        output.status.success(),
        "legal fixture {package} failed:\n{}",
        compiler_output(&output)
    );
}

pub(crate) fn assert_package_is_denied(package: &str, legal_prefixes: &[&str], target_dir: &Path) {
    let output = run_cargo(
        &["check", "--package", package],
        &fixture_manifest(),
        target_dir,
    );
    let output_text = compiler_output(&output);
    assert!(
        !output.status.success(),
        "wrong-band fixture {package} unexpectedly compiled"
    );
    let legal_list = format!("legal package prefixes: {}", legal_prefixes.join(", "));
    for required_text in [
        legal_list.as_str(),
        "cad/docs/worthy-foundations/BOUNDARIES.md",
        package,
    ] {
        assert!(
            output_text.contains(required_text),
            "diagnostic for {package} omitted {required_text:?}:\n{output_text}"
        );
    }
}

pub(crate) fn write_zero_cost_specimen(root: &Path) {
    std::fs::create_dir_all(root.join("src")).expect("create zero-cost specimen source directory");
    let canonical_dependency_path = worth_proof_root()
        .canonicalize()
        .expect("canonical worth-proof path")
        .to_string_lossy()
        .replace('\\', "/");
    let dependency_path = canonical_dependency_path
        .strip_prefix("//?/")
        .unwrap_or(&canonical_dependency_path);
    std::fs::write(
        root.join("Cargo.toml"),
        format!(
            r#"[package]
name = "zero-cost-band-guard"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
worth-proof = {{ path = "{dependency_path}" }}

[workspace]
"#,
        ),
    )
    .expect("write zero-cost specimen manifest");
}

pub(crate) fn write_specimen_source(root: &Path, guarded: bool) {
    let guard = if guarded {
        "worth_proof::band_guard!(\"zero-cost-\");\n"
    } else {
        "use worth_proof as _;\n"
    };
    std::fs::write(
        root.join("src/lib.rs"),
        format!(
            r#"{guard}
#[no_mangle]
pub extern "C" fn observed_runtime_surface(value: u32) -> u32 {{
    value.wrapping_add(1)
}}
"#,
        ),
    )
    .expect("write zero-cost specimen source");
}

pub(crate) fn compile_runtime_surface(manifest: &Path, target_dir: &Path) -> Vec<String> {
    remove_llvm_ir(target_dir);
    let output = std::process::Command::new("cargo")
        .args(["rustc", "--release", "--lib"])
        .arg("--manifest-path")
        .arg(manifest)
        .arg("--offline")
        .args(["--", "--emit=llvm-ir"])
        .env("CARGO_TARGET_DIR", target_dir)
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {}: {error}", manifest.display()));
    assert!(
        output.status.success(),
        "zero-cost specimen failed:\n{}",
        compiler_output(&output)
    );
    let llvm_ir = llvm_ir_path(target_dir);
    std::fs::read_to_string(&llvm_ir)
        .unwrap_or_else(|error| panic!("read {}: {error}", llvm_ir.display()))
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("define ") || line.starts_with('@') && line.contains(" = "))
        .map(str::to_owned)
        .collect()
}

pub(crate) fn remove_llvm_ir(target_dir: &Path) {
    let deps = target_dir.join("release/deps");
    if let Ok(entries) = std::fs::read_dir(deps) {
        for entry in entries.flatten() {
            if entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "ll")
            {
                std::fs::remove_file(entry.path()).expect("remove prior LLVM IR");
            }
        }
    }
}

pub(crate) fn llvm_ir_path(target_dir: &Path) -> PathBuf {
    let paths: Vec<_> = std::fs::read_dir(target_dir.join("release/deps"))
        .expect("read release artifacts")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "ll"))
        .collect();
    assert_eq!(paths.len(), 1, "expected one LLVM IR artifact: {paths:#?}");
    paths.into_iter().next().expect("one LLVM IR artifact")
}
