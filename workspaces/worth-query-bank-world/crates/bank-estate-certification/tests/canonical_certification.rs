#[path = "canonical_certification/support.rs"]
mod support;

use bank_domain::{
    estate::{EstateCapabilityPurpose, RestrictedBankField},
    queries,
    schema::RestrictedBankFieldBinding,
};
use support::{certification_fixture, ESTATE};
use worth_foundational::{
    admit_canonical_sequence_digest_derivation, compare_canonical_basis, derive_canonical_digest,
    prepare_canonical_basis_sequence, prepare_canonical_comparison, AspectValue,
    CanonicalBasisDomain, CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus,
    CanonicalBasisReadyArtifact, CanonicalBasisValue, CanonicalComparisonOutcome,
    CanonicalDigestAlgorithmId, CanonicalEquivalenceBasis,
    CanonicalSingleSequenceDigestAlgorithmSlot, CanonicalizationRuleVersion, InternedString,
};
use worth_query_host::facade::declaration::application_schema::ApplicationScalarValueBinding;

const RULE_VERSION: &str = "bank.estate.governed-disclosure-field.v1";

#[test]
fn capability_and_declared_disclosure_meaning_compare_canonically_before_closed_publication() {
    let fixture = certification_fixture();
    let request = queries::estate_legal_compliance(ESTATE);
    let capability_meaning = request.capability_request();
    assert_eq!(
        capability_meaning.purpose(),
        EstateCapabilityPurpose::LegalCompliance
    );
    let capability_value = RestrictedBankFieldBinding::encode(
        &capability_meaning
            .field()
            .expect("the product request carries its exact field"),
    )
    .expect("the governed field must encode through its installed binding");

    let definition = queries::estate_legal_compliance_definition();
    let disclosure_rules = definition.disclosure().rules();
    assert!(!disclosure_rules.is_empty());
    let disclosure_value = disclosure_rules[0].disclosure_value().clone();
    assert!(disclosure_rules
        .iter()
        .all(|rule| rule.disclosure_value() == &disclosure_value));

    let result = fixture
        .runtime
        .query(bank_server::queries::estate_legal_compliance(ESTATE))
        .as_principal(&fixture.principal)
        .controls(fixture.controls)
        .execute()
        .expect("the external consumer should execute the public product query");
    let publication = result.receipt().disclosure();
    assert_eq!(
        publication.disclosure_decision_count(),
        disclosure_rules.len()
    );
    assert!(publication.disclosed_value_count() > 0);
    assert_eq!(publication.omitted_value_count(), 0);

    assert_equivalent(&capability_value, &disclosure_value);
    assert_mismatched(
        &capability_value,
        &RestrictedBankFieldBinding::encode(&RestrictedBankField::AuditTrail)
            .expect("the comparison field must encode through its installed binding"),
    );

    let version = canonical_version();
    let admitted = admit_canonical_sequence_digest_derivation(
        ready(&capability_value),
        CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
            CanonicalDigestAlgorithmId::sha256(),
            CanonicalBasisDomain::Value,
            version,
        ),
    )
    .into_result()
    .expect("the matching single-sequence slot should admit the exact basis");
    let digest = derive_canonical_digest(admitted);
    assert_ne!(digest.value().bytes(), &[0; 32]);
}

fn assert_equivalent(left: &AspectValue, right: &AspectValue) {
    assert!(matches!(
        compare(left, right),
        CanonicalComparisonOutcome::Equivalent(_)
    ));
}

fn assert_mismatched(left: &AspectValue, right: &AspectValue) {
    assert!(matches!(
        compare(left, right),
        CanonicalComparisonOutcome::Mismatched(_)
    ));
}

fn compare(left: &AspectValue, right: &AspectValue) -> CanonicalComparisonOutcome {
    let comparison = prepare_canonical_comparison(
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ready(left),
        ready(right),
    )
    .into_result()
    .expect("the same-domain comparison should prepare");
    compare_canonical_basis(&comparison)
}

fn ready(value: &AspectValue) -> CanonicalBasisReadyArtifact {
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

fn clone_text(value: &InternedString) -> InternedString {
    value.clone()
}

fn canonical_version() -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new(RULE_VERSION).expect("the rule version is static and valid")
}
