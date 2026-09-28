//! Temporary-repository fixture for Query audience production-binary tests.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn unique_temp_root(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "boundary-check-query-audience-{label}-{}-{nanos}",
        std::process::id()
    ))
}

pub fn write_file(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parents");
    }
    fs::write(path, contents).expect("write");
}

fn write_stub_lib(root: &Path, relative_crate: &str, package_name: &str) {
    write_file(
        root,
        &format!("{relative_crate}/Cargo.toml"),
        &format!(
            r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2021"
[workspace]
"#
        ),
    );
    write_file(root, &format!("{relative_crate}/src/lib.rs"), "// stub\n");
}

pub fn write_root_shell(root: &Path, config_body: &str) {
    write_file(root, "cad/docs/worthy-foundations/NAMING.md", "# naming\n");
    // seed_contracts discovers under cad/workspaces; keep an empty lane present.
    fs::create_dir_all(root.join("cad/workspaces")).expect("cad/workspaces");
    write_file(
        root,
        "Cargo.toml",
        r#"[workspace]
resolver = "2"
exclude = ["cad/workspaces/*"]
members = []

[workspace.metadata.worth_topology]
role = "thin_orchestrator"
road1_subworkspaces = []
forbidden_member_prefixes = ["cad/workspaces/"]
boundary_check_manifest = "tools/boundary-check/Cargo.toml"
boundary_check_config = "tools/boundary-check/config/road1.toml"
"#,
    );
    write_file(
        root,
        "tools/boundary-check/Cargo.toml",
        r#"[package]
name = "boundary-check"
version = "0.1.0"
edition = "2021"
publish = false
[workspace]
"#,
    );
    write_file(
        root,
        "tools/boundary-check/snapshots/legacy-references.toml",
        "schema_version = 1\nreferences = []\n",
    );
    write_file(root, "tools/boundary-check/config/road1.toml", config_body);
}

fn query_audience_config_fragment() -> &'static str {
    r#"[rule_contracts.query_audience]
engine_package = "worth-query"

[[rule_contracts.query_audience.audiences]]
package = "worth-query-decl"
label = "declaration"
allowed_bands = ["entry", "cert"]
guidance = "declaration artifacts and handles"

[[rule_contracts.query_audience.audiences]]
package = "worth-query-host"
label = "host"
allowed_bands = ["entry", "cert"]
guidance = "admission, lowering, and execution"

[[rule_contracts.query_audience.audiences]]
package = "worth-query-replay"
label = "replay"
allowed_bands = ["cert"]
guidance = "cert-only reconstruction and replay"
"#
}

pub fn base_config(
    subworkspaces: &str,
    born_crates: &str,
    reserved: &str,
    band_rules: &str,
) -> String {
    let mut config = String::new();
    config.push_str("root_manifest = \"Cargo.toml\"\n");
    config.push_str("forbidden_root_prefixes = [\"cad/workspaces/\"]\n");
    config.push_str("seed_skeletons = []\n");
    if born_crates.trim().is_empty() {
        config.push_str("born_crates = []\n");
    }
    if subworkspaces.trim().is_empty() {
        config.push_str("subworkspaces = []\n");
    }
    config.push('\n');
    config.push_str("[machine_authority]\n");
    config.push_str("canonical_config = \"tools/boundary-check/config/road1.toml\"\n");
    config.push_str("mirrored_docs = [\"cad/docs/worthy-foundations/NAMING.md\"]\n\n");
    config.push_str("[naming]\n");
    config.push_str("bands = [\"schema\", \"entry\", \"derived\", \"cert\", \"pack\"]\n");
    config.push_str(reserved);
    config.push('\n');
    config.push_str("[[law_substrates]]\n");
    config.push_str("package = \"worth-proof\"\n");
    config.push_str("tiers = [\"worth\", \"worthy\"]\n");
    config.push_str("bands = [\"schema\", \"entry\", \"derived\", \"cert\", \"pack\"]\n\n");
    config.push_str("[rule_contracts]\n");
    config.push_str(query_audience_config_fragment());
    config.push('\n');
    config.push_str("[[rule_contracts.replay_surfaces]]\n");
    config.push_str("label = \"certification replay\"\n");
    config.push_str("package_prefixes = [\"worth-cert-replay\", \"worthy-cert-replay\"]\n");
    config.push_str("cert_domains = [\"replay\", \"reconstruction\"]\n\n");
    config.push_str(band_rules);
    config.push('\n');
    if !born_crates.trim().is_empty() {
        config.push_str(born_crates);
        config.push('\n');
    }
    if !subworkspaces.trim().is_empty() {
        config.push_str(subworkspaces);
        config.push('\n');
    }
    config.push_str("[legacy_reference_ratchet]\n");
    config.push_str("governed_roots = []\n");
    config.push_str("forbidden_fragments = []\n");
    config.push_str("snapshot = \"tools/boundary-check/snapshots/legacy-references.toml\"\n");
    config.push_str("exclude_paths = []\n");
    config.push_str(
        "replacement_guidance = \"Use the corresponding worth_/worth- spelling instead of the retired name.\"\n",
    );
    config
}

pub fn run_boundary_check(root: &Path) -> (bool, String) {
    let snapshot = root.join("tools/boundary-check/snapshots/crate-dag.toml");
    if !snapshot.exists() {
        if let Err(error) = seed_snapshots(root) {
            return (false, error);
        }
    }
    let output = Command::new(env!("CARGO_BIN_EXE_boundary-check"))
        .arg("--root")
        .arg(root)
        .arg("--config")
        .arg("tools/boundary-check/config/road1.toml")
        .output()
        .expect("run boundary-check");
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    (output.status.success(), format!("{stdout}{stderr}"))
}

pub fn seed_snapshots(root: &Path) -> Result<(), String> {
    let seed = Command::new(env!("CARGO_BIN_EXE_boundary-check"))
        .arg("--root")
        .arg(root)
        .arg("--config")
        .arg("tools/boundary-check/config/road1.toml")
        .arg("--update-snapshots")
        .output()
        .expect("seed Phase 6 snapshots");
    if seed.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Phase 6 snapshot seed failed:\n{}{}",
            String::from_utf8_lossy(&seed.stdout),
            String::from_utf8_lossy(&seed.stderr)
        ))
    }
}

pub fn write_subworkspace_crate(
    root: &Path,
    workspace_rel: &str,
    constitutional_lane: &str,
    allowed_prefix: &str,
    crate_name: &str,
    package_name: &str,
    deps_toml: &str,
) {
    write_file(
        root,
        &format!("{workspace_rel}/Cargo.toml"),
        &format!(
            r#"[workspace]
resolver = "2"
members = ["crates/{crate_name}"]

[workspace.metadata.worth_topology]
role = "road1_subworkspace"
constitutional_lane = "{constitutional_lane}"
member_lane = "crates/*"
allowed_crate_prefixes = ["{allowed_prefix}"]
"#
        ),
    );
    write_file(
        root,
        &format!("{workspace_rel}/README.md"),
        &format!("# {constitutional_lane}\n\nRoad 1 subworkspace charter for tests.\n"),
    );
    write_file(
        root,
        &format!("{workspace_rel}/crates/{crate_name}/Cargo.toml"),
        &format!(
            r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
{deps_toml}
"#
        ),
    );
    write_file(
        root,
        &format!("{workspace_rel}/crates/{crate_name}/src/lib.rs"),
        "mod test_surface;\npub mod facade;\n",
    );
    write_file(
        root,
        &format!("{workspace_rel}/crates/{crate_name}/src/test_surface.rs"),
        "pub fn seed() {}\n",
    );
    write_file(
        root,
        &format!("{workspace_rel}/crates/{crate_name}/src/facade.rs"),
        "// Phase 6 fixture facade: intentionally no exports.\n",
    );
}

/// A documented engine whose items the audience leaf facades re-export, so
/// the facade documentation ratchet resolves every leaf name.
const STUB_ENGINE: &str = r#"//! Stub Query engine.

/// Stub engine facade.
pub mod facade {
    /// Stub foundation vocabulary.
    pub mod foundation {
        /// Stub canonical artifact.
        pub struct CanonicalQueryArtifact;
        /// Stub replay basis.
        pub struct ScopedReplayBasis;
    }
    /// Stub domain namespace.
    pub mod domain {}
    /// Stub runtime namespace.
    pub mod runtime {}
}
"#;

pub fn write_query_stubs(root: &Path) {
    // Vendor stubs resolve Cargo metadata for governed Road 1 packages.
    write_stub_lib(root, "vendor/worth-query", "worth-query");
    write_stub_lib(root, "vendor/worth-query-decl", "worth-query-decl");
    write_stub_lib(root, "vendor/worth-query-host", "worth-query-host");
    write_stub_lib(root, "vendor/worth-query-replay", "worth-query-replay");

    // Root framework leaves must also exist under crates/ for production
    // audience-facade contract validation (engine-only dep + re-export shape).
    write_stub_lib(root, "crates/worth-query", "worth-query");
    write_file(root, "crates/worth-query/src/lib.rs", STUB_ENGINE);
    write_leaf_facade(
        root,
        "worth-query-decl",
        "pub use worth_query::facade::foundation::CanonicalQueryArtifact;\n",
    );
    write_leaf_facade(
        root,
        "worth-query-host",
        "pub use worth_query::facade::domain;\npub use worth_query::facade::runtime;\n",
    );
    write_leaf_facade(
        root,
        "worth-query-replay",
        "pub use worth_query::facade::foundation::ScopedReplayBasis;\n",
    );
}

fn write_leaf_facade(root: &Path, package: &str, facade_body: &str) {
    write_file(
        root,
        &format!("crates/{package}/Cargo.toml"),
        &format!(
            r#"[package]
name = "{package}"
version = "0.1.0"
edition = "2021"

[dependencies]
worth-query = {{ path = "../worth-query" }}

[workspace]
"#
        ),
    );
    write_file(
        root,
        &format!("crates/{package}/src/lib.rs"),
        "pub mod facade;\n",
    );
    write_file(
        root,
        &format!("crates/{package}/src/facade.rs"),
        facade_body,
    );
}
