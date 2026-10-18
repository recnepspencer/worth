//! Complete binding scopes cannot hide an unbound installed portable target.
use super::*;

#[test]
fn extra_valid_operation_node_refuses_before_initialization() {
    refusal::<{ super::super::ORPHAN_OPERATION }>("unbound-output-ready");
}

#[test]
fn same_named_unbound_workflow_stage_node_refuses_before_initialization() {
    refusal::<{ super::super::ORPHAN_STAGE }>("orphan-stage");
}

fn refusal<const MODE: u8>(target: &str) {
    use worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationDenial;
    let _guard = checkpoint_recovery_test_guard();
    let initialized = std::cell::Cell::new(false);
    let result =
        support::try_install_program_with_limits::<OracleProgram<false, TOTALS_WORK, 1, MODE>>(
            None,
            Default::default(),
            support::limits(32, support::invalidation(128 * 1024 * 1024, 1_000_000, 32)),
            |_| initialized.set(true),
        );
    assert!(
        !initialized.get(),
        "complete portable coverage is checked before initialization"
    );
    match *result.err().expect("a valid unbound target prevents installation") {
        WorthQueryInMemoryApplicationDenial::Contributions(denial) => {
            assert_eq!(denial.kind(), primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::MissingConditionalBinding);
            assert!(denial.subject().contains("conditional target coverage"), "{denial:?}");
            assert!(denial.subject().contains(target), "full target location: {denial:?}");
        }
        other => panic!("expected installed target-coverage refusal, not an earlier malformed package/admission: {other:?}"),
    }
}
