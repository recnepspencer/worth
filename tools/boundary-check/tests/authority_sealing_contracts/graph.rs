//! Production-binary matrix: BC7001 inventories the Cargo/rustc library surface.

use super::authority_sealing_fixture::{
    cfg_attr_path_root, hostile_cfg_modules_hostile_then_safe,
    hostile_cfg_modules_safe_then_hostile, hostile_gate_path_body, hostile_ordinary_api_lib,
    legal_ordinary_api_lib, nested_cfg_attr_path_root, nested_inline_path_attr_root,
    nested_outline_path_parent_rs, nested_outline_path_root, path_attr_real_gate_rs,
    path_attr_root, AuthoritySealingTestRepository,
};

fn assert_denial(label: &str, ok: bool, output: &str) {
    assert!(!ok, "{label} must fail authority sealing:\n{output}");
    assert!(
        output.contains("BC7001_AUTHORITY_SEALING"),
        "{label}: expected BC7001, got:\n{output}"
    );
    assert!(
        output.contains("Authority sealing law"),
        "{label}: expected law quote, got:\n{output}"
    );
}

fn assert_pass(label: &str, ok: bool, output: &str) {
    assert!(ok, "{label} must pass:\n{output}");
    assert!(
        !output.contains("BC7001_AUTHORITY_SEALING"),
        "{label}: unexpected sealing diagnostic:\n{output}"
    );
}

#[test]
fn custom_lib_path_hostile_ceremony_is_denied() {
    let repo = AuthoritySealingTestRepository::create("hostile-lib-path");
    repo.assemble_with_lib_path_and_files(
        "src/ordinary_api.rs",
        &[("src/ordinary_api.rs", hostile_ordinary_api_lib())],
    );
    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert_denial("hostile-lib-path", ok, &output);
}

#[test]
fn custom_lib_path_concrete_ceremony_passes() {
    let repo = AuthoritySealingTestRepository::create("legal-lib-path");
    repo.assemble_with_lib_path_and_files(
        "src/ordinary_api.rs",
        &[("src/ordinary_api.rs", legal_ordinary_api_lib())],
    );
    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert_pass("legal-lib-path", ok, &output);
}

#[test]
fn cfg_exclusive_modules_safe_then_hostile_is_denied() {
    let repo = AuthoritySealingTestRepository::create("hostile-cfg-safe-first");
    repo.assemble_with_lib_source(hostile_cfg_modules_safe_then_hostile());
    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert_denial("hostile-cfg-safe-first", ok, &output);
}

#[test]
fn cfg_exclusive_modules_hostile_then_safe_is_denied() {
    let repo = AuthoritySealingTestRepository::create("hostile-cfg-hostile-first");
    repo.assemble_with_lib_source(hostile_cfg_modules_hostile_then_safe());
    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert_denial("hostile-cfg-hostile-first", ok, &output);
}

#[test]
fn path_attr_with_conventional_decoy_is_denied() {
    let repo = AuthoritySealingTestRepository::create("hostile-path-attr");
    repo.assemble_with_lib_path_and_files(
        "src/lib.rs",
        &[
            ("src/lib.rs", path_attr_root()),
            ("src/real_gate.rs", path_attr_real_gate_rs()),
        ],
    );
    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert_denial("hostile-path-attr", ok, &output);
}

#[test]
fn conventional_lib_root_concrete_still_passes() {
    let repo = AuthoritySealingTestRepository::create("legal-conventional-root");
    repo.assemble_with_lib_source(legal_ordinary_api_lib());
    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert_pass("legal-conventional-root", ok, &output);
}

#[test]
fn nested_inline_path_attr_is_denied() {
    let repo = AuthoritySealingTestRepository::create("hostile-nested-inline-path");
    repo.assemble_with_lib_path_and_files(
        "src/lib.rs",
        &[
            ("src/lib.rs", nested_inline_path_attr_root()),
            ("src/outer/hostile_gate.rs", hostile_gate_path_body()),
        ],
    );
    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert_denial("hostile-nested-inline-path", ok, &output);
}

#[test]
fn nested_outline_path_attr_is_denied() {
    let repo = AuthoritySealingTestRepository::create("hostile-nested-outline-path");
    repo.assemble_with_lib_path_and_files(
        "src/lib.rs",
        &[
            ("src/lib.rs", nested_outline_path_root()),
            ("src/outer.rs", nested_outline_path_parent_rs()),
            ("src/hostile_gate.rs", hostile_gate_path_body()),
        ],
    );
    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert_denial("hostile-nested-outline-path", ok, &output);
}

#[test]
fn cfg_attr_path_root_is_denied() {
    let repo = AuthoritySealingTestRepository::create("hostile-cfg-attr-path");
    repo.assemble_with_lib_path_and_files(
        "src/lib.rs",
        &[
            ("src/lib.rs", cfg_attr_path_root()),
            ("src/hostile_gate.rs", hostile_gate_path_body()),
        ],
    );
    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert_denial("hostile-cfg-attr-path", ok, &output);
}

#[test]
fn nested_cfg_attr_path_is_denied() {
    let repo = AuthoritySealingTestRepository::create("hostile-nested-cfg-attr-path");
    repo.assemble_with_lib_path_and_files(
        "src/lib.rs",
        &[
            ("src/lib.rs", nested_cfg_attr_path_root()),
            ("src/outer/hostile_gate.rs", hostile_gate_path_body()),
        ],
    );
    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert_denial("hostile-nested-cfg-attr-path", ok, &output);
}

#[test]
fn sibling_path_concrete_ceremony_passes() {
    let repo = AuthoritySealingTestRepository::create("legal-sibling-path");
    repo.assemble_with_lib_source("#[path = \"../../../shared-source/gate.rs\"] pub mod gate;\n");
    repo.write_file(
        "cad/workspaces/worth-entry/shared-source/gate.rs",
        legal_ordinary_api_lib(),
    );
    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert_pass("legal-sibling-path", ok, &output);
}

#[test]
fn sibling_path_hostile_ceremony_keeps_consuming_namespace_and_locator() {
    let repo = AuthoritySealingTestRepository::create("hostile-sibling-path");
    repo.assemble_with_lib_source("#[path = \"../../../shared-source/gate.rs\"] pub mod gate;\n");
    repo.write_file(
        "cad/workspaces/worth-entry/shared-source/gate.rs",
        hostile_gate_path_body(),
    );
    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert_denial("hostile-sibling-path", ok, &output);
    assert!(
        output.contains(
            "worth-entry-adoption::../../shared-source/gate.rs::test_surface::gate::admit"
        ),
        "expected consuming namespace and sibling locator:\n{output}"
    );
    assert!(output.contains("Offending trait bound: `AuthorityMarker`"));
    assert!(!output.contains("could not parse governed crate"));
}

#[test]
fn canonical_and_noncanonical_incrate_and_sibling_paths_share_identity() {
    let repo = AuthoritySealingTestRepository::create("source-path-identity");
    repo.assemble_with_lib_source(
        r#"
#[path = "local_gate.rs"]
#[cfg_attr(any(), path = "../src/./local_gate.rs")]
pub mod local;
#[path = "../../../shared-source/gate.rs"]
#[cfg_attr(any(), path = "../../../shared-source/../shared-source/./gate.rs")]
pub mod sibling;
"#,
    );
    repo.write_file(
        "cad/workspaces/worth-entry/crates/worth-entry-adoption/src/local_gate.rs",
        hostile_gate_path_body(),
    );
    repo.write_file(
        "cad/workspaces/worth-entry/shared-source/gate.rs",
        hostile_gate_path_body(),
    );
    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert_denial("source-path-identity", ok, &output);
    for subject in [
        "worth-entry-adoption::src/local_gate.rs::test_surface::local::admit",
        "worth-entry-adoption::../../shared-source/gate.rs::test_surface::sibling::admit",
    ] {
        assert_eq!(output.matches(subject).count(), 1, "{output}");
    }
    assert!(!output.contains("local_gate.rs;"), "{output}");
    assert!(!output.contains("gate.rs;"), "{output}");
    assert!(!output.contains("could not parse governed crate"));
}

#[test]
fn missing_sibling_path_fails_closed() {
    let repo = AuthoritySealingTestRepository::create("missing-sibling-path");
    repo.assemble_with_lib_source(
        "#[path = \"../../../shared-source/missing.rs\"] pub mod gate;\n",
    );
    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert!(!ok, "missing external selector must fail:\n{output}");
    assert!(output.contains("BC7001_AUTHORITY_SEALING"), "{output}");
    assert!(
        output.contains("unresolved #[path = \"../../../shared-source/missing.rs\"]"),
        "expected missing selector refusal:\n{output}"
    );
}
