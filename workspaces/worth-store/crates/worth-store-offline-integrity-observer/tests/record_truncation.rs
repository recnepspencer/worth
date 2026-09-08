use crate::phase_4_literal_vectors::{bootstrap_catalog, routing_block};
use crate::support::{checksum, clean_store, refresh_crc32c, StoreFixture};
use serde_json::{json, Value};
use std::{fs, path::Path, process::Command};

const ROUTING: &str =
    "families/records/roots/root-0000000000000001-block-0000000000000003.manifest";
const BOOTSTRAP: &str = "families/records/bootstrap.catalog";

#[test]
fn binary_bootstrap_alias_is_not_reinterpreted_as_another_family() {
    let fixture = clean_store("bootstrap-alias");
    let baseline = observe(&fixture.store);
    fs::hard_link(
        fixture.records.join("root-current.selector"),
        fixture.store.join(BOOTSTRAP),
    )
    .unwrap();
    let report = observe(&fixture.store);
    let artifact = row(&report, BOOTSTRAP);
    assert_eq!(artifact["range"], json!({"offset":0,"length":82}));
    assert_eq!(
        artifact["outcome"],
        json!({"posture":"unknown","reason":"physical_alias_not_reinspected"})
    );
    assert_eq!(
        artifact["duplicates"],
        json!([{"kind":"physical_alias","first_path":"families/records/root-current.selector"}])
    );
    for counter in ["bytes", "checksum_calculations", "durable_frame_decoders"] {
        assert_eq!(report["consumed"][counter], baseline["consumed"][counter]);
    }
}

#[test]
fn binary_distinguishes_short_tree_from_false_declared_length() {
    let fixture = clean_store("tree-length-localization");
    let clean = addressed_routing(&fixture);
    let control = observe(&fixture.store);
    assert_eq!(row(&control, ROUTING)["outcome"]["posture"], "intact");

    for prefix in [60, 175] {
        fs::write(fixture.store.join(ROUTING), &clean[..prefix]).unwrap();
        let truncated = observe(&fixture.store);
        assert_damage(
            row(&truncated, ROUTING),
            "truncation",
            prefix as u64,
            176 - prefix as u64,
            Value::Null,
            "artifact",
        );
    }

    let mut lied = clean.clone();
    lied[24..28].copy_from_slice(&129_u32.to_le_bytes());
    refresh_crc32c(&mut lied);
    fs::write(fixture.store.join(ROUTING), lied).unwrap();
    let framing = observe(&fixture.store);
    assert_damage(
        row(&framing, ROUTING),
        "framing",
        24,
        4,
        json!("payload_length"),
        "field",
    );
}

#[test]
fn binary_bootstrap_expected_range_survives_truncation_and_absence() {
    let fixture = clean_store("bootstrap-length-localization");
    let mut bytes = bootstrap_catalog();
    bytes[48..64].copy_from_slice(&(1_u8..=16).collect::<Vec<_>>());
    refresh_crc32c(&mut bytes);
    fs::write(fixture.store.join(BOOTSTRAP), &bytes).unwrap();
    let control = observe(&fixture.store);
    assert_eq!(row(&control, BOOTSTRAP)["outcome"]["posture"], "intact");
    for prefix in [0, 47, 81] {
        fs::write(fixture.store.join(BOOTSTRAP), &bytes[..prefix]).unwrap();
        let report = observe(&fixture.store);
        let artifact = row(&report, BOOTSTRAP);
        assert_eq!(artifact["range"], json!({"offset":0,"length":82}));
        assert_damage(
            artifact,
            "truncation",
            prefix as u64,
            82 - prefix as u64,
            Value::Null,
            "artifact",
        );
    }
    fs::remove_file(fixture.store.join(BOOTSTRAP)).unwrap();
    let report = observe(&fixture.store);
    let artifact = row(&report, BOOTSTRAP);
    assert_eq!(artifact["range"], json!({"offset":0,"length":82}));
    assert_eq!(artifact["outcome"]["cause"], "missing_artifact");
}

// Frozen byte grammar is an independent parser fixture, not a claim that this
// partial directory was produced by Store. The real binary discovers its child
// scope from the valid root reference; no expected verdict is passed to it.
fn addressed_routing(fixture: &StoreFixture) -> Vec<u8> {
    let mut tree = routing_block();
    tree[48..56].copy_from_slice(&7_u64.to_le_bytes());
    tree[72..80].copy_from_slice(&1_u64.to_le_bytes());
    refresh_crc32c(&mut tree);
    let path = fixture.roots.join("root-0000000000000001.manifest");
    let mut root = fs::read(&path).unwrap();
    root[72..80].copy_from_slice(&1_u64.to_le_bytes());
    root[80..88].copy_from_slice(&4_u64.to_le_bytes());
    root[88] = 1;
    root[96..104].copy_from_slice(&1_u64.to_le_bytes());
    root[104..112].copy_from_slice(&3_u64.to_le_bytes());
    root[116..120].copy_from_slice(&checksum(&[&tree]).to_le_bytes());
    root[120..144].copy_from_slice(&tree[88..112]);
    root[144..168].copy_from_slice(&tree[88..112]);
    refresh_crc32c(&mut root);
    fs::write(path, root).unwrap();
    fs::write(fixture.store.join(ROUTING), &tree).unwrap();
    tree
}

fn observe(root: &Path) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_physical_store_integrity_observer"))
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
            "5",
            "--max-depth",
            "8",
            "--max-symlinks",
            "0",
            "--max-elapsed-ms",
            "10000",
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

fn row<'a>(report: &'a Value, path: &str) -> &'a Value {
    report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["path"] == path)
        .unwrap()
}

fn assert_damage(row: &Value, cause: &str, offset: u64, length: u64, field: Value, blast: &str) {
    assert_eq!(
        row["outcome"],
        json!({
            "posture":"damaged", "cause":cause,
            "damaged_range":{"offset":offset,"length":length},
            "field":field, "blast_radius":blast,
        })
    );
}
