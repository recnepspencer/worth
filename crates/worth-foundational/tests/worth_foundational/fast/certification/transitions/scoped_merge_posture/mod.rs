use std::fmt::Debug;
use worth_foundational::{
    FoundationalMergeAdmissionDenial, FoundationalMergeConstructionDenial, FoundationalMergeScope,
    FoundationalScopedMergeUnavailableOutcomeCategory, FoundationalScopedMergeUnavailablePosture,
    FoundationalScopedMergeUnavailableReason,
};
use worth_proof::TransitionOutcome;

use super::fixtures::branch::branch_id;
use super::fixtures::scoped_merge::{scoped_candidate, selected_aspect, selected_node};

#[test]
fn scoped_unavailable_reasons_map_to_their_required_proof_categories() {
    assert_deferred_unavailable(
        FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope"),
        FoundationalScopedMergeUnavailableReason::RuntimeDoesNotSupportSelectedNodes,
    );
    assert_deferred_unavailable(
        FoundationalMergeScope::selected_aspects([selected_aspect("gear", "teeth")])
            .expect("aspect scope"),
        FoundationalScopedMergeUnavailableReason::RuntimeDoesNotSupportSelectedAspects,
    );
    assert_failed_unavailable(
        FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope"),
        FoundationalScopedMergeUnavailableReason::MaterializerUnavailable,
    );
    assert_rebind_unavailable(
        FoundationalMergeScope::selected_aspects([selected_aspect("gear", "teeth")])
            .expect("aspect scope"),
        FoundationalScopedMergeUnavailableReason::IdentityCorrespondenceUnavailable,
    );
    assert_stale_unavailable(
        FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope"),
        FoundationalScopedMergeUnavailableReason::RetainedProofUnavailable,
    );
}

#[test]
fn scoped_unavailable_constructor_denies_reason_scope_mismatch() {
    let mismatch = FoundationalScopedMergeUnavailablePosture::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        FoundationalMergeScope::selected_aspects([selected_aspect("gear", "teeth")])
            .expect("aspect scope"),
        FoundationalScopedMergeUnavailableReason::RuntimeDoesNotSupportSelectedNodes,
    )
    .expect_err("selected-node support reason cannot describe selected-aspect scope");

    assert_eq!(
        mismatch,
        FoundationalMergeConstructionDenial::ScopedUnavailableReasonScopeMismatch
    );
}

#[test]
fn scoped_unavailable_constructor_denies_correspondence_unavailable_without_aspect_scope() {
    let full_branch_mismatch = FoundationalScopedMergeUnavailablePosture::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        FoundationalMergeScope::full_branch(),
        FoundationalScopedMergeUnavailableReason::IdentityCorrespondenceUnavailable,
    )
    .expect_err("full-branch scope has no selected-aspect correspondence locus");
    let selected_node_mismatch = FoundationalScopedMergeUnavailablePosture::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope"),
        FoundationalScopedMergeUnavailableReason::IdentityCorrespondenceUnavailable,
    )
    .expect_err("selected-node scope has no selected-aspect correspondence locus");

    assert_eq!(
        full_branch_mismatch,
        FoundationalMergeConstructionDenial::ScopedUnavailableReasonScopeMismatch
    );
    assert_eq!(
        selected_node_mismatch,
        FoundationalMergeConstructionDenial::ScopedUnavailableReasonScopeMismatch
    );
}

#[test]
fn scoped_unavailable_admission_denies_mismatched_scope_or_branch_basis() {
    let candidate_scope =
        FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope");
    let different_scope =
        FoundationalMergeScope::selected_nodes([selected_node("material")]).expect("node scope");
    let scope_mismatch = scoped_unavailable(
        different_scope,
        FoundationalScopedMergeUnavailableReason::RuntimeDoesNotSupportSelectedNodes,
    );
    let source_mismatch = FoundationalScopedMergeUnavailablePosture::new(
        branch_id("feature/other"),
        branch_id("main"),
        candidate_scope.clone(),
        FoundationalScopedMergeUnavailableReason::RuntimeDoesNotSupportSelectedNodes,
    )
    .expect("source mismatch posture");
    let target_mismatch = FoundationalScopedMergeUnavailablePosture::new(
        branch_id("feature/geometry"),
        branch_id("release"),
        candidate_scope.clone(),
        FoundationalScopedMergeUnavailableReason::RuntimeDoesNotSupportSelectedNodes,
    )
    .expect("target mismatch posture");

    assert_admission_denial(
        scoped_candidate(candidate_scope.clone()).scope_unavailable(scope_mismatch),
        FoundationalMergeAdmissionDenial::ScopedEvidenceScopeMismatch,
    );
    assert_admission_denial(
        scoped_candidate(candidate_scope.clone()).scope_unavailable(source_mismatch),
        FoundationalMergeAdmissionDenial::ScopedEvidenceSourceBranchMismatch,
    );
    assert_admission_denial(
        scoped_candidate(candidate_scope).scope_unavailable(target_mismatch),
        FoundationalMergeAdmissionDenial::ScopedEvidenceTargetBranchMismatch,
    );
}

#[test]
fn scoped_unavailable_is_not_invalid_scope_denial_or_admitted_scope_evidence() {
    let scope =
        FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope");
    let outcome = scoped_candidate(scope.clone()).scope_unavailable(scoped_unavailable(
        scope,
        FoundationalScopedMergeUnavailableReason::RuntimeDoesNotSupportSelectedNodes,
    ));

    match outcome {
        TransitionOutcome::Deferred(deferred) => {
            let posture = deferred
                .scope_unavailable_posture()
                .expect("unavailable posture");
            assert_eq!(
                posture.outcome_category(),
                FoundationalScopedMergeUnavailableOutcomeCategory::Deferred
            );
            assert_eq!(
                posture.reason(),
                FoundationalScopedMergeUnavailableReason::RuntimeDoesNotSupportSelectedNodes
            );
        }
        TransitionOutcome::Success(verdict) => {
            panic!("unavailable scope must not become admitted scope: {verdict:?}")
        }
        TransitionOutcome::Denied(denial) => {
            panic!("unavailable scope must not become invalid-scope denial: {denial:?}")
        }
        other => panic!("expected deferred scoped unavailable posture, got {other:?}"),
    }
}

fn assert_deferred_unavailable(
    scope: FoundationalMergeScope,
    reason: FoundationalScopedMergeUnavailableReason,
) {
    match scoped_candidate(scope.clone()).scope_unavailable(scoped_unavailable(scope, reason)) {
        TransitionOutcome::Deferred(deferred) => assert_posture(
            deferred.scope_unavailable_posture(),
            reason,
            FoundationalScopedMergeUnavailableOutcomeCategory::Deferred,
        ),
        other => panic!("expected deferred unavailable posture, got {other:?}"),
    }
}

fn assert_stale_unavailable(
    scope: FoundationalMergeScope,
    reason: FoundationalScopedMergeUnavailableReason,
) {
    match scoped_candidate(scope.clone()).scope_unavailable(scoped_unavailable(scope, reason)) {
        TransitionOutcome::Stale(stale) => assert_posture(
            stale.scope_unavailable_posture(),
            reason,
            FoundationalScopedMergeUnavailableOutcomeCategory::Stale,
        ),
        other => panic!("expected stale unavailable posture, got {other:?}"),
    }
}

fn assert_rebind_unavailable(
    scope: FoundationalMergeScope,
    reason: FoundationalScopedMergeUnavailableReason,
) {
    match scoped_candidate(scope.clone()).scope_unavailable(scoped_unavailable(scope, reason)) {
        TransitionOutcome::RebindRequired(rebind) => assert_posture(
            rebind.scope_unavailable_posture(),
            reason,
            FoundationalScopedMergeUnavailableOutcomeCategory::RebindRequired,
        ),
        other => panic!("expected rebind unavailable posture, got {other:?}"),
    }
}

fn assert_failed_unavailable(
    scope: FoundationalMergeScope,
    reason: FoundationalScopedMergeUnavailableReason,
) {
    match scoped_candidate(scope.clone()).scope_unavailable(scoped_unavailable(scope, reason)) {
        TransitionOutcome::Failed(failure) => assert_posture(
            failure.scope_unavailable_posture(),
            reason,
            FoundationalScopedMergeUnavailableOutcomeCategory::Failed,
        ),
        other => panic!("expected failed unavailable posture, got {other:?}"),
    }
}

fn assert_posture(
    posture: Option<&FoundationalScopedMergeUnavailablePosture>,
    reason: FoundationalScopedMergeUnavailableReason,
    outcome_category: FoundationalScopedMergeUnavailableOutcomeCategory,
) {
    let posture = posture.expect("scope unavailable posture");
    assert_eq!(posture.source_branch(), &branch_id("feature/geometry"));
    assert_eq!(posture.target_branch(), &branch_id("main"));
    assert_eq!(posture.reason(), reason);
    assert_eq!(posture.outcome_category(), outcome_category);
}

fn assert_admission_denial<T: Debug>(
    actual: worth_foundational::FoundationalMergeAdmissionOutcome<T>,
    expected: FoundationalMergeAdmissionDenial,
) {
    match actual {
        TransitionOutcome::Denied(denial) => assert_eq!(denial, expected),
        other => panic!("expected scoped unavailable admission denial, got {other:?}"),
    }
}

fn scoped_unavailable(
    requested_scope: FoundationalMergeScope,
    reason: FoundationalScopedMergeUnavailableReason,
) -> FoundationalScopedMergeUnavailablePosture {
    FoundationalScopedMergeUnavailablePosture::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        requested_scope,
        reason,
    )
    .expect("scoped unavailable posture")
}
