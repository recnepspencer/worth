pub(crate) use worth_store_test_support::harness::physical_isolation::epoch_scope as support;

pub(crate) use worth_proof::BasisPostureKind;
pub(crate) use {
    worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisLocus,
    worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalIntegerWidth,
    worth_foundational::InternedString,
};
pub(crate) use {
    worth_store_physical_isolation::compare_physical_epoch_vectors_with_evidence,
    worth_store_physical_isolation::EpochComparisonScope,
    worth_store_physical_isolation::EpochRetryDecision,
    worth_store_physical_isolation::PhysicalEpochDriftKind,
    worth_store_physical_isolation::PhysicalEpochVector,
};
pub(crate) use {
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_extent_reference,
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_page_reference,
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_segment_reference,
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_root_from_authority,
    worth_store_test_support::harness::physical_isolation::epoch_scope::physical_authority_from_complete_closeout,
};

pub(crate) fn assert_u64_entry(
    entries: &[CanonicalBasisEntry],
    locus: &'static str,
    expected: u64,
) {
    assert_eq!(
        entry_value(entries, locus),
        &worth_foundational::CanonicalBasisValue::UnsignedInteger {
            width: worth_foundational::CanonicalIntegerWidth::Bits64,
            value: expected.into(),
        }
    );
}

pub(crate) fn assert_text_entry(
    entries: &[CanonicalBasisEntry],
    locus: &'static str,
    expected: &'static str,
) {
    assert_eq!(
        entry_value(entries, locus),
        &worth_foundational::CanonicalBasisValue::ExactText(expected.into())
    );
}

pub(crate) fn entry_value<'a>(
    entries: &'a [CanonicalBasisEntry],
    locus: &'static str,
) -> &'a CanonicalBasisValue {
    let expected_locus = worth_foundational::CanonicalBasisLocus::Named(
        worth_foundational::InternedString::from(locus),
    );
    entries
        .iter()
        .find(|entry| entry.locus() == &expected_locus)
        .unwrap_or_else(|| panic!("missing canonical basis entry for {locus}"))
        .value()
}
