pub(crate) use {worth_foundational::classify_foundational_profile_compatibility, worth_foundational::compare_foundational_profile_identities, worth_foundational::derive_foundational_profile_identity, worth_foundational::foundational_profile_canonical_basis_entries, worth_foundational::prepare_admitted_foundational_profile_for_canonical_basis, worth_foundational::AdmissionReadinessProfile, worth_foundational::CanonicalBasisDomain, worth_foundational::CanonicalBasisEntryKind, worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalComparisonOutcome, worth_foundational::CanonicalizationRuleVersion, worth_foundational::CertificationPostureProfile, worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile, worth_foundational::FoundationalProfileCompatibilityClass, worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile};
pub(crate) use {worth_proof::TransitionOutcome};

pub(crate) use {crate::certification::ordinary_context::profiles::support::admit_same_profile, crate::certification::ordinary_context::profiles::support::profile};

pub(crate) fn version() -> CanonicalizationRuleVersion {
    worth_foundational::CanonicalizationRuleVersion::new("m3.profile.identity").expect("valid test version")
}