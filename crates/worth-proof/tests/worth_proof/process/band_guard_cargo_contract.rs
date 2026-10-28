use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn worth_proof_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_manifest() -> PathBuf {
    worth_proof_root().join("tests/fixtures/band_guard_cargo_contract/Cargo.toml")
}

fn isolated_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock must be after Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "worth-proof-{label}-{}-{nonce}",
        std::process::id()
    ))
}

fn run_cargo(arguments: &[&str], manifest: &Path, target_dir: &Path) -> Output {
    Command::new("cargo")
        .args(arguments)
        .arg("--manifest-path")
        .arg(manifest)
        .arg("--offline")
        .env("CARGO_TARGET_DIR", target_dir)
        .output()
        .unwrap_or_else(|error| panic!("failed to run cargo for {}: {error}", manifest.display()))
}

fn compiler_output(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_package_compiles(package: &str, target_dir: &Path) {
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

fn assert_package_is_denied(package: &str, legal_prefixes: &[&str], target_dir: &Path) {
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

#[test]
fn macro_arguments_select_disjoint_expanding_package_bands() {
    let target_dir = isolated_root("band-policy-target");

    for legal_package in [
        "worth-entry-consumer",
        "worthy-entry-consumer",
        "worth-cert-consumer",
    ] {
        assert_package_compiles(legal_package, &target_dir);
    }
    assert_package_is_denied(
        "worth-schema-consumer",
        &["worth-entry-", "worthy-entry-"],
        &target_dir,
    );
    assert_package_is_denied(
        "worth-entry-cert-consumer",
        &["worth-cert-", "worthy-cert-"],
        &target_dir,
    );

    let _ = fs::remove_dir_all(target_dir);
}

#[test]
fn empty_prefix_cannot_disable_the_expansion_fence() {
    let target_dir = isolated_root("empty-prefix-target");
    let output = run_cargo(
        &["check", "--package", "empty-prefix-consumer"],
        &fixture_manifest(),
        &target_dir,
    );
    let _ = fs::remove_dir_all(target_dir);
    let output_text = compiler_output(&output);
    assert!(
        !output.status.success(),
        "empty prefix unexpectedly compiled"
    );
    assert!(
        output_text.contains("worth_proof::band_guard! rejected package"),
        "empty-prefix denial omitted the band-guard diagnostic:\n{output_text}"
    );
}

#[test]
fn production_package_has_no_normal_dependencies() {
    let target_dir = isolated_root("dependency-target");
    let output = run_cargo(
        &[
            "tree",
            "--package",
            "worth-proof",
            "--edges",
            "normal",
            "--depth",
            "1",
            "--prefix",
            "none",
        ],
        &worth_proof_root().join("Cargo.toml"),
        &target_dir,
    );
    let _ = fs::remove_dir_all(target_dir);
    assert!(
        output.status.success(),
        "cargo tree failed:\n{}",
        compiler_output(&output)
    );
    let resolved_packages: Vec<_> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(str::to_owned)
        .collect();
    assert_eq!(
        resolved_packages.len(),
        1,
        "worth-proof gained a normal dependency: {resolved_packages:#?}"
    );
    assert!(resolved_packages[0].starts_with("worth-proof v"));
}

#[test]
fn successful_guard_expansion_adds_no_runtime_artifact_surface() {
    let specimen_root = isolated_root("zero-cost-specimen");
    write_zero_cost_specimen(&specimen_root);
    let manifest = specimen_root.join("Cargo.toml");
    let target_dir = specimen_root.join("target");

    write_specimen_source(&specimen_root, false);
    let baseline_surface = compile_runtime_surface(&manifest, &target_dir);
    write_specimen_source(&specimen_root, true);
    let guarded_surface = compile_runtime_surface(&manifest, &target_dir);

    let _ = fs::remove_dir_all(&specimen_root);
    assert!(
        baseline_surface
            .iter()
            .any(|definition| definition.contains("observed_runtime_surface")),
        "artifact inspection missed the specimen's exported runtime function: {baseline_surface:#?}"
    );
    assert_eq!(
        guarded_surface, baseline_surface,
        "a successful band guard expansion changed release LLVM runtime definitions"
    );
}

fn write_zero_cost_specimen(root: &Path) {
    fs::create_dir_all(root.join("src")).expect("create zero-cost specimen source directory");
    let canonical_dependency_path = worth_proof_root()
        .canonicalize()
        .expect("canonical worth-proof path")
        .to_string_lossy()
        .replace('\\', "/");
    let dependency_path = canonical_dependency_path
        .strip_prefix("//?/")
        .unwrap_or(&canonical_dependency_path);
    fs::write(
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

fn write_specimen_source(root: &Path, guarded: bool) {
    let guard = if guarded {
        "worth_proof::band_guard!(\"zero-cost-\");\n"
    } else {
        "use worth_proof as _;\n"
    };
    fs::write(
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

fn compile_runtime_surface(manifest: &Path, target_dir: &Path) -> Vec<String> {
    remove_llvm_ir(target_dir);
    let output = Command::new("cargo")
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
    fs::read_to_string(&llvm_ir)
        .unwrap_or_else(|error| panic!("read {}: {error}", llvm_ir.display()))
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("define ") || line.starts_with('@') && line.contains(" = "))
        .map(str::to_owned)
        .collect()
}

fn remove_llvm_ir(target_dir: &Path) {
    let deps = target_dir.join("release/deps");
    if let Ok(entries) = fs::read_dir(deps) {
        for entry in entries.flatten() {
            if entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "ll")
            {
                fs::remove_file(entry.path()).expect("remove prior LLVM IR");
            }
        }
    }
}

fn llvm_ir_path(target_dir: &Path) -> PathBuf {
    let paths: Vec<_> = fs::read_dir(target_dir.join("release/deps"))
        .expect("read release artifacts")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "ll"))
        .collect();
    assert_eq!(paths.len(), 1, "expected one LLVM IR artifact: {paths:#?}");
    paths.into_iter().next().expect("one LLVM IR artifact")
}
