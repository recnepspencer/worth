pub(crate) use worth_query_bank_testkit_estate_certification::facade::canonical_certification::*;

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
