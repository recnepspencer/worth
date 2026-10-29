#[allow(unused_imports)] use crate::harness_authority_compile_fail::*;
use {std::path::Path};

use {worth_store_test_support::compiler_boundary::cargo_dependency_manifest, worth_store_test_support::compiler_boundary::run_cargo_ui_fixture_suite};

#[test]
fn courtroom_test_authority_cannot_satisfy_production_surfaces() {
    let root = store_workspace_root();
    let source = root.join("crates/worth-store-test-support/tests/ui/harness_authority");
    let physical = run_cargo_ui_fixture_suite(
        root,
        "test-support-harness-physical-authority",
        physical_environment(root),
        "physical-isolation-fixtures",
        "diagnostic-test",
        &source,
        PHYSICAL_FIXTURES,
    )
    .unwrap();
    let production = run_cargo_ui_fixture_suite(
        root,
        "test-support-harness-production-authority",
        production_environment(root),
        "production",
        "diagnostic-test",
        &source,
        PRODUCTION_FIXTURES,
    )
    .unwrap();
    assert_eq!(
        physical.fixtures.len() + production.fixtures.len(),
        PHYSICAL_FIXTURES.len() + PRODUCTION_FIXTURES.len()
    );
}
