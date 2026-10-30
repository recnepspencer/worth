#[allow(unused_imports)]
use crate::certification::ordinary_context::transitions::scoped_merge_canonical::*;
use worth_proof::TransitionOutcome;
use {
    worth_foundational::compare_canonical_basis,
    worth_foundational::prepare_admitted_merge_scope_for_canonical_basis,
    worth_foundational::prepare_canonical_comparison,
    worth_foundational::prepare_merge_scope_for_canonical_basis,
    worth_foundational::prepare_merge_verdict_for_canonical_basis,
    worth_foundational::prepare_scoped_merge_denial_for_canonical_basis,
    worth_foundational::prepare_scoped_merge_unavailable_for_canonical_basis,
    worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisLocus,
    worth_foundational::CanonicalBasisReadyArtifact, worth_foundational::CanonicalBasisValue,
    worth_foundational::CanonicalComparisonOutcome, worth_foundational::CanonicalEquivalenceBasis,
    worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalizationRuleVersion,
    worth_foundational::FoundationalAdmittedMergeScopeEvidence,
    worth_foundational::FoundationalDeniedScopeLocus, worth_foundational::FoundationalMergeScope,
    worth_foundational::FoundationalScopeAdmissionBasis,
    worth_foundational::FoundationalScopedMergeDenialEvidence,
    worth_foundational::FoundationalScopedMergeDenialKind,
    worth_foundational::FoundationalScopedMergeUnavailablePosture,
    worth_foundational::FoundationalScopedMergeUnavailableReason,
    worth_foundational::FoundationalSelectedScopeLocus,
    worth_foundational::FoundationalSelectedScopeNoOpCause,
    worth_foundational::FoundationalSelectedScopeNoOpEvidence,
    worth_foundational::FoundationalSkippedOutOfScopeEvidence, worth_foundational::InternedString,
};

use crate::certification::ordinary_context::transitions::fixtures::branch::branch_id;
use {
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::scoped_candidate,
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::selected_aspect,
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::selected_node,
};

#[test]
fn selected_node_scope_canonical_basis_is_order_independent() {
    let left = ready_scope(
        &worth_foundational::FoundationalMergeScope::selected_nodes([
            selected_node("gear"),
            selected_node("material"),
        ])
        .expect("left scope"),
    );
    let right = ready_scope(
        &worth_foundational::FoundationalMergeScope::selected_nodes([
            selected_node("material"),
            selected_node("gear"),
        ])
        .expect("right scope"),
    );

    assert_equivalent(left, right);
}

#[test]
fn selected_aspect_scope_and_admitted_evidence_canonicalize_by_node_then_aspect() {
    let requested_scope = worth_foundational::FoundationalMergeScope::selected_aspects([
        selected_aspect("gear", "teeth"),
        selected_aspect("gear", "thickness"),
        selected_aspect("material", "finish"),
    ])
    .expect("scope");
    let left = admitted_scope_evidence(
        requested_scope.clone(),
        [
            selected_aspect("gear", "thickness"),
            selected_aspect("material", "finish"),
        ],
        [no_op_for_aspect("gear", "teeth")],
    );
    let right = admitted_scope_evidence(
        requested_scope,
        [
            selected_aspect("material", "finish"),
            selected_aspect("gear", "thickness"),
        ],
        [no_op_for_aspect("gear", "teeth")],
    );

    assert_equivalent(ready_admitted_scope(&left), ready_admitted_scope(&right));
}

#[test]
fn merge_verdict_canonical_basis_carries_admitted_scope_evidence() {
    let scope = worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("gear")])
        .expect("node scope");
    let evidence = worth_foundational::FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        scope.clone(),
        worth_foundational::FoundationalScopeAdmissionBasis::DirectSourceIdentity,
        [selected_node("gear")],
        [],
        [],
        worth_foundational::FoundationalSkippedOutOfScopeEvidence::new(2, None),
        2,
    )
    .expect("scope evidence");
    let verdict = match scoped_candidate(scope).admit_as_accepted_with_scope_evidence(evidence) {
        worth_proof::TransitionOutcome::Success(verdict) => verdict,
        other => panic!("expected scoped verdict, got {other:?}"),
    };
    let entries = ready_verdict(&verdict).payload().entries().to_vec();

    assert_contains_text(
        &entries,
        "merge.scope_evidence.requested_scope.family",
        "selected-nodes",
    );
    assert_contains_u64(
        &entries,
        "merge.scope_evidence.breadth.skipped_candidate_count",
        2,
    );
}

#[test]
fn scoped_denial_and_unavailable_posture_have_distinct_canonical_basis() {
    let scope = worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("gear")])
        .expect("node scope");
    let denial = worth_foundational::FoundationalScopedMergeDenialEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        scope.clone(),
        worth_foundational::FoundationalScopedMergeDenialKind::UnknownSelectedNode,
        worth_foundational::FoundationalDeniedScopeLocus::Node(selected_node("gear")),
    )
    .expect("denial evidence");
    let unavailable = worth_foundational::FoundationalScopedMergeUnavailablePosture::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        scope,
        worth_foundational::FoundationalScopedMergeUnavailableReason::RuntimeDoesNotSupportSelectedNodes,
    )
    .expect("unavailable posture");
    let denial_entries = ready_denial(&denial).payload().entries().to_vec();
    let unavailable_entries = ready_unavailable(&unavailable).payload().entries().to_vec();

    assert_contains_text(
        &denial_entries,
        "merge.scope_denial.denial_kind",
        "unknown-selected-node",
    );
    assert_contains_text(
        &unavailable_entries,
        "merge.scope_unavailable.outcome_category",
        "deferred",
    );
    assert!(matches!(
        exact_compare(ready_denial(&denial), ready_unavailable(&unavailable)),
        worth_foundational::CanonicalComparisonOutcome::Mismatched(_)
    ));
}
