#[allow(unused_imports)]
use crate::terminal_projection_quarantine_ui::*;
use std::path::Path;
use worth_store_test_support::compiler_boundary::UiCompilerEnvironment;

use worth_store_test_support::compiler_boundary::read_ui_fixture_suite;

#[test]
fn terminal_projection_quarantine_denies_neutral_public_callers() {
    let root = store_workspace_root();
    let evidence = read_ui_fixture_suite(root, "terminal-projection-quarantine", UiCompilerEnvironment::Production, &root.join("crates/worth-store-court-maintenance-recovery-runtime-replication/tests/worth_store_court_maintenance_recovery_runtime_replication/sources/worth_store_certification/ui/terminal_projection_quarantine"), FIXTURES)
    .unwrap();
    assert_eq!(evidence.fixtures.len(), FIXTURES.len());
}
