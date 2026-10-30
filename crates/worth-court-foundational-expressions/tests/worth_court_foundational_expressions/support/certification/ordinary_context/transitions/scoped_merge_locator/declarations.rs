pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_foundational::prepare_canonical_comparison,
    worth_foundational::prepare_locator_for_canonical_basis,
    worth_foundational::CanonicalBasisDomain, worth_foundational::CanonicalBasisEntry,
    worth_foundational::CanonicalBasisEntryKind, worth_foundational::CanonicalBasisLocus,
    worth_foundational::CanonicalBasisReadyArtifact, worth_foundational::CanonicalBasisValue,
    worth_foundational::CanonicalComparisonOutcome, worth_foundational::CanonicalEquivalenceBasis,
    worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalLocatorInput,
    worth_foundational::CanonicalizationRuleVersion,
    worth_foundational::FoundationalBoundaryEvidenceAuthorityPath,
    worth_foundational::FoundationalBoundaryEvidenceReceiptBoundary,
    worth_foundational::FoundationalDiagnosticLocator,
    worth_foundational::FoundationalMergeConflictLocator,
    worth_foundational::FoundationalSelectedAspectScopeLocator,
    worth_foundational::FoundationalSelectedNodeScopeLocator,
    worth_foundational::FoundationalTransitionLocator,
};

pub(crate) use crate::certification::ordinary_context::transitions::fixtures::branch::branch_id;
pub(crate) use {
    crate::certification::ordinary_context::transitions::fixtures::merge::authority_first_merge_candidate,
    crate::certification::ordinary_context::transitions::fixtures::merge::conflict_locus,
};
pub(crate) use {
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::selected_aspect,
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::selected_node,
};

pub(crate) fn locator_entries(locator: FoundationalTransitionLocator) -> Vec<CanonicalBasisEntry> {
    ready_locator(locator).payload().entries().to_vec()
}

pub(crate) fn ready_locator(locator: FoundationalTransitionLocator) -> CanonicalBasisReadyArtifact {
    match prepare_locator_for_canonical_basis(
        version(),
        worth_foundational::CanonicalLocatorInput::Transition(locator),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready scoped transition locator"),
    }
}

pub(crate) fn assert_mismatched(
    left: CanonicalBasisReadyArtifact,
    right: CanonicalBasisReadyArtifact,
) {
    let ready = match prepare_canonical_comparison(
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left,
        right,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected comparison readiness"),
    };
    assert!(matches!(
        worth_foundational::compare_canonical_basis(&ready),
        worth_foundational::CanonicalComparisonOutcome::Mismatched(_)
    ));
}

pub(crate) fn transition_locator_text_entry(locus: &str, value: &str) -> CanonicalBasisEntry {
    worth_foundational::CanonicalBasisEntry::new(
        worth_foundational::CanonicalBasisDomain::Locator,
        worth_foundational::CanonicalBasisLocus::Named(locus.into()),
        worth_foundational::CanonicalBasisEntryKind::TransitionLocator,
        worth_foundational::CanonicalBasisValue::ExactText(value.into()),
    )
}

#[allow(dead_code)]
pub(crate) fn transition_locator_integer_entry(locus: &str, value: u64) -> CanonicalBasisEntry {
    worth_foundational::CanonicalBasisEntry::new(
        worth_foundational::CanonicalBasisDomain::Locator,
        worth_foundational::CanonicalBasisLocus::Named(locus.into()),
        worth_foundational::CanonicalBasisEntryKind::TransitionLocator,
        worth_foundational::CanonicalBasisValue::UnsignedInteger {
            width: worth_foundational::CanonicalIntegerWidth::Bits64,
            value: u128::from(value),
        },
    )
}

pub(crate) fn version() -> CanonicalizationRuleVersion {
    worth_foundational::CanonicalizationRuleVersion::new("milestone-9-phase-12").expect("version")
}
