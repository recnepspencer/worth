use super::*;

#[test]
fn ordinary_consumers_cannot_reference_the_five_sealed_names() {
    let source = "macro_rules! forge { () => { registered_authoritative_source(); with_authoritative_source(); from_owner_publication(); from_owner_partition_publication(); r#certification_query(); } }";
    let path = "workspaces/worth-query/crates/worth-query-host/src/ordinary.rs";
    let diagnostics = source_references(path, source);
    assert_eq!(diagnostics.len(), 5);
    for name in BRIDGE.iter().copied().chain([LOOKUP]) {
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message().contains(&format!("`{name}`"))));
    }
    for diagnostic in diagnostics {
        assert_eq!(diagnostic.subject(), path);
        assert!(diagnostic.message().contains(path));
    }
}

#[test]
fn recorded_certification_callers_and_owners_keep_the_construction_doors() {
    assert!(source_references("workspaces/worth-query/crates/worth-query-certification/examples/product_workflow_support/conditional_contribution.rs", "fn f() { certification_query(); }").is_empty());
    assert!(source_references("workspaces/worth-query/crates/worth-query/src/domain_installation/dependency_impact/compiled/invalidation_manifest/structural_manifest_tests.rs", "fn f() { registered_authoritative_source(); }").is_empty());
    for line in CALLERS.lines().filter(|line| !line.starts_with('#')) {
        let Some((class, path)) = line.split_once(' ') else {
            continue;
        };
        let name = if class == "bridge" { BRIDGE[0] } else { LOOKUP };
        assert!(
            source_references(path, &format!("fn f() {{ {name}(); }}")).is_empty(),
            "{path}"
        );
    }
    assert!(source_references(
        "crates/worth-runtime-bridge/src/input/envelope/core.rs",
        "fn registered_authoritative_source() {}"
    )
    .is_empty());
    assert!(source_references(
        "workspaces/worth-query/crates/worth-query-installation/src/lib.rs",
        "pub fn certification_query() {}"
    )
    .is_empty());
}

#[test]
fn documentation_does_not_create_a_constructor_reference() {
    assert!(source_references(
        "crates/ordinary/src/lib.rs",
        "// certification_query\nconst NOTE: &str = \"registered_authoritative_source\";"
    )
    .is_empty());
}
