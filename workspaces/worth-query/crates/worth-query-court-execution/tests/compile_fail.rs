//! The real consumer fixture names its member's unmet schema requirements.
use std::path::Path;

#[test]
fn foreign_schema_member_reports_the_schema_requirement() {
    let consumer = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../worth-query-certification/fixtures/consumer_entry");
    worth_testkit_compiler::production_semantic_denial(
        consumer.join("hostile_foreign_schema/src/main.rs"),
        "E0277",
        &[
            "TopologyContribution",
            "cannot be declared in schema `ConsumerSchema`",
            "schema requirements",
        ],
    )
    .unwrap();
    for source in [
        "topology_entry/src/lib.rs",
        "parameter_entry/src/lib.rs",
        "consumer_root/src/main.rs",
    ] {
        worth_testkit_compiler::compiled(consumer.join(source)).unwrap();
    }
}

#[test]
fn recovery_authority_minting_doors_are_absent() {
    use worth_query_execution::facade::domain_computation::WorthQueryRecoveryEffectAuthority;
    use worth_query_execution_runtime::facade::managed_run::WorthQueryRecoveryHandleRegistry;

    // The production types and operations resolve from an external crate.
    let _ = WorthQueryRecoveryEffectAuthority::ensure_for;
    let _ = WorthQueryRecoveryHandleRegistry::runtime_authority;

    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/recovery_authority/effect_authority_mint.rs");
    tests.compile_fail("tests/ui/recovery_authority/registry_constructor.rs");
}
