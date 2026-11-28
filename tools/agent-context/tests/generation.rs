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
    fs::create_dir_all(root.join("scripts/ci")).expect("create owner map directory");
    fs::write(
        root.join("scripts/ci/relational_dag_owners.toml"),
        "[owners]\n",
    )
    .expect("write owner map for fixture without Relational crates");
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
    let certification = "workspaces/worth-query/crates/worth-query-certification/src/facade.rs";
    fs::create_dir_all(root.join(certification).parent().unwrap())
        .expect("create certification source directory");
    fs::copy(
        workspace_root().join(certification),
        root.join(certification),
    )
    .expect("copy real certification facade");
    root
}

// The approved boundary triage retires facade snapshots; orientation follows source.
#[test]
fn facade_exports_follow_source_declarations() {
    let root = test_root();
    let source = root.join("workspaces/worth-query/crates/worth-query-decl/src/facade.rs");
    let text = fs::read_to_string(&source).unwrap();
    fs::write(&source, format!("{text}\npub use worth_query_declaration::facade::canonicalization::CanonicalQueryArtifact as SourceOwnedExport;\n")).unwrap();
    let generate = run_tool(&root, "generate");
    assert!(
        generate.status.success(),
        "{}",
        String::from_utf8_lossy(&generate.stderr)
    );
    let context = fs::read_to_string(
        root.join("workspaces/worth-query/crates/worth-query-decl/AGENT_CONTEXT.md"),
    )
    .unwrap();
    assert!(context.contains("SourceOwnedExport"));
}

#[test]
fn missing_facade_source_fails_closed() {
    let root = test_root();
    fs::remove_file(root.join("workspaces/worth-query/crates/worth-query-decl/src/facade.rs"))
        .unwrap();
    let output = run_tool(&root, "generate");
    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("facade source") && error.contains("worth-query-decl"));
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
