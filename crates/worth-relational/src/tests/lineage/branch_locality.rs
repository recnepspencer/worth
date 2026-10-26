use crate::facade::history::BranchId;
use crate::facade::lineage::{LineageDivergenceRequest, LineageDivergenceTraversalBasis};
use crate::tests::support::*;

// CONTRACT: lineage_branch_locality
// LANES: success, determinism

#[test]
fn lineage_branch_divergence_is_queryable() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let _main = create_entity_outcome(execution, &runtime, "main");
    runtime
        .history_authority()
        .fork_branch_from(
            execution,
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let _feature = create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature",
        BranchId("feature".to_string()),
    );
    let divergence = runtime
        .lineage_access()
        .divergence_between_branches(
            execution,
            LineageDivergenceRequest {
                left_branch: BranchId("main".to_string()),
                right_branch: BranchId("feature".to_string()),
                traversal_basis: LineageDivergenceTraversalBasis::FullBranchGraphComparison,
            },
        )
        .unwrap();

    assert_eq!(
        divergence.traversal_basis,
        LineageDivergenceTraversalBasis::FullBranchGraphComparison
    );
    assert!(!divergence.right_only_event_ids.is_empty());
    assert!(!divergence.shared_lineage_ids.is_empty());
    assert_eq!(divergence.metrics.right_event_count, 1);
    assert!(divergence.metrics.shared_lineage_count >= 1);
}
