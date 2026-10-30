pub(crate) use {
    worth_foundational::facade::prepare_canonical_basis_sequence,
    worth_foundational::facade::prepare_owned_canonical_basis_sequence_admitted,
    worth_foundational::facade::AspectKey,
    worth_foundational::facade::CanonicalBasisConstructionDenial,
    worth_foundational::facade::CanonicalBasisDomain,
    worth_foundational::facade::CanonicalBasisEntry,
    worth_foundational::facade::CanonicalBasisEntryKind,
    worth_foundational::facade::CanonicalBasisLocus,
    worth_foundational::facade::CanonicalBasisPreparationStop,
    worth_foundational::facade::CanonicalBasisValue,
    worth_foundational::facade::CanonicalFieldPath,
    worth_foundational::facade::CanonicalizationRuleVersion, worth_foundational::facade::FieldKey,
};

pub(crate) fn version() -> CanonicalizationRuleVersion {
    worth_foundational::facade::CanonicalizationRuleVersion::new("admitted.preparation.v1").unwrap()
}

pub(crate) fn entry(ordinal: usize) -> CanonicalBasisEntry {
    worth_foundational::facade::CanonicalBasisEntry::new(
        worth_foundational::facade::CanonicalBasisDomain::Value,
        worth_foundational::facade::CanonicalBasisLocus::Named(
            format!("field.{ordinal:04}").into(),
        ),
        worth_foundational::facade::CanonicalBasisEntryKind::Field,
        worth_foundational::facade::CanonicalBasisValue::ExactText(
            format!("value.{ordinal}").into(),
        ),
    )
}
