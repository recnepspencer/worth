#[allow(unused_imports)]
use crate::certification::ordinary_context::transitions::scoped_merge_posture::*;
use std::fmt::Debug;
use worth_proof::TransitionOutcome;
use {
    worth_foundational::FoundationalMergeAdmissionDenial,
    worth_foundational::FoundationalMergeConstructionDenial,
    worth_foundational::FoundationalMergeScope,
    worth_foundational::FoundationalScopedMergeUnavailableOutcomeCategory,
    worth_foundational::FoundationalScopedMergeUnavailablePosture,
    worth_foundational::FoundationalScopedMergeUnavailableReason,
};

use crate::certification::ordinary_context::transitions::fixtures::branch::branch_id;
use {
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::scoped_candidate,
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::selected_aspect,
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::selected_node,
};

#[test]
fn scoped_unavailable_reasons_map_to_their_required_proof_categories() {
    assert_deferred_unavailable(
        worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope"),
        worth_foundational::FoundationalScopedMergeUnavailableReason::RuntimeDoesNotSupportSelectedNodes,
    );
    assert_deferred_unavailable(
        worth_foundational::FoundationalMergeScope::selected_aspects([selected_aspect("gear", "teeth")])
            .expect("aspect scope"),
        worth_foundational::FoundationalScopedMergeUnavailableReason::RuntimeDoesNotSupportSelectedAspects,
    );
    assert_failed_unavailable(
        worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("gear")])
            .expect("node scope"),
        worth_foundational::FoundationalScopedMergeUnavailableReason::MaterializerUnavailable,
    );
    assert_rebind_unavailable(
        worth_foundational::FoundationalMergeScope::selected_aspects([selected_aspect("gear", "teeth")])
            .expect("aspect scope"),
        worth_foundational::FoundationalScopedMergeUnavailableReason::IdentityCorrespondenceUnavailable,
    );
    assert_stale_unavailable(
        worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("gear")])
            .expect("node scope"),
        worth_foundational::FoundationalScopedMergeUnavailableReason::RetainedProofUnavailable,
    );
}

#[test]
fn scoped_unavailable_constructor_denies_reason_scope_mismatch() {
    let mismatch = worth_foundational::FoundationalScopedMergeUnavailablePosture::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        worth_foundational::FoundationalMergeScope::selected_aspects([selected_aspect("gear", "teeth")])
            .expect("aspect scope"),
        worth_foundational::FoundationalScopedMergeUnavailableReason::RuntimeDoesNotSupportSelectedNodes,
    )
    .expect_err("selected-node support reason cannot describe selected-aspect scope");

    assert_eq!(
        mismatch,
        worth_foundational::FoundationalMergeConstructionDenial::ScopedUnavailableReasonScopeMismatch
    );
}

#[test]
fn scoped_unavailable_constructor_denies_correspondence_unavailable_without_aspect_scope() {
    let full_branch_mismatch = worth_foundational::FoundationalScopedMergeUnavailablePosture::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        worth_foundational::FoundationalMergeScope::full_branch(),
        worth_foundational::FoundationalScopedMergeUnavailableReason::IdentityCorrespondenceUnavailable,
    )
    .expect_err("full-branch scope has no selected-aspect correspondence locus");
    let selected_node_mismatch = worth_foundational::FoundationalScopedMergeUnavailablePosture::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope"),
        worth_foundational::FoundationalScopedMergeUnavailableReason::IdentityCorrespondenceUnavailable,
    )
    .expect_err("selected-node scope has no selected-aspect correspondence locus");

    assert_eq!(
        full_branch_mismatch,
        worth_foundational::FoundationalMergeConstructionDenial::ScopedUnavailableReasonScopeMismatch
    );
    assert_eq!(
        selected_node_mismatch,
        worth_foundational::FoundationalMergeConstructionDenial::ScopedUnavailableReasonScopeMismatch
    );
}

#[test]
fn scoped_unavailable_admission_denies_mismatched_scope_or_branch_basis() {
    let candidate_scope =
        worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("gear")])
            .expect("node scope");
    let different_scope =
        worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("material")])
            .expect("node scope");
    let scope_mismatch = scoped_unavailable(
        different_scope,
        worth_foundational::FoundationalScopedMergeUnavailableReason::RuntimeDoesNotSupportSelectedNodes,
    );
    let source_mismatch = worth_foundational::FoundationalScopedMergeUnavailablePosture::new(
        branch_id("feature/other"),
        branch_id("main"),
        candidate_scope.clone(),
        worth_foundational::FoundationalScopedMergeUnavailableReason::RuntimeDoesNotSupportSelectedNodes,
    )
    .expect("source mismatch posture");
    let target_mismatch = worth_foundational::FoundationalScopedMergeUnavailablePosture::new(
        branch_id("feature/geometry"),
        branch_id("release"),
        candidate_scope.clone(),
        worth_foundational::FoundationalScopedMergeUnavailableReason::RuntimeDoesNotSupportSelectedNodes,
    )
    .expect("target mismatch posture");

    assert_admission_denial(
        scoped_candidate(candidate_scope.clone()).scope_unavailable(scope_mismatch),
        worth_foundational::FoundationalMergeAdmissionDenial::ScopedEvidenceScopeMismatch,
    );
    assert_admission_denial(
        scoped_candidate(candidate_scope.clone()).scope_unavailable(source_mismatch),
        worth_foundational::FoundationalMergeAdmissionDenial::ScopedEvidenceSourceBranchMismatch,
    );
    assert_admission_denial(
        scoped_candidate(candidate_scope).scope_unavailable(target_mismatch),
        worth_foundational::FoundationalMergeAdmissionDenial::ScopedEvidenceTargetBranchMismatch,
    );
}

#[test]
fn scoped_unavailable_is_not_invalid_scope_denial_or_admitted_scope_evidence() {
    let scope = worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("gear")])
        .expect("node scope");
    let outcome = scoped_candidate(scope.clone()).scope_unavailable(scoped_unavailable(
        scope,
        worth_foundational::FoundationalScopedMergeUnavailableReason::RuntimeDoesNotSupportSelectedNodes,
    ));

    match outcome {
        worth_proof::TransitionOutcome::Deferred(deferred) => {
            let posture = deferred
                .scope_unavailable_posture()
                .expect("unavailable posture");
            assert_eq!(
                posture.outcome_category(),
                worth_foundational::FoundationalScopedMergeUnavailableOutcomeCategory::Deferred
            );
            assert_eq!(
                posture.reason(),
                worth_foundational::FoundationalScopedMergeUnavailableReason::RuntimeDoesNotSupportSelectedNodes
            );
        }
        worth_proof::TransitionOutcome::Success(verdict) => {
            panic!("unavailable scope must not become admitted scope: {verdict:?}")
        }
        worth_proof::TransitionOutcome::Denied(denial) => {
            panic!("unavailable scope must not become invalid-scope denial: {denial:?}")
        }
        other => panic!("expected deferred scoped unavailable posture, got {other:?}"),
    }
}
