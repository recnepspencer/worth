#[path = "canonical_certification/support.rs"]
pub mod support;

pub use bank_domain::{
    estate::{EstateCapabilityPurpose, RestrictedBankField},
    queries,
    schema::RestrictedBankFieldBinding,
};
pub use support::{certification_fixture, ESTATE};
pub use worth_foundational::{
    admit_canonical_sequence_digest_derivation, compare_canonical_basis, derive_canonical_digest,
    prepare_canonical_basis_sequence, prepare_canonical_comparison, AspectValue,
    CanonicalBasisDomain, CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus,
    CanonicalBasisReadyArtifact, CanonicalBasisValue, CanonicalComparisonOutcome,
    CanonicalDigestAlgorithmId, CanonicalEquivalenceBasis,
    CanonicalSingleSequenceDigestAlgorithmSlot, CanonicalizationRuleVersion, InternedString,
};
pub use worth_query_host::facade::declaration::application_schema::ApplicationScalarValueBinding;

pub const RULE_VERSION: &str = "bank.estate.governed-disclosure-field.v1";

pub fn assert_equivalent(left: &AspectValue, right: &AspectValue) {
    assert!(matches!(
        compare(left, right),
        CanonicalComparisonOutcome::Equivalent(_)
    ));
}

pub fn assert_mismatched(left: &AspectValue, right: &AspectValue) {
    assert!(matches!(
        compare(left, right),
        CanonicalComparisonOutcome::Mismatched(_)
    ));
}

pub fn compare(left: &AspectValue, right: &AspectValue) -> CanonicalComparisonOutcome {
    let comparison = prepare_canonical_comparison(
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ready(left),
        ready(right),
    )
    .into_result()
    .expect("the same-domain comparison should prepare");
    compare_canonical_basis(&comparison)
}

pub fn ready(value: &AspectValue) -> CanonicalBasisReadyArtifact {
    let AspectValue::String(value) = value else {
        panic!("the governed field must retain its typed string representation")
    };
    prepare_canonical_basis_sequence(
        canonical_version(),
        CanonicalBasisDomain::Value,
        [CanonicalBasisEntry::new(
            CanonicalBasisDomain::Value,
            CanonicalBasisLocus::Named("governed-disclosure-field".into()),
            CanonicalBasisEntryKind::Value,
            CanonicalBasisValue::ExactText(clone_text(value)),
        )],
    )
    .into_result()
    .expect("the typed field basis should prepare")
}

pub fn clone_text(value: &InternedString) -> InternedString {
    value.clone()
}

pub fn canonical_version() -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new(RULE_VERSION).expect("the rule version is static and valid")
}
