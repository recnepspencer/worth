use worth_foundational::{FoundationalMergeIntent, FoundationalMergeScopeFamily};
use worth_proof::TransitionOutcome;

use crate::facade::history::BranchId;
use crate::facade::merge::{
    MergeExecutionRequest, MergeIntent, NormalizedRelationalMergeRequest,
    RelationalMergeCorrespondencePosture, RelationalMergeRequestFamily,
    RelationalMergeSchemaReconciliationPosture, RelationalMergeScope,
    RelationalMergeTopologyIntent,
};
use crate::tests::support::persisted_runtime_with_test_schema;

#[test]
fn foundational_lowering_preserves_exact_relational_request_truth() {
    let runtime = persisted_runtime_with_test_schema();
    let normalized = normalized_request("main", "feature");

    let lowered = runtime
        .merge()
        .lower_merge_request_to_foundational(normalized.clone());
    let TransitionOutcome::Success(lowered) = lowered else {
        panic!("expected foundational admission success");
    };

    assert_eq!(lowered.normalized_request(), &normalized);
    assert_eq!(
        lowered.normalized_request().request_digest(),
        normalized.request_digest()
    );
    assert_eq!(
        lowered.normalized_request().family(),
        RelationalMergeRequestFamily::FullBranchReconciliation
    );
    assert_eq!(
        lowered.normalized_request().scope(),
        RelationalMergeScope::FullBranch
    );
    assert_eq!(
        lowered.normalized_request().correspondence_posture(),
        RelationalMergeCorrespondencePosture::Advisory
    );
    assert_eq!(
        lowered.normalized_request().schema_reconciliation_posture(),
        RelationalMergeSchemaReconciliationPosture::Participate
    );
    assert_eq!(
        lowered.normalized_request().topology_intent(),
        RelationalMergeTopologyIntent::PreserveTopologySemantics
    );
    assert_eq!(
        lowered.foundational_scope_family(),
        FoundationalMergeScopeFamily::FullBranch
    );
    assert_eq!(
        lowered.foundational_intent(),
        FoundationalMergeIntent::ReconcileIntoTarget
    );
}

#[test]
fn foundational_lowering_does_not_collapse_distinct_relational_meanings() {
    let runtime = persisted_runtime_with_test_schema();
    let main_to_feature = normalized_request("main", "feature");
    let release_to_hotfix = normalized_request("release", "hotfix");

    let lowered_main_to_feature = runtime
        .merge()
        .lower_merge_request_to_foundational(main_to_feature);
    let lowered_release_to_hotfix = runtime
        .merge()
        .lower_merge_request_to_foundational(release_to_hotfix);
    let TransitionOutcome::Success(lowered_main_to_feature) = lowered_main_to_feature else {
        panic!("expected foundational admission success");
    };
    let TransitionOutcome::Success(lowered_release_to_hotfix) = lowered_release_to_hotfix else {
        panic!("expected foundational admission success");
    };

    assert_ne!(lowered_main_to_feature, lowered_release_to_hotfix);
    assert_ne!(
        lowered_main_to_feature.normalized_request(),
        lowered_release_to_hotfix.normalized_request()
    );
    assert_eq!(
        lowered_main_to_feature.foundational_scope_family(),
        lowered_release_to_hotfix.foundational_scope_family()
    );
    assert_eq!(
        lowered_main_to_feature.foundational_intent(),
        lowered_release_to_hotfix.foundational_intent()
    );
    assert_ne!(
        lowered_main_to_feature.lowering_digest(),
        lowered_release_to_hotfix.lowering_digest()
    );
}

#[test]
fn execution_authoring_and_explicit_normalization_lower_identically() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    crate::tests::support::create_entity_outcome(execution, &runtime, "merge-seed");
    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .expect("feature branch has an exact owner cell");
    let raw = MergeExecutionRequest::new(
        BranchId("main".to_string()),
        BranchId("feature".to_string()),
        MergeIntent::ReconcileIntoTarget,
    );
    let normalized_from_execution = runtime
        .merge()
        .normalize_merge_request(raw)
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

#[test]
fn foundational_lowering_uses_foundational_admission_vocabulary() {
    let runtime = persisted_runtime_with_test_schema();
    let normalized = normalized_request("main", "feature");

    let lowered = runtime
        .merge()
        .lower_merge_request_to_foundational(normalized);

    match lowered {
        TransitionOutcome::Success(lowered) => {
            assert_eq!(
                lowered.foundational_scope_family(),
                FoundationalMergeScopeFamily::FullBranch
            );
            assert_eq!(
                lowered.foundational_intent(),
                FoundationalMergeIntent::ReconcileIntoTarget
            );
        }
        TransitionOutcome::Denied(_)
        | TransitionOutcome::Deferred(_)
        | TransitionOutcome::Stale(_)
        | TransitionOutcome::RebindRequired(_)
        | TransitionOutcome::Failed(_) => {
            panic!("expected foundational success posture for admitted phase-3 request");
        }
    }
}
