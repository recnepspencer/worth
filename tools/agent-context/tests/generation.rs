use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEST_ROOT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn test_root() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let sequence = TEST_ROOT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "agent-context-test-{}-{unique}-{sequence}",
        std::process::id()
    ));
    copy_dir(
        &workspace_root().join("tools/boundary-check/config"),
        &root.join("tools/boundary-check/config"),
    );
    copy_dir(
        &workspace_root().join("tools/boundary-check/snapshots"),
        &root.join("tools/boundary-check/snapshots"),
    );
    copy_dir(
        &workspace_root().join("workspaces/worth-contracts/crates/worth-schema-core"),
        &root.join("workspaces/worth-contracts/crates/worth-schema-core"),
    );
    // Framework Query packages are configured in road1.toml and live in their
    // dedicated workspace for orientation generation.
    for package in ["worth-query-decl", "worth-query-host", "worth-query-replay"] {
        copy_dir(
            &workspace_root()
                .join("workspaces/worth-query/crates")
                .join(package),
            &root.join("workspaces/worth-query/crates").join(package),
        );
    }
    fs::create_dir_all(root.join("workspaces/worth-query/crates/worth-query-certification"))
        .expect("create Query certification fixture root");
    let app_root = root.join("workspaces/worth-ui/apps/platform-pulse");
    let crate_root = root.join("workspaces/worth-ui/crates/worth-ui-runtime");
    fs::create_dir_all(app_root.join("src")).expect("create Worth UI application fixture root");
    fs::create_dir_all(crate_root.join("src")).expect("create Worth UI crate fixture root");
    fs::write(
        root.join("workspaces/worth-ui/Cargo.toml"),
        "[workspace]\nmembers = [\"apps/platform-pulse\", \"crates/worth-ui-runtime\"]\n",
    )
    .expect("write Worth UI workspace fixture");
    fs::write(
        app_root.join("Cargo.toml"),
        "[package]\nname = \"worth-ui-platform-pulse\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write Worth UI application manifest fixture");
    fs::write(app_root.join("src/main.rs"), "fn main() {}\n")
        .expect("write Worth UI application source fixture");
    fs::write(app_root.join("src/application.rs"), "\n")
        .expect("write Worth UI application module fixture");
    fs::create_dir_all(app_root.join("src/application"))
        .expect("write Worth UI application submodule fixture");
    fs::write(
        crate_root.join("Cargo.toml"),
        "[package]\nname = \"worth-ui-runtime\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write Worth UI crate manifest fixture");
    fs::write(crate_root.join("src/lib.rs"), "\n").expect("write Worth UI crate source fixture");
    root
}

#[test]
fn facade_exports_are_rendered_from_snapshot() {
    let root = test_root();
    let snapshot = root.join("tools/boundary-check/snapshots/facades.toml");
    let text = fs::read_to_string(&snapshot)
        .expect("read facade snapshot")
        .replace("CanonicalQueryArtifact", "SnapshotOwnedExport");
    fs::write(&snapshot, text).expect("write divergent snapshot");
    let generate = run_tool(&root, "generate");
    assert!(
        generate.status.success(),
        "{}",
        String::from_utf8_lossy(&generate.stderr)
    );
    let context = fs::read_to_string(
        root.join("workspaces/worth-query/crates/worth-query-decl/AGENT_CONTEXT.md"),
    )
    .expect("read context");
    assert!(context.contains("SnapshotOwnedExport"));
    assert!(!context.contains("CanonicalQueryArtifact"));
}

#[test]
fn missing_facade_snapshot_row_fails_closed() {
    let root = test_root();
    let snapshot = root.join("tools/boundary-check/snapshots/facades.toml");
    let text = fs::read_to_string(&snapshot).expect("read facade snapshot");
    let package = text
        .find("package = \"worth-query-decl\"")
        .expect("decl package");
    let start = text[..package].rfind("[[facades]]").expect("decl row");
    let end = text[package..]
        .find("[[facades]]")
        .map(|offset| package + offset)
        .unwrap_or(text.len());
    fs::write(&snapshot, format!("{}{}", &text[..start], &text[end..])).expect("remove row");
    let output = run_tool(&root, "generate");
    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("facades.toml") && error.contains("worth-query-decl"));
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("tools directory")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn copy_dir(source: &Path, target: &Path) {
    fs::create_dir_all(target).expect("create target directory");
    for entry in fs::read_dir(source).expect("read source directory") {
        let entry = entry.expect("read source entry");
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir(&source_path, &target_path);
        } else {
            fs::create_dir_all(target_path.parent().expect("target parent"))
                .expect("create parent");
            fs::copy(&source_path, &target_path).expect("copy file");
        }
    }
}

fn run_tool(root: &Path, mode: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_agent-context"))
        .arg(mode)
        .arg("--root")
        .arg(root)
        .arg("--config")
        .arg("tools/boundary-check/config/road1.toml")
        .output()
        .expect("run agent-context")
}

#[test]
fn generation_is_stable_and_check_passes() {
    let root = test_root();
    let generate = run_tool(&root, "generate");
    assert!(
        generate.status.success(),
        "{}",
        String::from_utf8_lossy(&generate.stderr)
    );

    let schema_path =
        root.join("workspaces/worth-contracts/crates/worth-schema-core/AGENT_CONTEXT.md");
    let first = fs::read_to_string(&schema_path).expect("read generated context");
    assert!(
        first.contains("Canonical machine constitution: `tools/boundary-check/config/road1.toml`")
    );
    assert!(first.contains("Constitutional class: `worth/schema`"));
    assert!(first.contains(
        "Road 1 exemplar role: Road 1 foundational identity / naming / tolerance specimen"
    ));
    assert!(first.contains(
        "`worth-entry-adoption` -> Query-native declaration/adoption facade (Milestone 3)"
    ));
    assert!(first.contains("Public surface: facade-only"));
    let app_context =
        fs::read_to_string(root.join("workspaces/worth-ui/apps/platform-pulse/AGENT_CONTEXT.md"))
            .expect("read generated application context");
    assert_eq!(
        app_context.lines().next(),
        Some("# worth-ui-platform-pulse")
    );
    assert!(app_context.contains("Must not depend on worthy-* crates."));
    assert!(app_context.contains(
        "Public surface: workspace-owned; package targets remain the explicit export or composition owners"
    ));
    let owned_modules = app_context
        .lines()
        .find_map(|line| line.strip_prefix("- Owned internal modules: `"))
        .and_then(|line| line.strip_suffix('`'))
        .expect("owned application modules");
    let modules = owned_modules.split(", ").collect::<Vec<_>>();
    let unique_modules = modules
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(modules.len(), unique_modules.len());
    assert_eq!(
        modules
            .iter()
            .filter(|module| **module == "application")
            .count(),
        1
    );
    let runtime_context = fs::read_to_string(
        root.join("workspaces/worth-ui/crates/worth-ui-runtime/AGENT_CONTEXT.md"),
    )
    .expect("read generated crate context");
    assert_eq!(runtime_context.lines().next(), Some("# worth-ui-runtime"));
    assert!(runtime_context
        .contains("Replay dependencies are admitted only for configured certification packages"));

    let generate_again = run_tool(&root, "generate");
    assert!(
        generate_again.status.success(),
        "{}",
        String::from_utf8_lossy(&generate_again.stderr)
    );
    let second = fs::read_to_string(&schema_path).expect("read regenerated context");
    assert_eq!(first, second);

    let check = run_tool(&root, "check");
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
}

#[test]
fn stale_hand_edit_is_rejected() {
    let root = test_root();
    let generate = run_tool(&root, "generate");
    assert!(
        generate.status.success(),
        "{}",
        String::from_utf8_lossy(&generate.stderr)
    );

    let schema_path =
        root.join("workspaces/worth-contracts/crates/worth-schema-core/AGENT_CONTEXT.md");
    fs::write(&schema_path, "tampered\n").expect("overwrite generated context");

    let check = run_tool(&root, "check");
    assert!(!check.status.success(), "stale context unexpectedly passed");
    assert!(String::from_utf8_lossy(&check.stderr).contains("stale or hand-edited"));
}

#[test]
fn stale_hand_edited_agent_context_is_rejected() {
    stale_hand_edit_is_rejected();
}
