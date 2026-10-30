pub(crate) use worth_query_bank_testkit_estate_certification::facade::residue_certification::*;

#[test]
fn certification_uses_only_the_entry_audience_and_foundational_surface() {
    let manifest = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(manifest.contains("worth-query-decl.workspace = true"));
    assert!(manifest.contains("worth-query-host.workspace = true"));
    for dependency in FORBIDDEN_DEPENDENCIES {
        assert!(
            !manifest.contains(dependency),
            "forbidden dependency: {dependency}"
        );
    }
}

#[test]
fn production_bank_source_contains_no_superseded_authority_or_oracle_lane() {
    let bank_world = manifest_dir().join("..").join("..");
    for source in [
        bank_world.join("crates").join("bank-domain").join("src"),
        bank_world.join("crates").join("bank-server").join("src"),
        bank_world
            .join("crates")
            .join("bank-http-adapter")
            .join("src"),
        bank_world.join("crates").join("bank-user-node").join("src"),
    ] {
        inspect_rust_sources(&source);
    }
}

#[test]
fn dependency_gate_rejects_a_renamed_forbidden_package_identity() {
    let metadata = serde_json::json!({
        "packages": [{
            "name": "bank-server",
            "dependencies": [{
                "name": "worth-query-execution",
                "rename": "query_core",
                "kind": null
            }]
        }]
    });
    assert_eq!(
        production_dependency_violations(&metadata),
        ["bank-server -> query_core (worth-query-execution)"]
    );
}
