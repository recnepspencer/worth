#[test]
fn every_checked_in_pulse_status_value_is_qualified_printable_basic_latin() {
    let root = super::workspace_source_inventory()
        .root()
        .join("apps/platform-pulse/query_samples");
    let mut observed = 0;
    for entry in std::fs::read_dir(root).expect("Pulse query samples") {
        let path = entry.expect("query sample entry").path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).expect("query sample bytes"))
                .expect("query sample JSON");
        let status = value["status"].as_str().expect("status string");
        assert!(!status.is_empty());
        assert!(
            status
                .chars()
                .all(|character| ('\u{20}'..='\u{7e}').contains(&character)),
            "{} leaves qualified Basic Latin: {status:?}",
            path.display()
        );
        observed += 1;
    }
    assert_eq!(
        observed, 3,
        "every checked-in Pulse status sample is audited"
    );
}
