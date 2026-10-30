#[allow(unused_imports)]
use crate::harness_authority_compile_fail::*;
use worth_store_test_support::compiler_boundary::UiCompilerEnvironment;

use worth_store_test_support::compiler_boundary::read_ui_fixture_suite;

#[test]
fn courtroom_test_authority_cannot_satisfy_production_surfaces() {
    let root = store_workspace_root();
    let source = root.join("crates/worth-store-court-recovery-runtime-replication-production-entry-production-custody-serving/tests/worth_store_court_recovery_runtime_replication_production_entry_production_custody_serving/sources/worth_store_test_support/ui/harness_authority");
    let physical = read_ui_fixture_suite(
        root,
        "test-support-harness-physical-authority",
        UiCompilerEnvironment::Certification,
        &source,
        PHYSICAL_FIXTURES,
    )
    .unwrap();
    let production = read_ui_fixture_suite(
        root,
        "test-support-harness-production-authority",
        UiCompilerEnvironment::Production,
        &source,
        PRODUCTION_FIXTURES,
    )
    .unwrap();
    assert_eq!(
        physical.fixtures.len() + production.fixtures.len(),
        PHYSICAL_FIXTURES.len() + PRODUCTION_FIXTURES.len()
    );
}
