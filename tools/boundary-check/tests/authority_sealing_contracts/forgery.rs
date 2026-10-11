//! Real cargo-check proof: open worth-proof substrate vs sealed concrete ceremonies.

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
        "authority-sealing-forgery-{label}-{}-{nanos}",
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

/// Provider crate: value-gated concrete authority types + three concrete ceremonies.
fn write_provider(root: &Path) {
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
use worth_proof::{
    AuthorityMarker, AuthorityProves, AuthorityWitness, CapabilityMarker, CapabilityWitness,
    Proof, ProofMarker,
};

/// Value-gated platform authority: private field, no Default, no public constructor.
pub struct EntryAdmission {
    _value_gate: (),
}

impl AuthorityMarker for EntryAdmission {}

pub struct EntryExecution {
    _value_gate: (),
}

impl CapabilityMarker for EntryExecution {}

pub struct AdmissionEligible;
impl ProofMarker for AdmissionEligible {}
impl AuthorityProves<AdmissionEligible> for EntryAdmission {}

pub fn issue_entry_admission() -> AuthorityWitness<EntryAdmission> {
    AuthorityWitness::from_authority_marker(EntryAdmission { _value_gate: () })
}

pub fn issue_entry_execution() -> CapabilityWitness<EntryExecution> {
    CapabilityWitness::from_capability_marker(EntryExecution { _value_gate: () })
}

pub fn issue_eligibility(
    authority: &AuthorityWitness<EntryAdmission>,
) -> Proof<AdmissionEligible, EntryAdmission> {
    Proof::from_authority_witness(authority)
}

pub fn admit(
    _authority: AuthorityWitness<EntryAdmission>,
    _capability: CapabilityWitness<EntryExecution>,
    _eligibility: Proof<AdmissionEligible, EntryAdmission>,
) {
}

pub fn admit_authority_only(_authority: AuthorityWitness<EntryAdmission>) {}

pub fn admit_capability_only(_capability: CapabilityWitness<EntryExecution>) {}

pub fn admit_proof_only(_eligibility: Proof<AdmissionEligible, EntryAdmission>) {}
"#,
    );
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

#[test]
fn owner_issued_values_satisfy_the_concrete_ceremony() {
    let root = unique_temp("owner-issued");
    let _ = fs::remove_dir_all(&root);
    write_provider(&root);
    write_file(
        &root,
        "caller/Cargo.toml",
        r#"[package]
name = "sealing-legitimate-caller"
version = "0.1.0"
edition = "2021"

[dependencies]
sealing-provider = { path = "../provider" }

[workspace]
"#,
    );
    write_file(
        &root,
        "caller/src/lib.rs",
        r#"
use sealing_provider::{
    admit, issue_eligibility, issue_entry_admission, issue_entry_execution,
};

pub fn legitimate_caller() {
    let authority = issue_entry_admission();
    let capability = issue_entry_execution();
    let eligibility = issue_eligibility(&authority);
    admit(authority, capability, eligibility);
}
"#,
    );

    let (ok, output) = cargo_check(&root.join("caller/Cargo.toml"));
    let _ = fs::remove_dir_all(&root);
    assert!(
        ok,
        "owner-issued authority, capability, and proof must satisfy the ceremony:\n{output}"
    );
}

#[test]
fn forged_marker_constructs_substrate_witnesses_and_proof() {
    let root = unique_temp("substrate-ok");
    let _ = fs::remove_dir_all(&root);
    write_provider(&root);
    write_file(
        &root,
        "attacker/Cargo.toml",
        &format!(
            r#"[package]
name = "sealing-attacker-substrate"
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
        &root,
        "attacker/src/lib.rs",
        r#"
use worth_proof::{
    AuthorityMarker, AuthorityProves, AuthorityWitness, CapabilityMarker, CapabilityWitness,
    Proof, ProofMarker,
};

struct ForgedAuth;
impl AuthorityMarker for ForgedAuth {}

struct ForgedCap;
impl CapabilityMarker for ForgedCap {}

struct ForgedFact;
impl ProofMarker for ForgedFact {}
impl AuthorityProves<ForgedFact> for ForgedAuth {}

pub fn forge() {
    let authority = AuthorityWitness::from_authority_marker(ForgedAuth);
    let capability = CapabilityWitness::from_capability_marker(ForgedCap);
    let proof = Proof::<ForgedFact, ForgedAuth>::from_authority_witness(&authority);
    let _ = (authority, capability, proof);
}
"#,
    );

    let (ok, output) = cargo_check(&root.join("attacker/Cargo.toml"));
    let _ = fs::remove_dir_all(&root);
    assert!(
        ok,
        "forged marker must compile against open worth-proof substrate:\n{output}"
    );
}

fn assert_ceremony_type_mismatch(label: &str, body: &str) {
    let root = unique_temp(label);
    let _ = fs::remove_dir_all(&root);
    write_provider(&root);
    write_file(
        &root,
        "attacker/Cargo.toml",
        &format!(
            r#"[package]
name = "sealing-attacker-{label}"
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
    write_file(&root, "attacker/src/lib.rs", body);

    let (ok, output) = cargo_check(&root.join("attacker/Cargo.toml"));
    let _ = fs::remove_dir_all(&root);
    assert!(!ok, "{label} must fail type-check:\n{output}");
    assert!(
        output.contains("mismatched types")
            || output.contains("E0308")
            || output.contains("expected")
            || output.contains("found"),
        "{label}: expected type mismatch, got:\n{output}"
    );
}

#[test]
fn forged_authority_fails_authority_witness_ceremony() {
    assert_ceremony_type_mismatch(
        "auth-ceremony",
        r#"
use worth_proof::{AuthorityMarker, AuthorityWitness};
use sealing_provider::admit_authority_only;

struct ForgedAuth;
impl AuthorityMarker for ForgedAuth {}

pub fn attack() {
    let forged = AuthorityWitness::from_authority_marker(ForgedAuth);
    admit_authority_only(forged);
}
"#,
    );
}

#[test]
fn forged_capability_fails_capability_witness_ceremony() {
    assert_ceremony_type_mismatch(
        "cap-ceremony",
        r#"
use worth_proof::{CapabilityMarker, CapabilityWitness};
use sealing_provider::admit_capability_only;

struct ForgedCap;
impl CapabilityMarker for ForgedCap {}

pub fn attack() {
    let forged = CapabilityWitness::from_capability_marker(ForgedCap);
    admit_capability_only(forged);
}
"#,
    );
}

#[test]
fn forged_proof_fails_proof_ceremony() {
    assert_ceremony_type_mismatch(
        "proof-ceremony",
        r#"
use worth_proof::{AuthorityMarker, AuthorityProves, AuthorityWitness, Proof, ProofMarker};
use sealing_provider::admit_proof_only;

struct ForgedAuth;
impl AuthorityMarker for ForgedAuth {}

struct ForgedFact;
impl ProofMarker for ForgedFact {}
impl AuthorityProves<ForgedFact> for ForgedAuth {}

pub fn attack() {
    let authority = AuthorityWitness::from_authority_marker(ForgedAuth);
    let forged = Proof::<ForgedFact, ForgedAuth>::from_authority_witness(&authority);
    admit_proof_only(forged);
}
"#,
    );
}
