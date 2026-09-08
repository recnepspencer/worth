use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[test]
#[ignore = "requires Cargo-built independent observer; selected by C9 process runner"]
fn independent_observer_traverses_current_family_graph() {
    let observer = std::env::var_os(super::OBSERVER_EXECUTABLE_ENV).expect("observer executable");
    let world = tempfile::tempdir().unwrap();
    let root = world.path().join("store");
    super::production_store::produce_closed_store(
        &root,
        super::production_profile::ProductionWorldProfile::Primary16KiB,
    )
    .unwrap();
    let original = snapshot(&root);
    let report = observe(Path::new(&observer), &root);
    let artifacts = report["artifacts"].as_array().unwrap();
    let families = [
        "namespace_identity",
        "bootstrap_catalog",
        "current_root_selector",
        "previous_root_selector",
        "root_manifest",
        "root_routing_block",
        "segment_membership_block",
        "page_frame",
        "extent_manifest",
        "extent_chunk_frame",
        "free_space_header",
        "free_space_membership_block",
        "wal_frame",
        "checkpoint_stream_header",
        "checkpoint_binding_compaction",
        "checkpoint_binding",
        "checkpoint_footer",
    ];
    for family in families {
        let rows: Vec<_> = artifacts
            .iter()
            .filter(|row| row["family"] == family)
            .collect();
        assert!(
            !rows.is_empty(),
            "missing production family {family}: {report}"
        );
        assert!(
            rows.iter().any(|row| row["outcome"]["posture"] == "intact"),
            "{family}: {rows:?}"
        );
        for row in rows {
            let outcome = &row["outcome"];
            assert!(
                outcome["posture"] == "intact"
                    || (outcome["posture"] == "unknown"
                        && matches!(
                            outcome["reason"].as_str(),
                            Some("root_not_addressed" | "parent_scope_unavailable")
                        )),
                "unexpected production outcome: {row}"
            );
        }
    }
    assert_eq!(report["completeness"], "complete");
    assert_eq!(snapshot(&root), original);

    // Selected missing process-boundary family coverage. Every row starts from
    // independently retained clean bytes and changes only its checksum field.
    for family in [
        "segment_membership_block",
        "page_frame",
        "extent_manifest",
        "extent_chunk_frame",
        "free_space_header",
        "free_space_membership_block",
    ] {
        let expected = artifacts
            .iter()
            .find(|row| row["family"] == family && row["outcome"]["posture"] == "intact")
            .unwrap();
        let relative = expected["path"].as_str().unwrap();
        let offset = expected["range"]["offset"].as_u64().unwrap();
        let length = expected["range"]["length"].as_u64().unwrap();
        let path = root.join(relative);
        let clean = fs::read(&path).unwrap();
        let mut poisoned = clean.clone();
        poisoned[offset as usize + 44] ^= 1;
        fs::write(&path, &poisoned).unwrap();
        let before_observation = snapshot(&root);
        let damaged = observe(Path::new(&observer), &root);
        let row = damaged["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| {
                row["path"] == relative
                    && row["family"] == family
                    && row["range"]["offset"] == offset
            })
            .unwrap();
        assert_eq!(
            row["outcome"],
            json!({
                "posture":"damaged", "cause":"checksum_mismatch",
                "damaged_range":{"offset":offset,"length":length},
                "field":null, "blast_radius":"frame"
            }),
            "{family}"
        );
        assert_eq!(snapshot(&root), before_observation);
        fs::write(path, clean).unwrap();
    }
    assert_eq!(snapshot(&root), original);
}

fn observe(executable: &Path, root: &Path) -> Value {
    let output = Command::new(executable)
        .args(["observe", "--store-root"])
        .arg(root)
        .args([
            "--report",
            "-",
            "--max-entries",
            "1000",
            "--max-bytes",
            "16777216",
            "--max-open-files",
            "8",
            "--max-depth",
            "12",
            "--max-symlinks",
            "0",
            "--max-elapsed-ms",
            "30000",
            "--max-report-bytes",
            "1048576",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    let mut result = BTreeMap::new();
    let mut pending = vec![root.to_owned()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let kind = entry.file_type().unwrap();
            assert!(!kind.is_symlink());
            let bytes = if kind.is_dir() {
                pending.push(path.clone());
                None
            } else {
                Some(fs::read(&path).unwrap())
            };
            result.insert(path.strip_prefix(root).unwrap().to_owned(), bytes);
        }
    }
    result
}
