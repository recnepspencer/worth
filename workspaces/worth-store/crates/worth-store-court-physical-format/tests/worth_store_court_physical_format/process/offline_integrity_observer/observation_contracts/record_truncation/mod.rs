#[allow(unused_imports)]
use crate::record_truncation::*;
use crate::support::checksum::refresh_crc32c;

#[test]
fn binary_bootstrap_alias_is_not_reinterpreted_as_another_family() {
    let fixture = clean_store("bootstrap-alias");
    let baseline = observe(&fixture.store);
    std::fs::hard_link(
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
        std::fs::write(fixture.store.join(ROUTING), &clean[..prefix]).unwrap();
        let truncated = observe(&fixture.store);
        assert_damage(
            row(&truncated, ROUTING),
            "truncation",
            prefix as u64,
            176 - prefix as u64,
            serde_json::Value::Null,
            "artifact",
        );
    }

    let mut lied = clean.clone();
    lied[24..28].copy_from_slice(&129_u32.to_le_bytes());
    refresh_crc32c(&mut lied);
    std::fs::write(fixture.store.join(ROUTING), lied).unwrap();
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
    std::fs::write(fixture.store.join(BOOTSTRAP), &bytes).unwrap();
    let control = observe(&fixture.store);
    assert_eq!(row(&control, BOOTSTRAP)["outcome"]["posture"], "intact");
    for prefix in [0, 47, 81] {
        std::fs::write(fixture.store.join(BOOTSTRAP), &bytes[..prefix]).unwrap();
        let report = observe(&fixture.store);
        let artifact = row(&report, BOOTSTRAP);
        assert_eq!(artifact["range"], json!({"offset":0,"length":82}));
        assert_damage(
            artifact,
            "truncation",
            prefix as u64,
            82 - prefix as u64,
            serde_json::Value::Null,
            "artifact",
        );
    }
    std::fs::remove_file(fixture.store.join(BOOTSTRAP)).unwrap();
    let report = observe(&fixture.store);
    let artifact = row(&report, BOOTSTRAP);
    assert_eq!(artifact["range"], json!({"offset":0,"length":82}));
    assert_eq!(artifact["outcome"]["cause"], "missing_artifact");
}
