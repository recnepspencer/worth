use super::branch_history_publications::*;
use worth_proof::TransitionOutcome;
use worth_relational::facade::{history::*, merge::*};
use worth_testkit_relational::{
    durable_medium::TestDurableMedium, durable_publication::durable_runtime,
};
#[test]
fn execution_authoring_and_explicit_normalization_lower_identically() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "merge-seed");
    fork(execution, &runtime, "feature");
    let raw = MergeExecutionRequest::new(
        BranchId("main".to_string()),
        BranchId("feature".to_string()),
        MergeIntent::ReconcileIntoTarget,
    );
    let normalized_from_execution = runtime
        .merge()
        .normalize_merge_request(runtime.bind_merge_execution_request(raw).unwrap())
        .expect("normalized execution request");
    let explicit = normalized_request("main", "feature");

    let lowered_from_execution = runtime
        .merge()
        .lower_merge_request_to_foundational(normalized_from_execution);
    let lowered_from_explicit = runtime
        .merge()
        .lower_merge_request_to_foundational(explicit);
    let TransitionOutcome::Success(lowered_from_execution) = lowered_from_execution else {
        panic!("expected foundational admission success");
    };
    let TransitionOutcome::Success(lowered_from_explicit) = lowered_from_explicit else {
        panic!("expected foundational admission success");
    };

    assert_eq!(lowered_from_execution, lowered_from_explicit);
    assert_eq!(
        lowered_from_execution.lowering_digest(),
        lowered_from_explicit.lowering_digest()
    );
}
fn normalized_request(
    target_branch: &str,
    source_branch: &str,
) -> NormalizedRelationalMergeRequest {
    NormalizedRelationalMergeRequest::admit_full_branch(
        BranchId(target_branch.to_string()),
        BranchId(source_branch.to_string()),
        MergeIntent::ReconcileIntoTarget,
        RelationalMergeCorrespondencePosture::Advisory,
        RelationalMergeSchemaReconciliationPosture::Participate,
        RelationalMergeTopologyIntent::PreserveTopologySemantics,
    )
    .expect("normalized request")
}
