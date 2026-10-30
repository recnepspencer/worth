#[allow(unused_imports)]
use crate::s5_1_security_scope_admission_compile_fail::*;
use std::path::Path;
use worth_store_test_support::compiler_boundary::UiCompilerEnvironment;

use worth_store_test_support::compiler_boundary::read_ui_fixture_suite;

#[test]
fn an_admission_receipt_cannot_substitute_for_current_scope_authority() {
    let root = store_workspace_root();
    let evidence = read_ui_fixture_suite(root, "security-scope-admission", UiCompilerEnvironment::Production, &root.join(
            "crates/worth-store-court-maintenance-recovery-runtime-replication/tests/worth_store_court_maintenance_recovery_runtime_replication/sources/worth_store_certification/compile_fail/security/security_scope_admission",
        ), &[(
            "admission_receipt_cannot_satisfy_key_witness.rs",
            &[
                "StoreCurrentKeyScopeWitness",
                "StoreSecurityScopeAdmissionReceipt",
            ],
        )])
    .unwrap();
    assert_eq!(evidence.fixtures.len(), 1);
}
