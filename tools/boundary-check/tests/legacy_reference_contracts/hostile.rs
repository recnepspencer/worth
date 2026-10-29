//! Hostile proofs for the rename ratchet production entrypoint.
//!
//! These exercise the real binary against committed self-authorization,
//! inventory multiplicity, and symlink/junction bypasses.

use super::legacy_reference_fixture::{
    retired_query_token, retired_underscore_fragment, LegacyReferenceTestRepository,
};
use std::fs;

#[test]
fn matching_snapshot_row_cannot_self_authorize_growth() {
    // Working-tree bypass: add occurrence + exact snapshot row together.
    // Empty inception ceiling denies non-empty candidate regardless of match.
    let repo = LegacyReferenceTestRepository::create("self-auth");

    let fragment = retired_underscore_fragment();
    let token = retired_query_token();
    let mut body = String::from("pub fn leak() { let _ = \"");
    body.push_str(&token);
    body.push_str("\"; }\n");
    let column = body.find(fragment.as_str()).expect("fragment in body") + 1;
    let mut snapshot = String::from("schema_version = 1\n\n[[references]]\n");
    snapshot.push_str("path = \"cad/workspaces/worth-contracts/crates/demo/src/lib.rs\"\n");
    snapshot.push_str(&format!("location = \"1:{column}\"\n"));
    snapshot.push_str("fragment = \"");
    snapshot.push_str(&fragment);
    snapshot.push_str("\"\n");
    repo.assemble_canonical_layout(&snapshot);
    repo.write_file(
        "cad/workspaces/worth-contracts/crates/demo/src/lib.rs",
        &body,
    );

    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert!(!ok, "self-authorized growth must fail");
    assert!(
        output.contains("BC6001_LEGACY_REFERENCE_GROWTH"),
        "expected BC6001 for baseline growth, got:\n{output}"
    );
    assert!(
        output.contains("may only shrink")
            || output.contains("baseline growth")
            || output.contains("empty Phase 1"),
        "expected shrink-only guidance, got:\n{output}"
    );
}

#[test]
fn committed_grown_snapshot_cannot_self_authorize_at_head() {
    // CI-shaped bypass: malicious growth is already committed as HEAD.
    // Production must not treat the candidate commit snapshot as prior.
    let repo = LegacyReferenceTestRepository::create("committed-growth");

    repo.assemble_canonical_layout("schema_version = 1\nreferences = []\n");
    repo.write_file(
        "cad/workspaces/worth-contracts/crates/demo/src/lib.rs",
        "pub fn ok() {}\n",
    );
    // This committed ratchet fixture has no governed Query or Bank workspace.
    repo.write_file("tools/boundary-check/config/test-workspaces.txt", "");
    repo.write_file("tools/boundary-check/config/test-binary-exceptions.txt", "");
    repo.git_init_commit_all();

    let (ok, output) = repo.run_boundary_check();
    assert!(ok, "empty inception commit must pass, got:\n{output}");

    let fragment = retired_underscore_fragment();
    let mut body = String::from("pub fn leak() { let _ = \"");
    body.push_str(&retired_query_token());
    body.push_str("\"; }\n");
    let column = body.find(fragment.as_str()).expect("fragment in body") + 1;
    let mut snapshot = String::from("schema_version = 1\n\n[[references]]\n");
    snapshot.push_str("path = \"cad/workspaces/worth-contracts/crates/demo/src/lib.rs\"\n");
    snapshot.push_str(&format!("location = \"1:{column}\"\n"));
    snapshot.push_str("fragment = \"");
    snapshot.push_str(&fragment);
    snapshot.push_str("\"\n");
    repo.write_file(
        "cad/workspaces/worth-contracts/crates/demo/src/lib.rs",
        &body,
    );
    repo.write_file(
        "tools/boundary-check/snapshots/legacy-references.toml",
        &snapshot,
    );
    repo.git_commit_all("malicious growth as HEAD");

    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert!(!ok, "committed candidate growth at HEAD must fail");
    assert!(
        output.contains("BC6001_LEGACY_REFERENCE_GROWTH"),
        "expected BC6001 for committed growth, got:\n{output}"
    );
}

#[test]
fn same_line_repeated_fragment_is_fully_inventoried() {
    let repo = LegacyReferenceTestRepository::create("multiplicity");
    repo.assemble_canonical_layout("schema_version = 1\nreferences = []\n");

    let fragment = retired_underscore_fragment();
    let mut body = String::from("let _ = (\"");
    body.push_str(&fragment);
    body.push_str("a\", \"");
    body.push_str(&fragment);
    body.push_str("b\");\n");
    repo.write_file(
        "cad/workspaces/worth-contracts/crates/demo/src/lib.rs",
        &body,
    );

    let first = body.find(fragment.as_str()).expect("first") + 1;
    let second = body[first..].find(fragment.as_str()).expect("second") + first + 1;

    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert!(!ok, "multiplicity growth must fail");
    assert!(
        output.contains(&format!("1:{first}")),
        "expected first location 1:{first}, got:\n{output}"
    );
    assert!(
        output.contains(&format!("1:{second}")),
        "expected second location 1:{second}, got:\n{output}"
    );
}

#[test]
fn governed_symlink_or_junction_is_rejected() {
    let repo = LegacyReferenceTestRepository::create("symlink");
    repo.assemble_canonical_layout("schema_version = 1\nreferences = []\n");
    repo.write_file(
        "cad/workspaces/worth-contracts/crates/demo/src/lib.rs",
        "pub fn ok() {}\n",
    );

    let outside = repo.path().join("outside_payload");
    fs::create_dir_all(&outside).expect("outside dir");
    let mut payload = String::from("retired ");
    payload.push_str(&retired_query_token());
    payload.push('\n');
    fs::write(outside.join("hidden.rs"), payload).expect("write outside");

    // Use Path::join so the OS path form is correct for mklink / junction.
    let link = repo
        .path()
        .join("cad")
        .join("workspaces")
        .join("worth-contracts")
        .join("crates")
        .join("demo")
        .join("escaped");
    repo.create_governed_link(&outside, &link);

    let (ok, output) = repo.run_boundary_check();
    repo.cleanup();
    assert!(!ok, "governed symlink/junction must fail closed");
    assert!(
        output.contains("BC6002_LEGACY_REFERENCE_BASELINE"),
        "expected BC6002 for symlink, got:\n{output}"
    );
    assert!(
        output.contains("symlink") || output.contains("junction"),
        "expected symlink/junction message, got:\n{output}"
    );
}
