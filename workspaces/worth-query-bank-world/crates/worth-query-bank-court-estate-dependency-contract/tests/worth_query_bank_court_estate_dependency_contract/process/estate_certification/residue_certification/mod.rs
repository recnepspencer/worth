pub(crate) use worth_query_bank_testkit_estate_certification::facade::residue_certification::*;

#[test]
fn production_bank_packages_use_only_query_audience_crates() {
    let bank_world = manifest_dir().join("..").join("..");
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--manifest-path",
        ])
        .arg(bank_world.join("Cargo.toml"))
        .output()
        .expect("cargo metadata must run");
    assert!(output.status.success(), "cargo metadata failed");
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        production_dependency_violations(&metadata),
        Vec::<String>::new()
    );
    let admitted = production_query_audience_dependencies(&metadata);
    assert!(admitted.contains("worth-query-decl"));
    assert!(admitted.contains("worth-query-host"));
}
