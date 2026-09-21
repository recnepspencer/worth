#[test]
fn bank_schema_source_has_no_raw_query_descriptor_or_dynamic_key_lane() {
    let schema_sources = [
        include_str!("../../src/schema/entities.rs"),
        include_str!("../../src/schema/authentication.rs"),
        include_str!("../../src/schema/decision_read_manifest.rs"),
        include_str!("../../src/schema/fields.rs"),
        include_str!("../../src/schema/governance.rs"),
        include_str!("../../src/schema/manifest.rs"),
        include_str!("../../src/schema/operations.rs"),
        include_str!("../../src/schema/program_manifest.rs"),
        include_str!("../../src/schema/relations.rs"),
        include_str!("../../src/schema/values.rs"),
    ]
    .join("\n");
    for forbidden in [
        "from_schema_identifier(",
        "from_schema_identifiers(",
        "ApplicationEntityRef::<",
        "ApplicationFieldRef::<",
        "DynamicApplication",
    ] {
        assert!(
            !schema_sources.contains(forbidden),
            "bank schema contains forbidden raw lane: {forbidden}"
        );
    }

    let manifest = include_str!("../../Cargo.toml");
    for forbidden_dependency in [
        "worth-query-declaration",
        "worth-query-installation",
        "worth-query-execution",
        "worth-query-replay",
        "worth-runtime-bridge",
        "worth-relational",
    ] {
        assert!(
            !manifest.contains(forbidden_dependency),
            "bank-domain crosses audience boundary through {forbidden_dependency}"
        );
    }
}
