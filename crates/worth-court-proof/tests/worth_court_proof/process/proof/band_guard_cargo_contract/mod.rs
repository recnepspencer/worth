#[allow(unused_imports)]
use crate::band_guard_cargo_contract::process_context::*;

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

    let _ = std::fs::remove_dir_all(target_dir);
}

#[test]
fn empty_prefix_cannot_disable_the_expansion_fence() {
    let target_dir = isolated_root("empty-prefix-target");
    let output = run_cargo(
        &["check", "--package", "empty-prefix-consumer"],
        &fixture_manifest(),
        &target_dir,
    );
    let _ = std::fs::remove_dir_all(target_dir);
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
    let _ = std::fs::remove_dir_all(target_dir);
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

    let _ = std::fs::remove_dir_all(&specimen_root);
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
