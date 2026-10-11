//! Cargo-check pair: caller-mintable platform markers open ceremonies; value-gated block them.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "authority-value-gate-forgery-{label}-{}-{nanos}",
        std::process::id()
    ))
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("tools")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn write_file(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parents");
    }
    fs::write(path, contents).expect("write");
}

fn worth_proof_path() -> String {
    workspace_root()
        .join("crates/worth-proof")
        .to_string_lossy()
        .replace('\\', "/")
}

fn cargo_check(manifest: &Path) -> (bool, String) {
    let output = worth_harness::tooling::process_deadline::output(
        Command::new("cargo")
            .arg("check")
            .arg("--manifest-path")
            .arg(manifest)
            .arg("--message-format=short"),
    )
    .expect("cargo check");
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    (output.status.success(), format!("{stdout}{stderr}"))
}

fn write_mintable_provider(root: &Path) {
    write_file(
        root,
        "provider/Cargo.toml",
        &format!(
            r#"[package]
name = "sealing-provider"
version = "0.1.0"
edition = "2021"

[dependencies]
worth-proof = {{ path = "{}" }}

[workspace]
"#,
            worth_proof_path()
        ),
    );
    write_file(
        root,
        "provider/src/lib.rs",
        r#"
use worth_proof::{AuthorityMarker, AuthorityWitness};

/// Hostile: unit struct is caller-constructible.
pub struct EntryAdmission;

impl AuthorityMarker for EntryAdmission {}

pub fn admit_authority_only(_authority: AuthorityWitness<EntryAdmission>) {}
"#,
    );
}

fn write_value_gated_provider(root: &Path) {
    write_file(
        root,
        "provider/Cargo.toml",
        &format!(
            r#"[package]
name = "sealing-provider"
version = "0.1.0"
edition = "2021"

[dependencies]
worth-proof = {{ path = "{}" }}

[workspace]
"#,
            worth_proof_path()
        ),
    );
    write_file(
        root,
        "provider/src/lib.rs",
        r#"
use worth_proof::{AuthorityMarker, AuthorityWitness};

/// Value-gated: private field, no Default, no public constructor.
pub struct EntryAdmission {
    _value_gate: (),
}

impl AuthorityMarker for EntryAdmission {}

pub fn issue_entry_admission() -> AuthorityWitness<EntryAdmission> {
    AuthorityWitness::from_authority_marker(EntryAdmission { _value_gate: () })
}

pub fn admit_authority_only(_authority: AuthorityWitness<EntryAdmission>) {}
"#,
    );
}

/// Caller can construct the provider's platform marker, mint the exact witness,
/// and open the governed ceremony when the marker is not value-gated.
#[test]
fn caller_constructible_platform_marker_opens_governed_ceremony() {
    let root = unique_temp("mintable-opens");
    let _ = fs::remove_dir_all(&root);
    write_mintable_provider(&root);
    write_file(
        &root,
        "attacker/Cargo.toml",
        &format!(
            r#"[package]
name = "sealing-attacker-mintable"
version = "0.1.0"
edition = "2021"

[dependencies]
worth-proof = {{ path = "{}" }}
sealing-provider = {{ path = "../provider" }}

[workspace]
"#,
            worth_proof_path()
        ),
    );
    write_file(
        &root,
        "attacker/src/lib.rs",
        r#"
use worth_proof::AuthorityWitness;
use sealing_provider::{admit_authority_only, EntryAdmission};

pub fn attack() {
    let minted = AuthorityWitness::from_authority_marker(EntryAdmission);
    admit_authority_only(minted);
}
"#,
    );

    let (ok, output) = cargo_check(&root.join("attacker/Cargo.toml"));
    let _ = fs::remove_dir_all(&root);
    assert!(
        ok,
        "caller-constructible platform marker must open the ceremony \
(proves the production mintability hazard):\n{output}"
    );
}

/// Value-gated platform marker: external caller cannot construct it, so cannot
/// mint the exact governed witness demanded by the ceremony.
#[test]
fn value_gated_platform_marker_blocks_caller_minted_ceremony() {
    let root = unique_temp("value-gated-blocks");
    let _ = fs::remove_dir_all(&root);
    write_value_gated_provider(&root);
    write_file(
        &root,
        "attacker/Cargo.toml",
        &format!(
            r#"[package]
name = "sealing-attacker-value-gated"
version = "0.1.0"
edition = "2021"

[dependencies]
worth-proof = {{ path = "{}" }}
sealing-provider = {{ path = "../provider" }}

[workspace]
"#,
            worth_proof_path()
        ),
    );
    write_file(
        &root,
        "attacker/src/lib.rs",
        r#"
use worth_proof::AuthorityWitness;
use sealing_provider::{admit_authority_only, EntryAdmission};

pub fn attack() {
    // EntryAdmission has a private field; external construction must fail.
    let minted = AuthorityWitness::from_authority_marker(EntryAdmission { _value_gate: () });
    admit_authority_only(minted);
}
"#,
    );

    let (ok, output) = cargo_check(&root.join("attacker/Cargo.toml"));
    let _ = fs::remove_dir_all(&root);
    assert!(
        !ok,
        "value-gated platform marker must block caller mint:\n{output}"
    );
    assert!(
        output.contains("private")
            || output.contains("E0451")
            || output.contains("field")
            || output.contains("cannot"),
        "expected private-field / construction denial, got:\n{output}"
    );
}
