use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

fn fixture_root(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn run_fixture(name: &str) -> String {
    let root = copied_fixture(name);
    let output = Command::new(env!("CARGO_BIN_EXE_boundary-check"))
        .arg("--root")
        .arg(&root)
        .arg("--config")
        .arg("tools/boundary-check/config/road1.toml")
        .output()
        .expect("run boundary-check fixture");
    let _ = fs::remove_dir_all(root);
    assert!(
        !output.status.success(),
        "fixture {name} unexpectedly passed"
    );
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn copied_fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "boundary-rule-{name}-{}-{}",
        std::process::id(),
        FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&root);
    copy_tree(&fixture_root(name), &root);
    let snapshots = root.join("tools/boundary-check/snapshots");
    fs::create_dir_all(&snapshots).unwrap();
    fs::write(
        snapshots.join("crate-dag.toml"),
        "schema_version = 1\npackages = []\n",
    )
    .unwrap();
    fs::write(
        snapshots.join("facades.toml"),
        "schema_version = 1\nfacades = []\n",
    )
    .unwrap();
    root
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = if entry.file_name() == "Cargo.fixture.toml" {
            destination.join("Cargo.toml")
        } else {
            destination.join(entry.file_name())
        };
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

#[test]
fn illegal_crate_name_is_rejected() {
    let output = run_fixture("illegal_crate_name");
    assert!(output.contains("BC1002_UNRESERVED_DOMAIN"));
}

#[test]
fn schema_query_import_is_rejected() {
    let output = run_fixture("schema_query_import");
    assert!(output.contains("BC3001_DIRECT_QUERY_ENGINE"));
}

#[test]
fn json_diagnostics_name_a_non_empty_legal_home() {
    let root = copied_fixture("schema_query_import");
    let output = Command::new(env!("CARGO_BIN_EXE_boundary-check"))
        .arg("--root")
        .arg(&root)
        .args([
            "--config",
            "tools/boundary-check/config/road1.toml",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(root);
    assert!(!output.status.success());
    let diagnostics: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    for diagnostic in diagnostics.as_array().unwrap() {
        assert!(!diagnostic["legal_home"].as_str().unwrap().trim().is_empty());
    }
}

#[test]
fn query_bridge_module_in_schema_is_rejected() {
    let output = run_fixture("schema_query_import");
    assert!(output.contains("BC3001_DIRECT_QUERY_ENGINE"));
}

#[test]
fn ordinary_replay_import_is_rejected() {
    let output = run_fixture("ordinary_replay_import");
    assert!(output.contains("BC4001_ORDINARY_REPLAY_IMPORT"));
}

#[test]
fn ordinary_reconstruction_import_is_rejected() {
    let output = run_fixture("ordinary_reconstruction_import");
    assert!(output.contains("BC4001_ORDINARY_REPLAY_IMPORT"));
}

#[test]
fn worth_to_worthy_inversion_is_rejected() {
    let output = run_fixture("worth_to_worthy_inversion");
    assert!(output.contains("BC2002_WORTH_TO_WORTHY_INVERSION"));
}

#[test]
fn root_owned_road1_package_is_rejected() {
    let output = run_fixture("root_owned_road1_package");
    assert!(output.contains("BC5001_ROOT_OWNS_ROAD1_PACKAGE"));
}

#[test]
fn schema_pack_import_is_rejected() {
    let output = run_fixture("schema_pack_import");
    assert!(output.contains("BC2001_BAND_DEPENDENCY_VIOLATION"));
}

#[test]
fn runtime_adapter_in_pack_registry_is_rejected() {
    let output = run_fixture("runtime_adapter_in_pack_registry");
    assert!(output.contains("BC2001_BAND_DEPENDENCY_VIOLATION"));
}

#[test]
fn placeholder_entry_birth_is_rejected() {
    let output = run_fixture("placeholder_entry_birth");
    assert!(output.contains("BC5003_SEED_CONTRACT_VIOLATION"));
    assert!(output.contains("born crate set mismatch"));
}

#[test]
fn facade_behavior_is_rejected() {
    let output = run_fixture("facade_behavior_seed");
    assert!(output.contains("BC5003_SEED_CONTRACT_VIOLATION"));
    assert!(output.contains("facade.rs must aggregate public exports only"));
}

#[test]
fn mixed_class_seed_module_is_rejected() {
    let output = run_fixture("mixed_class_seed_module");
    assert!(output.contains("BC5003_SEED_CONTRACT_VIOLATION"));
    assert!(output.contains("seed crate skeleton mismatch"));
}
