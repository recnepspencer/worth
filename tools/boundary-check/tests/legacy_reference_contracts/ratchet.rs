use super::legacy_reference_fixture::{
    retired_hyphen_fragment, retired_hyphen_query_token, retired_query_token,
    retired_underscore_fragment, LegacyReferenceTestRepository,
};

#[test]
fn clean_empty_baseline_passes() {
    let repo = LegacyReferenceTestRepository::create("clean");
    repo.assemble_canonical_layout("schema_version = 1\nreferences = []\n");
    repo.write_file(
        "cad/workspaces/worth-contracts/crates/demo/src/lib.rs",
        "pub fn ok() {}\n",
    );

    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert!(ok, "clean baseline should pass, got:\n{output}");
}

#[test]
fn new_governed_legacy_reference_is_denied() {
    let repo = LegacyReferenceTestRepository::create("growth");
    repo.assemble_canonical_layout("schema_version = 1\nreferences = []\n");

    let mut body = String::from("pub fn leak() { let _ = \"");
    body.push_str(&retired_query_token());
    body.push_str("\"; }\n");
    repo.write_file(
        "cad/workspaces/worth-contracts/crates/demo/src/lib.rs",
        &body,
    );

    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert!(!ok, "growth should fail");
    assert!(
        output.contains("BC6001_LEGACY_REFERENCE_GROWTH"),
        "expected BC6001, got:\n{output}"
    );
    assert!(
        output.contains("worth_"),
        "expected worth replacement guidance, got:\n{output}"
    );
    assert!(
        output.contains("tools/boundary-check/snapshots/legacy-references.toml"),
        "expected snapshot pointer, got:\n{output}"
    );
}

#[test]
fn docs_outside_governed_roots_are_ignored() {
    let repo = LegacyReferenceTestRepository::create("docs-scope");
    repo.assemble_canonical_layout("schema_version = 1\nreferences = []\n");

    let mut body = String::from("# history mentions ");
    body.push_str(&retired_query_token());
    body.push('\n');
    repo.write_file("plans/history.md", &body);
    repo.write_file(
        "cad/workspaces/worth-contracts/crates/demo/src/lib.rs",
        "pub fn ok() {}\n",
    );

    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert!(
        ok,
        "retired spelling under plans/ must be out of scope, got:\n{output}"
    );
}

#[test]
fn malformed_snapshot_fails_closed() {
    let repo = LegacyReferenceTestRepository::create("malformed");
    repo.assemble_canonical_layout("schema_version = 1\nreferences = [ { not = \"valid\" } ]\n");
    repo.write_file(
        "cad/workspaces/worth-contracts/crates/demo/src/lib.rs",
        "pub fn ok() {}\n",
    );

    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert!(!ok, "malformed snapshot must fail closed");
    assert!(
        output.contains("BC6002_LEGACY_REFERENCE_BASELINE")
            || output.contains("parse legacy-reference snapshot"),
        "expected baseline/parse failure, got:\n{output}"
    );
}

#[test]
fn drifting_snapshot_row_fails_closed() {
    let repo = LegacyReferenceTestRepository::create("drift");

    let mut snapshot = String::from("schema_version = 1\n\n[[references]]\n");
    snapshot.push_str("path = \"cad/workspaces/worth-contracts/crates/demo/src/lib.rs\"\n");
    snapshot.push_str("location = \"1:1\"\n");
    snapshot.push_str("fragment = \"");
    snapshot.push_str(&retired_underscore_fragment());
    snapshot.push_str("\"\n");
    repo.assemble_canonical_layout(&snapshot);
    repo.write_file(
        "cad/workspaces/worth-contracts/crates/demo/src/lib.rs",
        "pub fn ok() {}\n",
    );

    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert!(!ok, "stale baseline row must fail closed");
    assert!(
        output.contains("BC6002_LEGACY_REFERENCE_BASELINE")
            || output.contains("BC6001_LEGACY_REFERENCE_GROWTH"),
        "expected BC6001/BC6002, got:\n{output}"
    );
}

#[test]
fn duplicate_snapshot_rows_fail_closed() {
    let repo = LegacyReferenceTestRepository::create("duplicate");

    let fragment = retired_underscore_fragment();
    let mut snapshot = String::from("schema_version = 1\n\n");
    for _ in 0..2 {
        snapshot.push_str("[[references]]\n");
        snapshot.push_str("path = \"cad/workspaces/worth-contracts/crates/demo/src/lib.rs\"\n");
        snapshot.push_str("location = \"1:1\"\n");
        snapshot.push_str("fragment = \"");
        snapshot.push_str(&fragment);
        snapshot.push_str("\"\n\n");
    }
    repo.assemble_canonical_layout(&snapshot);

    let mut body = String::from("pub fn leak() { let _ = \"");
    body.push_str(&retired_query_token());
    body.push_str("\"; }\n");
    repo.write_file(
        "cad/workspaces/worth-contracts/crates/demo/src/lib.rs",
        &body,
    );

    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert!(!ok, "duplicate baseline rows must fail closed");
    assert!(
        output.contains("BC6002_LEGACY_REFERENCE_BASELINE"),
        "expected BC6002, got:\n{output}"
    );
}

/// Matrix proof: both forbidden spellings under every governed root.
///
/// Each cell is independently observable in denial output so a scanner that
/// hard-codes only the underscore fragment under cad/workspaces cannot stay green.
#[test]
fn forbidden_spelling_and_governed_root_matrix_is_denied() {
    let repo = LegacyReferenceTestRepository::create("matrix");
    repo.assemble_canonical_layout("schema_version = 1\nreferences = []\n");
    // Keep a clean cad/workspaces seed so the matrix cells own the failures.
    repo.write_file(
        "cad/workspaces/worth-contracts/crates/demo/src/lib.rs",
        "pub fn ok() {}\n",
    );

    let underscore = retired_underscore_fragment();
    let hyphen = retired_hyphen_fragment();
    let underscore_token = retired_query_token();
    let hyphen_token = retired_hyphen_query_token();

    let cells: [(&str, &str, &str); 6] = [
        (
            "cad/workspaces/worth-contracts/crates/demo/src/hyphen_leak.rs",
            &hyphen_token,
            &hyphen,
        ),
        (
            "tools/matrix_underscore_leak.rs",
            &underscore_token,
            &underscore,
        ),
        ("tools/matrix_hyphen_leak.rs", &hyphen_token, &hyphen),
        (
            "crates/worth-proof/src/matrix_underscore_leak.rs",
            &underscore_token,
            &underscore,
        ),
        (
            "crates/worth-proof/src/matrix_hyphen_leak.rs",
            &hyphen_token,
            &hyphen,
        ),
        // Second cad/workspaces cell with underscore keeps that spelling
        // independently visible under the primary root as well.
        (
            "cad/workspaces/worth-contracts/crates/demo/src/underscore_leak.rs",
            &underscore_token,
            &underscore,
        ),
    ];

    for (path, token, _fragment) in cells {
        let mut body = String::from("// retired occurrence: ");
        body.push_str(token);
        body.push('\n');
        repo.write_file(path, &body);
    }

    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert!(!ok, "matrix growth must fail, got:\n{output}");
    assert!(
        output.contains("BC6001_LEGACY_REFERENCE_GROWTH"),
        "expected BC6001, got:\n{output}"
    );
    assert!(
        output.contains("worth_") || output.contains("worth-"),
        "expected worth replacement guidance, got:\n{output}"
    );
    assert!(
        output.contains("tools/boundary-check/snapshots/legacy-references.toml"),
        "expected snapshot pointer, got:\n{output}"
    );

    for (path, _token, fragment) in cells {
        assert!(
            output.contains(path),
            "expected denial subject path {path}, got:\n{output}"
        );
        assert!(
            output.contains(fragment),
            "expected fragment `{fragment}` for path {path}, got:\n{output}"
        );
    }
}
