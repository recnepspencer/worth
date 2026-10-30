pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
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

pub(crate) use crate::certification::ordinary_context::transitions::fixtures::branch::branch_id;
pub(crate) use {
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::scoped_candidate,
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::selected_aspect,
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::selected_node,
};

pub(crate) fn admitted_scope_evidence(
    requested_scope: FoundationalMergeScope,
    admitted_aspects: impl IntoIterator<
        Item = worth_foundational::FoundationalSelectedAspectRequestEntry,
    >,
    selected_no_ops: impl IntoIterator<Item = FoundationalSelectedScopeNoOpEvidence>,
) -> FoundationalAdmittedMergeScopeEvidence {
    worth_foundational::FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        requested_scope,
        worth_foundational::FoundationalScopeAdmissionBasis::IdentityCorresponded,
        [],
        admitted_aspects,
        selected_no_ops,
        worth_foundational::FoundationalSkippedOutOfScopeEvidence::new(1, None),
        2,
    )
    .expect("admitted scope evidence")
}

pub(crate) fn no_op_for_aspect(node: &str, aspect: &str) -> FoundationalSelectedScopeNoOpEvidence {
    worth_foundational::FoundationalSelectedScopeNoOpEvidence::new(
        worth_foundational::FoundationalSelectedScopeLocus::Aspect(selected_aspect(node, aspect)),
        worth_foundational::FoundationalSelectedScopeNoOpCause::EquivalentTargetTruth,
    )
}

pub(crate) fn ready_scope(scope: &FoundationalMergeScope) -> CanonicalBasisReadyArtifact {
    match prepare_merge_scope_for_canonical_basis(version(), scope) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        other => panic!("expected ready scope basis, got {other:?}"),
    }
}

pub(crate) fn ready_admitted_scope(
    evidence: &FoundationalAdmittedMergeScopeEvidence,
) -> CanonicalBasisReadyArtifact {
    match prepare_admitted_merge_scope_for_canonical_basis(version(), evidence) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        other => panic!("expected ready admitted scope basis, got {other:?}"),
    }
}

pub(crate) fn ready_denial(
    evidence: &FoundationalScopedMergeDenialEvidence,
) -> CanonicalBasisReadyArtifact {
    match prepare_scoped_merge_denial_for_canonical_basis(version(), evidence) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        other => panic!("expected ready denial basis, got {other:?}"),
    }
}

pub(crate) fn ready_unavailable(
    posture: &FoundationalScopedMergeUnavailablePosture,
) -> CanonicalBasisReadyArtifact {
    match prepare_scoped_merge_unavailable_for_canonical_basis(version(), posture) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        other => panic!("expected ready unavailable basis, got {other:?}"),
    }
}

pub(crate) fn ready_verdict<T>(
    verdict: &worth_foundational::FoundationalMergeVerdict<T>,
) -> CanonicalBasisReadyArtifact {
    match prepare_merge_verdict_for_canonical_basis(version(), verdict) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        other => panic!("expected ready verdict basis, got {other:?}"),
    }
}

pub(crate) fn exact_compare(
    left: CanonicalBasisReadyArtifact,
    right: CanonicalBasisReadyArtifact,
) -> CanonicalComparisonOutcome {
    let ready = match prepare_canonical_comparison(
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left,
        right,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected comparison readiness"),
    };
    compare_canonical_basis(&ready)
}

pub(crate) fn assert_equivalent(
    left: CanonicalBasisReadyArtifact,
    right: CanonicalBasisReadyArtifact,
) {
    assert!(matches!(
        exact_compare(left, right),
        worth_foundational::CanonicalComparisonOutcome::Equivalent(_)
    ));
}

pub(crate) fn assert_contains_text(entries: &[CanonicalBasisEntry], locus: &str, value: &str) {
    assert!(entries.iter().any(|entry| {
        matches!(entry.locus(), worth_foundational::CanonicalBasisLocus::Named(name) if interned_eq(name, locus))
            && matches!(entry.value(), worth_foundational::CanonicalBasisValue::ExactText(text) if interned_eq(text, value))
    }));
}

pub(crate) fn assert_contains_u64(entries: &[CanonicalBasisEntry], locus: &str, value: u64) {
    assert!(entries.iter().any(|entry| {
        matches!(entry.locus(), worth_foundational::CanonicalBasisLocus::Named(name) if interned_eq(name, locus))
            && matches!(
                entry.value(),
                worth_foundational::CanonicalBasisValue::UnsignedInteger {
                    width: worth_foundational::CanonicalIntegerWidth::Bits64,
                    value: actual
                } if *actual == u128::from(value)
            )
    }));
}

pub(crate) fn interned_eq(value: &InternedString, expected: &str) -> bool {
    matches!(value, worth_foundational::InternedString::Raw(actual) if actual == expected)
}

pub(crate) fn version() -> CanonicalizationRuleVersion {
    worth_foundational::CanonicalizationRuleVersion::new("milestone-9-phase-11").expect("version")
}
