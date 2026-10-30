#[allow(unused_imports)]
use crate::s6_secure_io_authority_compile_fail::*;
use std::path::Path;
use worth_store_test_support::compiler_boundary::UiCompilerEnvironment;

use worth_store_test_support::compiler_boundary::read_ui_fixture_suite;

#[test]
fn io_qos_secure_io_authority_rejects_lower_authority_sources() {
    let root = store_workspace_root();
    let evidence = read_ui_fixture_suite(root, "secure-io-authority", UiCompilerEnvironment::Production, &root.join(
            "crates/worth-store-court-maintenance-recovery-runtime-replication/tests/worth_store_court_maintenance_recovery_runtime_replication/sources/worth_store_certification/compile_fail/scheduling/secure_io_authority",
        ), FIXTURES)
    .unwrap();
    assert_eq!(evidence.fixtures.len(), FIXTURES.len());
}
