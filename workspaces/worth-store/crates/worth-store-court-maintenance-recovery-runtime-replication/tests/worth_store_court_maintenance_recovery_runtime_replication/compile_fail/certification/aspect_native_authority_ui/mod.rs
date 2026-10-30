#[allow(unused_imports)]
use crate::aspect_native_authority_ui::*;
use std::path::Path;
use worth_store_test_support::compiler_boundary::UiCompilerEnvironment;

use worth_store_test_support::compiler_boundary::read_ui_fixture_suite;

#[test]
fn aspect_native_authority_denies_raw_public_callers() {
    let root = store_workspace_root();
    let forge_root = root.ancestors().nth(2).unwrap();
    let evidence = read_ui_fixture_suite(root, "aspect-native-authority", UiCompilerEnvironment::Production, &root.join("crates/worth-store-court-maintenance-recovery-runtime-replication/tests/worth_store_court_maintenance_recovery_runtime_replication/sources/worth_store_certification/ui/aspect_native_authority"), FIXTURES)
    .unwrap();
    assert_eq!(evidence.fixtures.len(), FIXTURES.len());
}
