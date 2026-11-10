use worth_foundational::{FoundationalMergeIntent, FoundationalMergeScopeFamily};
use worth_proof::TransitionOutcome;

use super::lower_merge_request_to_foundational;
use crate::facade::history::BranchId;
use crate::facade::merge::{
    MergeIntent, NormalizedRelationalMergeRequest, RelationalMergeCorrespondencePosture,
    RelationalMergeRequestFamily, RelationalMergeSchemaReconciliationPosture, RelationalMergeScope,
    RelationalMergeTopologyIntent,
};

#[test]
fn foundational_lowering_preserves_exact_relational_request_truth() {
    let normalized = normalized_request("main", "feature");

    let lowered = lower_merge_request_to_foundational(normalized.clone());
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
    let main_to_feature = normalized_request("main", "feature");
    let release_to_hotfix = normalized_request("release", "hotfix");

    let lowered_main_to_feature = lower_merge_request_to_foundational(main_to_feature);
    let lowered_release_to_hotfix = lower_merge_request_to_foundational(release_to_hotfix);
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
    let normalized = normalized_request("main", "feature");

    let lowered = lower_merge_request_to_foundational(normalized);

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
