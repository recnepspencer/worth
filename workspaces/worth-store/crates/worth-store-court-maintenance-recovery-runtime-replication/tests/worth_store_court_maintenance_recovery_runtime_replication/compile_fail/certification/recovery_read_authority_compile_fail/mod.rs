#[allow(unused_imports)]
use crate::recovery_read_authority_compile_fail::*;

use worth_store_test_support::compiler_boundary::read_ui_proof_suite;

#[test]
fn recovery_reports_cannot_reenter_physical_read_authority() {
    let root = store_workspace_root();
    let evidence = read_ui_proof_suite(root, &suite(root)).unwrap();
    assert_eq!(evidence.fixtures.len(), fixture_denials().len());
    assert!(evidence
        .fixtures
        .iter()
        .all(|fixture| fixture.semantic_denial_matched));
}
