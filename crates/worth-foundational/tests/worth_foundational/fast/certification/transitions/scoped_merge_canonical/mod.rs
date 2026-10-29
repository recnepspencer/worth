use worth_foundational::{
    compare_canonical_basis, prepare_admitted_merge_scope_for_canonical_basis,
    prepare_canonical_comparison, prepare_merge_scope_for_canonical_basis,
    prepare_merge_verdict_for_canonical_basis, prepare_scoped_merge_denial_for_canonical_basis,
    prepare_scoped_merge_unavailable_for_canonical_basis, CanonicalBasisEntry, CanonicalBasisLocus,
    CanonicalBasisReadyArtifact, CanonicalBasisValue, CanonicalComparisonOutcome,
    CanonicalEquivalenceBasis, CanonicalIntegerWidth, CanonicalizationRuleVersion,
    FoundationalAdmittedMergeScopeEvidence, FoundationalDeniedScopeLocus, FoundationalMergeScope,
    FoundationalScopeAdmissionBasis, FoundationalScopedMergeDenialEvidence,
    FoundationalScopedMergeDenialKind, FoundationalScopedMergeUnavailablePosture,
    FoundationalScopedMergeUnavailableReason, FoundationalSelectedScopeLocus,
    FoundationalSelectedScopeNoOpCause, FoundationalSelectedScopeNoOpEvidence,
    FoundationalSkippedOutOfScopeEvidence, InternedString,
};
use worth_proof::TransitionOutcome;

use super::fixtures::branch::branch_id;
use super::fixtures::scoped_merge::{scoped_candidate, selected_aspect, selected_node};

#[test]
fn selected_node_scope_canonical_basis_is_order_independent() {
    let left = ready_scope(
        &FoundationalMergeScope::selected_nodes([selected_node("gear"), selected_node("material")])
            .expect("left scope"),
    );
    let right = ready_scope(
        &FoundationalMergeScope::selected_nodes([selected_node("material"), selected_node("gear")])
            .expect("right scope"),
    );

    assert_equivalent(left, right);
}

#[test]
fn selected_aspect_scope_and_admitted_evidence_canonicalize_by_node_then_aspect() {
    let requested_scope = FoundationalMergeScope::selected_aspects([
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
    let scope =
        FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope");
    let evidence = FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        scope.clone(),
        FoundationalScopeAdmissionBasis::DirectSourceIdentity,
        [selected_node("gear")],
        [],
        [],
        FoundationalSkippedOutOfScopeEvidence::new(2, None),
        2,
    )
    .expect("scope evidence");
    let verdict = match scoped_candidate(scope).admit_as_accepted_with_scope_evidence(evidence) {
        TransitionOutcome::Success(verdict) => verdict,
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
    let scope =
        FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope");
    let denial = FoundationalScopedMergeDenialEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        scope.clone(),
        FoundationalScopedMergeDenialKind::UnknownSelectedNode,
        FoundationalDeniedScopeLocus::Node(selected_node("gear")),
    )
    .expect("denial evidence");
    let unavailable = FoundationalScopedMergeUnavailablePosture::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        scope,
        FoundationalScopedMergeUnavailableReason::RuntimeDoesNotSupportSelectedNodes,
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
        CanonicalComparisonOutcome::Mismatched(_)
    ));
}

fn admitted_scope_evidence(
    requested_scope: FoundationalMergeScope,
    admitted_aspects: impl IntoIterator<
        Item = worth_foundational::FoundationalSelectedAspectRequestEntry,
    >,
    selected_no_ops: impl IntoIterator<Item = FoundationalSelectedScopeNoOpEvidence>,
) -> FoundationalAdmittedMergeScopeEvidence {
    FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        requested_scope,
        FoundationalScopeAdmissionBasis::IdentityCorresponded,
        [],
        admitted_aspects,
        selected_no_ops,
        FoundationalSkippedOutOfScopeEvidence::new(1, None),
        2,
    )
    .expect("admitted scope evidence")
}

fn no_op_for_aspect(node: &str, aspect: &str) -> FoundationalSelectedScopeNoOpEvidence {
    FoundationalSelectedScopeNoOpEvidence::new(
        FoundationalSelectedScopeLocus::Aspect(selected_aspect(node, aspect)),
        FoundationalSelectedScopeNoOpCause::EquivalentTargetTruth,
    )
}

fn ready_scope(scope: &FoundationalMergeScope) -> CanonicalBasisReadyArtifact {
    match prepare_merge_scope_for_canonical_basis(version(), scope) {
        TransitionOutcome::Success(ready) => ready,
        other => panic!("expected ready scope basis, got {other:?}"),
    }
}

fn ready_admitted_scope(
    evidence: &FoundationalAdmittedMergeScopeEvidence,
) -> CanonicalBasisReadyArtifact {
    match prepare_admitted_merge_scope_for_canonical_basis(version(), evidence) {
        TransitionOutcome::Success(ready) => ready,
        other => panic!("expected ready admitted scope basis, got {other:?}"),
    }
}

fn ready_denial(evidence: &FoundationalScopedMergeDenialEvidence) -> CanonicalBasisReadyArtifact {
    match prepare_scoped_merge_denial_for_canonical_basis(version(), evidence) {
        TransitionOutcome::Success(ready) => ready,
        other => panic!("expected ready denial basis, got {other:?}"),
    }
}

fn ready_unavailable(
    posture: &FoundationalScopedMergeUnavailablePosture,
) -> CanonicalBasisReadyArtifact {
    match prepare_scoped_merge_unavailable_for_canonical_basis(version(), posture) {
        TransitionOutcome::Success(ready) => ready,
        other => panic!("expected ready unavailable basis, got {other:?}"),
    }
}

fn ready_verdict<T>(
    verdict: &worth_foundational::FoundationalMergeVerdict<T>,
) -> CanonicalBasisReadyArtifact {
    match prepare_merge_verdict_for_canonical_basis(version(), verdict) {
        TransitionOutcome::Success(ready) => ready,
        other => panic!("expected ready verdict basis, got {other:?}"),
    }
}

fn exact_compare(
    left: CanonicalBasisReadyArtifact,
    right: CanonicalBasisReadyArtifact,
) -> CanonicalComparisonOutcome {
    let ready = match prepare_canonical_comparison(
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left,
        right,
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected comparison readiness"),
    };
    compare_canonical_basis(&ready)
}

fn assert_equivalent(left: CanonicalBasisReadyArtifact, right: CanonicalBasisReadyArtifact) {
    assert!(matches!(
        exact_compare(left, right),
        CanonicalComparisonOutcome::Equivalent(_)
    ));
}

fn assert_contains_text(entries: &[CanonicalBasisEntry], locus: &str, value: &str) {
    assert!(entries.iter().any(|entry| {
        matches!(entry.locus(), CanonicalBasisLocus::Named(name) if interned_eq(name, locus))
            && matches!(entry.value(), CanonicalBasisValue::ExactText(text) if interned_eq(text, value))
    }));
}

fn assert_contains_u64(entries: &[CanonicalBasisEntry], locus: &str, value: u64) {
    assert!(entries.iter().any(|entry| {
        matches!(entry.locus(), CanonicalBasisLocus::Named(name) if interned_eq(name, locus))
            && matches!(
                entry.value(),
                CanonicalBasisValue::UnsignedInteger {
                    width: CanonicalIntegerWidth::Bits64,
                    value: actual
                } if *actual == u128::from(value)
            )
    }));
}

fn interned_eq(value: &InternedString, expected: &str) -> bool {
    matches!(value, InternedString::Raw(actual) if actual == expected)
}

fn version() -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new("milestone-9-phase-11").expect("version")
}
