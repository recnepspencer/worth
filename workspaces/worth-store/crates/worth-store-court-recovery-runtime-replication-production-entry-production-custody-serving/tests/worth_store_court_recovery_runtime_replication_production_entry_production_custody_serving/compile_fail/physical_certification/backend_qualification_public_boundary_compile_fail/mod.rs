#[allow(unused_imports)]
use crate::backend_qualification_public_boundary_compile_fail::*;
use std::path::Path;
use worth_store_test_support::compiler_boundary::UiCompilerEnvironment;

use worth_store_test_support::compiler_boundary::read_ui_fixture_suite;

#[test]
fn copied_rows_and_proof_authority_cannot_enter_public_publication_boundary() {
    let root = store_workspace_root();
    let evidence = read_ui_fixture_suite(root, "backend-qualification-public-boundary", UiCompilerEnvironment::Certification, &root.join("crates/worth-store-court-recovery-runtime-replication-production-entry-production-custody-serving/tests/worth_store_court_recovery_runtime_replication_production_entry_production_custody_serving/sources/worth_store_physical_certification/ui/qualification/backend_public_boundary"), FIXTURES).unwrap();
    assert_eq!(evidence.fixtures.len(), FIXTURES.len());
}
