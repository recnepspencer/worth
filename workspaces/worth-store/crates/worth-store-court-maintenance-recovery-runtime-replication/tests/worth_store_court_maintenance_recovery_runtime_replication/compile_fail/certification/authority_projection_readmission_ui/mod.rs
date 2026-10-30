#[allow(unused_imports)]
use crate::authority_projection_readmission_ui::*;
use worth_store_test_support::compiler_boundary::UiCompilerEnvironment;

use worth_store_test_support::compiler_boundary::read_ui_fixture_suite;

#[test]
fn authority_projection_readmission_denies_lower_authority_inputs() {
    let root = store_workspace_root();
    let evidence = read_ui_fixture_suite(root, "authority-projection-readmission", UiCompilerEnvironment::Production, &root.join("crates/worth-store-court-maintenance-recovery-runtime-replication/tests/worth_store_court_maintenance_recovery_runtime_replication/sources/worth_store_certification/ui/authority_projection_readmission"), FIXTURES)
    .unwrap();
    assert_eq!(evidence.fixtures.len(), FIXTURES.len());
}
