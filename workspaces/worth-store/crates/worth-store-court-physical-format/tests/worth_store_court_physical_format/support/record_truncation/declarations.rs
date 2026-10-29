pub(crate) use {crate::phase_4_literal_vectors::bootstrap_catalog, crate::phase_4_literal_vectors::routing_block};
pub(crate) use {crate::support::fixtures::clean_store, crate::support::fixtures::StoreFixture};
pub(crate) use {serde_json::json, serde_json::Value};
pub(crate) use {std::fs, std::path::Path, std::process::Command};

pub(crate) const ROUTING: &str =
    "families/records/roots/root-0000000000000001-block-0000000000000003.manifest";
pub(crate) const BOOTSTRAP: &str = "families/records/bootstrap.catalog";

// Frozen byte grammar is an independent parser fixture, not a claim that this
// partial directory was produced by Store. The real binary discovers its child
// scope from the valid root reference; no expected verdict is passed to it.
pub(crate) fn addressed_routing(fixture: &StoreFixture) -> Vec<u8> {
    let mut tree = routing_block();
    tree[48..56].copy_from_slice(&7_u64.to_le_bytes());
    tree[72..80].copy_from_slice(&1_u64.to_le_bytes());
    refresh_crc32c(&mut tree);
    let path = fixture.roots.join("root-0000000000000001.manifest");
    let mut root = std::fs::read(&path).unwrap();
    root[72..80].copy_from_slice(&1_u64.to_le_bytes());
    root[80..88].copy_from_slice(&4_u64.to_le_bytes());
    root[88] = 1;
    root[96..104].copy_from_slice(&1_u64.to_le_bytes());
    root[104..112].copy_from_slice(&3_u64.to_le_bytes());
    root[116..120].copy_from_slice(&checksum(&[&tree]).to_le_bytes());
    root[120..144].copy_from_slice(&tree[88..112]);
    root[144..168].copy_from_slice(&tree[88..112]);
    refresh_crc32c(&mut root);
    std::fs::write(path, root).unwrap();
    std::fs::write(fixture.store.join(ROUTING), &tree).unwrap();
    tree
}

pub(crate) fn observe(root: &Path) -> Value {
    let output = std::process::Command::new(worth_store_testkit_executables::physical_store_integrity_observer())
        .args(["observe", "--store-root"])
        .arg(root)
        .args([
            "--report",
            "-",
            "--max-entries",
            "100",
            "--max-bytes",
            "16384",
            "--max-open-files",
            crate::store_offline_integrity_observer_support::FIXTURE_OPEN_FILE_ARGUMENT,
            "--max-depth",
            "8",
            "--max-symlinks",
            "0",
            "--max-elapsed-ms",
            crate::store_offline_integrity_observer_support::FIXTURE_ELAPSED_ARGUMENT,
            "--max-report-bytes",
            "65536",
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

pub(crate) fn row<'a>(report: &'a Value, path: &str) -> &'a Value {
    report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["path"] == path)
        .unwrap()
}

pub(crate) fn assert_damage(row: &Value, cause: &str, offset: u64, length: u64, field: Value, blast: &str) {
    assert_eq!(
        row["outcome"],
        json!({
            "posture":"damaged", "cause":cause,
            "damaged_range":{"offset":offset,"length":length},
            "field":field, "blast_radius":blast,
        })
    );
}