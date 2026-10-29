pub(crate) use {worth_foundational::prepare_locator_for_canonical_basis, worth_foundational::CanonicalBasisDomain, worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisEntryKind, worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalLocatorInput, worth_foundational::FoundationalBranchCandidateLocator, worth_foundational::FoundationalCommitParentageLocator, worth_foundational::FoundationalCommittedDeltaLocator, worth_foundational::FoundationalMergeConflictLocator, worth_foundational::FoundationalTransitionLocator};
pub(crate) use {worth_proof::TransitionOutcome};

pub(crate) use {crate::certification::ordinary_context::transitions::fixtures::branch::authority_first_candidate};
pub(crate) use {crate::certification::ordinary_context::transitions::fixtures::committed::accepted_verdict, crate::certification::ordinary_context::transitions::fixtures::committed::committed_authority, crate::certification::ordinary_context::transitions::fixtures::committed::ordinary_commit_input};
pub(crate) use {crate::certification::ordinary_context::transitions::fixtures::merge::authority_first_merge_candidate, crate::certification::ordinary_context::transitions::fixtures::merge::conflict_locus};
pub(crate) use {crate::certification::ordinary_context::transitions::fixtures::receipt::commit_id};
pub(crate) use {crate::certification::ordinary_context::transitions::phase5_basis::canonical_basis::version};

pub(crate) fn locator_entries(
    locator: FoundationalTransitionLocator,
) -> Vec<worth_foundational::CanonicalBasisEntry> {
    match prepare_locator_for_canonical_basis(version(), worth_foundational::CanonicalLocatorInput::Transition(locator))
    {
        worth_proof::TransitionOutcome::Success(ready) => ready.payload().entries().to_vec(),
        _ => panic!("expected ready locator basis"),
    }
}

pub(crate) fn transition_locator_text_entry(locus: &str, value: &str) -> CanonicalBasisEntry {
    worth_foundational::CanonicalBasisEntry::new(
        worth_foundational::CanonicalBasisDomain::Locator,
        worth_foundational::CanonicalBasisLocus::Named(locus.into()),
        worth_foundational::CanonicalBasisEntryKind::TransitionLocator,
        worth_foundational::CanonicalBasisValue::ExactText(value.into()),
    )
}

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