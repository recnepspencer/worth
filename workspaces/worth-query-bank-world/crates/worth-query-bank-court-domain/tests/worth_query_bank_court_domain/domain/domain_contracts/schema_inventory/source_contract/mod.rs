#[test]
fn bank_schema_source_has_no_raw_query_descriptor_or_dynamic_key_lane() {
    let schema_sources = [
        include_str!("../../../../../../../bank-domain/src/schema/entities.rs"),
        include_str!("../../../../../../../bank-domain/src/schema/entities/records.rs"),
        include_str!("../../../../../../../bank-domain/src/schema/entities/records/rosters.rs"),
        include_str!("../../../../../../../bank-domain/src/schema/entities/records/bindings.rs"),
        include_str!(
            "../../../../../../../bank-domain/src/schema/entities/records/entity_members.rs"
        ),
        include_str!(
            "../../../../../../../bank-domain/src/schema/entities/records/account_rows.rs"
        ),
        include_str!(
            "../../../../../../../bank-domain/src/schema/entities/records/payment_rows.rs"
        ),
        include_str!("../../../../../../../bank-domain/src/schema/entities/records/estate_rows.rs"),
        include_str!(
            "../../../../../../../bank-domain/src/schema/entities/records/capability_rows.rs"
        ),
        include_str!(
            "../../../../../../../bank-domain/src/schema/entities/records/principal_rows.rs"
        ),
        include_str!("../../../../../../../bank-domain/src/schema/entities/records/selections.rs"),
        include_str!(
            "../../../../../../../bank-domain/src/schema/entities/records/account_relations.rs"
        ),
        include_str!(
            "../../../../../../../bank-domain/src/schema/entities/records/payment_relations.rs"
        ),
        include_str!(
            "../../../../../../../bank-domain/src/schema/entities/records/estate_relations.rs"
        ),
        include_str!("../../../../../../../bank-domain/src/schema/authentication.rs"),
        include_str!("../../../../../../../bank-domain/src/schema/decision_read_manifest.rs"),
        include_str!("../../../../../../../bank-domain/src/schema/fields.rs"),
        include_str!("../../../../../../../bank-domain/src/schema/governance.rs"),
        include_str!("../../../../../../../bank-domain/src/schema/manifest.rs"),
        include_str!("../../../../../../../bank-domain/src/schema/operations.rs"),
        include_str!("../../../../../../../bank-domain/src/schema/program_manifest.rs"),
        include_str!("../../../../../../../bank-domain/src/schema/relations.rs"),
        include_str!("../../../../../../../bank-domain/src/schema/values.rs"),
    ]
    .join("\n");
    for forbidden in [
        "from_schema_identifier(",
        "from_schema_identifiers(",
        "ApplicationEntityRef::<",
        "AspectIdentity(",
        "AspectContractRevision(",
        "DynamicApplication",
    ] {
        assert!(
            !schema_sources.contains(forbidden),
            "bank schema contains forbidden raw lane: {forbidden}"
        );
    }

    let manifest = include_str!("../../../../../../../bank-domain/Cargo.toml");
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
