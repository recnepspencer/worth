//! Named temporary repository for production-path rename-ratchet proofs.

use super::retired_tokens::{retired_hyphen_fragment, retired_underscore_fragment};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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

/// Isolated temporary repository exercising the production boundary-check binary.
pub struct LegacyReferenceTestRepository {
    root: PathBuf,
}

impl LegacyReferenceTestRepository {
    /// Create a unique empty root for one scenario. Caller must [`cleanup`].
    pub fn create(label: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "boundary-check-legacy-{label}-{}-{nanos}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        Self { root }
    }

    pub fn path(&self) -> &Path {
        &self.root
    }

    /// Assemble the canonical thin-orchestrator layout, config, and snapshot body.
    pub fn assemble_canonical_layout(&self, snapshot_body: &str) {
        self.write_file(
            ".claude/settings.json",
            r#"{"hooks":{"SessionStart":[{"hooks":[{"type":"command","command":"powershell -NoProfile -File scripts/prepare-constitution-hook.ps1"}]}],"PostToolUse":[{"matcher":"Write|Edit|MultiEdit|apply_patch|Bash","hooks":[{"type":"command","command":"powershell -NoProfile -File scripts/check-constitution-post-tool-use.ps1"}]}]}}"#,
        );
        self.write_file("cad/docs/worthy-foundations/NAMING.md", "# naming\n");
        fs::create_dir_all(self.root.join("cad/workspaces")).expect("cad/workspaces");
        self.write_file(
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
        self.write_file(
            "tools/boundary-check/Cargo.toml",
            r#"[package]
name = "boundary-check"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]
"#,
        );
        self.write_file(
            "tools/boundary-check/snapshots/legacy-references.toml",
            snapshot_body,
        );
        let fragments = vec![retired_underscore_fragment(), retired_hyphen_fragment()];
        self.write_file(
            "tools/boundary-check/config/road1.toml",
            &render_minimal_config(
                "tools/boundary-check/snapshots/legacy-references.toml",
                &fragments,
            ),
        );
        self.write_query_audience_leaf_facades();
    }

    /// Minimal production-shaped Query audience leaves for BC3003 contract checks.
    fn write_query_audience_leaf_facades(&self) {
        self.write_file(
            "crates/worth-query/Cargo.toml",
            r#"[package]
name = "worth-query"
version = "0.1.0"
edition = "2021"
[workspace]
"#,
        );
        self.write_file("crates/worth-query/src/lib.rs", STUB_ENGINE);
        for (package, export) in [
            (
                "worth-query-decl",
                "pub use worth_query::facade::foundation::CanonicalQueryArtifact;\n",
            ),
            (
                "worth-query-host",
                "pub use worth_query::facade::domain;\npub use worth_query::facade::runtime;\n",
            ),
            (
                "worth-query-replay",
                "pub use worth_query::facade::foundation::ScopedReplayBasis;\n",
            ),
        ] {
            self.write_file(
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
            self.write_file(&format!("crates/{package}/src/lib.rs"), "pub mod facade;\n");
            self.write_file(&format!("crates/{package}/src/facade.rs"), export);
        }
    }

    pub fn write_file(&self, relative: &str, contents: &str) {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent directories");
        }
        fs::write(path, contents).expect("write file");
    }

    /// Run the compiled production binary against this repository root.
    pub fn run_boundary_check(&self) -> (bool, String) {
        let seed = Command::new(env!("CARGO_BIN_EXE_boundary-check"))
            .arg("--root")
            .arg(&self.root)
            .arg("--update-snapshots")
            .output()
            .expect("seed Phase 6 snapshots");
        if !seed.status.success() {
            return (
                false,
                format!(
                    "snapshot seed failed:\n{}{}",
                    String::from_utf8_lossy(&seed.stdout),
                    String::from_utf8_lossy(&seed.stderr)
                ),
            );
        }
        let output = Command::new(env!("CARGO_BIN_EXE_boundary-check"))
            .arg("--root")
            .arg(&self.root)
            .arg("--config")
            .arg("tools/boundary-check/config/road1.toml")
            .output()
            .expect("run boundary-check");
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        (output.status.success(), format!("{stdout}{stderr}"))
    }

    pub fn git_init_commit_all(&self) {
        self.git_run(&["init"]);
        self.git_run(&["config", "user.email", "ratchet@test.local"]);
        self.git_run(&["config", "user.name", "Ratchet Test"]);
        self.git_run(&["add", "."]);
        self.git_run(&["commit", "-m", "prior baseline"]);
    }

    pub fn git_commit_all(&self, message: &str) {
        self.git_run(&["add", "."]);
        self.git_run(&["commit", "-m", message]);
    }

    /// Create a governed symlink/junction under this root (fail-closed production path).
    pub fn create_governed_link(&self, target: &Path, link: &Path) {
        if let Some(parent) = link.parent() {
            fs::create_dir_all(parent).expect("link parent");
        }
        #[cfg(windows)]
        {
            // Directory junctions do not require Administrator rights.
            // mklink treats `/` as switch separators — force Windows path form.
            let link_s = link.to_string_lossy().replace('/', "\\");
            let target_s = target.to_string_lossy().replace('/', "\\");
            let status = Command::new("cmd")
                .args(["/C", "mklink", "/J", &link_s, &target_s])
                .status()
                .expect("run mklink");
            assert!(status.success(), "mklink /J failed for {link_s}");
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target, link).expect("create unix symlink");
        }
    }

    pub fn cleanup(self) {
        let _ = fs::remove_dir_all(&self.root);
    }

    fn git_run(&self, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(&self.root)
            .status()
            .unwrap_or_else(|error| panic!("git {args:?}: {error}"));
        assert!(status.success(), "git {args:?} failed");
    }
}

fn render_minimal_config(snapshot_relative: &str, fragments: &[String]) -> String {
    let fragment_list = fragments
        .iter()
        .map(|fragment| format!("  \"{fragment}\","))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"root_manifest = "Cargo.toml"
forbidden_root_prefixes = ["cad/workspaces/"]
born_crates = []
seed_skeletons = []
subworkspaces = []

[machine_authority]
canonical_config = "tools/boundary-check/config/road1.toml"
mirrored_docs = ["cad/docs/worthy-foundations/NAMING.md"]

[naming]
bands = ["schema"]

[[naming.reserved_domains]]
tier = "worth"
band = "schema"
domains = ["core"]

[[law_substrates]]
package = "worth-proof"
tiers = ["worth", "worthy"]
bands = ["schema"]

[rule_contracts]

[rule_contracts.query_audience]
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

[[rule_contracts.replay_surfaces]]
label = "certification replay"
package_prefixes = ["worth-cert-replay"]
cert_domains = ["replay"]

[[rule_contracts.band_rules]]
source_band = "schema"
allowed_target_bands = []

[legacy_reference_ratchet]
governed_roots = [
  "cad/workspaces/",
  "tools/",
  "crates/worth-proof/",
]
forbidden_fragments = [
{fragment_list}
]
snapshot = "{snapshot_relative}"
exclude_paths = [
  "tools/boundary-check/config/road1.toml",
  "{snapshot_relative}",
]
replacement_guidance = "Use the corresponding worth_/worth- spelling instead of the retired name."
"#
    )
}
