pub(crate) use {worth_foundational::diagnostic_mask_locator_canonical_basis_entries, worth_foundational::identity_canonical_basis_entries, worth_foundational::locator_canonical_basis_entries, worth_foundational::mutation_mask_locator_canonical_basis_entries, worth_foundational::prepare_canonical_basis_sequence, worth_foundational::prepare_identity_for_canonical_basis, worth_foundational::prepare_locator_for_canonical_basis, worth_foundational::projection_mask_locator_canonical_basis_entries, worth_foundational::AspectFieldLocator, worth_foundational::AspectKey, worth_foundational::AspectLocator, worth_foundational::AspectMask, worth_foundational::AspectValueLocator, worth_foundational::BoundaryArtifactField, worth_foundational::BoundaryArtifactId, worth_foundational::BoundaryArtifactLocator, worth_foundational::BoundaryEpoch, worth_foundational::BoundaryHandle, worth_foundational::BoundaryMismatchLocator, worth_foundational::BoundarySourceLocator, worth_foundational::CanonicalBasisDomain, worth_foundational::CanonicalBasisEntryKind, worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalFieldPath, worth_foundational::CanonicalIdentityInput, worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalLocatorInput, worth_foundational::CanonicalizationRuleVersion, worth_foundational::DiagnosticMask, worth_foundational::EquivalenceBasisId, worth_foundational::FieldKey, worth_foundational::LocatorAuthority, worth_foundational::MutationMask, worth_foundational::ProjectionMask};
pub(crate) use {worth_proof::TransitionOutcome};

pub(crate) fn version() -> CanonicalizationRuleVersion {
    worth_foundational::CanonicalizationRuleVersion::new("m2.identity-locator").expect("valid test version")
}

pub(crate) fn key(value: &str) -> AspectKey {
    worth_foundational::AspectKey::new(value).expect("valid aspect key")
}

pub(crate) fn field(value: &str) -> FieldKey {
    worth_foundational::FieldKey::new(value).expect("valid field")
}

pub(crate) fn path(value: &str) -> CanonicalFieldPath {
    worth_foundational::CanonicalFieldPath::single(field(value))
}