use worth_query_execution::facade::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

#[test]
fn oversized_evidence_identity_returns_its_scope_and_entry_refusal() {
    let scope = WorthQueryEvidenceScope::IntentDenialEvidence;
    let maximum = worth_foundational::facade::CanonicalDigestWorkBudget::standard()
        .maximum_entry_count() as usize;
    let denial = worth_query_evidence_identity(scope)
        .field_value_sequence(
            WorthQueryEvidenceTag::new("items"),
            (0..=maximum).map(|value| value.to_string()),
        )
        .seal()
        .expect_err("oversized identity must refuse");
    assert_eq!(denial.scope, scope);
    assert!(
        matches!(denial.cause, worth_foundational::facade::CanonicalDigestDerivationDenial::EntryLimitExceeded { maximum, actual } if actual > maximum)
    );
}

#[test]
fn oversized_evidence_text_returns_its_scope_and_byte_refusal() {
    let scope = WorthQueryEvidenceScope::IntentDenialEvidence;
    let maximum =
        worth_foundational::facade::CanonicalDigestWorkBudget::standard().maximum_encoded_bytes();
    let denial = worth_query_evidence_identity(scope)
        .field_value(WorthQueryEvidenceTag::new("text"), "x".repeat(maximum))
        .seal()
        .expect_err("oversized text must refuse");
    assert_eq!(denial.scope, scope);
    assert!(
        matches!(denial.cause, worth_foundational::facade::CanonicalDigestDerivationDenial::EncodedByteLimitExceeded { maximum, attempted } if attempted > maximum)
    );
}
