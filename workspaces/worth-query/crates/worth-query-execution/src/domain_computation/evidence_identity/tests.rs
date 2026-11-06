use super::artifact::WorthQueryEvidenceIdentityComparisonError;
use super::encoder::{worth_query_evidence_identity, worth_query_evidence_identity_with_scheme};
use super::scheme::WorthQueryEvidenceIdentityScheme;
use super::scope::WorthQueryEvidenceScope;
use super::tag::WorthQueryEvidenceTag;

#[test]
fn evidence_identity_sequences_do_not_collapse_delimiter_injection() {
    let left = worth_query_evidence_identity(WorthQueryEvidenceScope::IntentDenialEvidence)
        .field_value_sequence(
            WorthQueryEvidenceTag::new("invariant_evidence"),
            ["alpha|beta", "gamma"],
        )
        .seal().expect("bounded evidence fixture");
    let right = worth_query_evidence_identity(WorthQueryEvidenceScope::IntentDenialEvidence)
        .field_value_sequence(
            WorthQueryEvidenceTag::new("invariant_evidence"),
            ["alpha", "beta|gamma"],
        )
        .seal().expect("bounded evidence fixture");

    assert_ne!(left.as_str(), right.as_str());
}

#[test]
fn evidence_identity_sequences_do_not_collide_with_tag_shaped_scalar_fields() {
    let sequence = worth_query_evidence_identity(WorthQueryEvidenceScope::IntentDenialEvidence)
        .field_value_sequence(WorthQueryEvidenceTag::new("invariant_evidence"), ["alpha"])
        .seal().expect("bounded evidence fixture");
    let scalar = worth_query_evidence_identity(WorthQueryEvidenceScope::IntentDenialEvidence)
        .field_value(WorthQueryEvidenceTag::new("invariant_evidence.0"), "alpha")
        .seal().expect("bounded evidence fixture");

    assert_ne!(sequence.as_str(), scalar.as_str());
}

#[test]
fn evidence_identity_empty_sequences_do_not_collapse_with_omitted_fields() {
    let explicit_empty =
        worth_query_evidence_identity(WorthQueryEvidenceScope::IntentDenialEvidence)
            .field_value_sequence(
                WorthQueryEvidenceTag::new("invariant_evidence"),
                std::iter::empty::<&str>(),
            )
            .seal().expect("bounded evidence fixture");
    let omitted =
        worth_query_evidence_identity(WorthQueryEvidenceScope::IntentDenialEvidence).seal().expect("bounded evidence fixture");
    let omitted_with_same_neighbor =
        worth_query_evidence_identity(WorthQueryEvidenceScope::IntentDenialEvidence)
            .field_shape(WorthQueryEvidenceTag::new("stage"), "admission")
            .seal().expect("bounded evidence fixture");
    let explicit_empty_with_same_neighbor =
        worth_query_evidence_identity(WorthQueryEvidenceScope::IntentDenialEvidence)
            .field_shape(WorthQueryEvidenceTag::new("stage"), "admission")
            .field_value_sequence(
                WorthQueryEvidenceTag::new("invariant_evidence"),
                std::iter::empty::<&str>(),
            )
            .seal().expect("bounded evidence fixture");

    assert_ne!(explicit_empty.as_str(), omitted.as_str());
    assert_ne!(
        explicit_empty_with_same_neighbor.as_str(),
        omitted_with_same_neighbor.as_str()
    );
}

#[test]
fn evidence_identity_scope_stays_part_of_the_public_identity_contract() {
    let left = worth_query_evidence_identity(WorthQueryEvidenceScope::PreviewIntentAdmission)
        .field_value(WorthQueryEvidenceTag::new("input_digest"), "input:v1")
        .seal().expect("bounded evidence fixture");
    let right = worth_query_evidence_identity(WorthQueryEvidenceScope::PreviewIntentReceipt)
        .field_value(WorthQueryEvidenceTag::new("input_digest"), "input:v1")
        .seal().expect("bounded evidence fixture");

    assert_ne!(left.as_str(), right.as_str());
}

#[test]
fn evidence_identity_keeps_explicit_scheme_metadata() {
    let identity = worth_query_evidence_identity(WorthQueryEvidenceScope::RuntimeStateSnapshot)
        .field_shape(WorthQueryEvidenceTag::new("state_kind"), "ready")
        .seal().expect("bounded evidence fixture");

    assert_eq!(identity.scheme(), WorthQueryEvidenceIdentityScheme::V1);
    assert_eq!(
        identity.scheme().as_str(),
        "worth.query.evidence-identity.v1"
    );
    assert!(identity
        .as_str()
        .starts_with("worth.query.evidence-identity.v1:sha256:"));
    assert!(!identity.canonical_digest().value().bytes().is_empty());
}

#[test]
fn evidence_identity_cross_scheme_comparison_fails_typed() {
    let left = worth_query_evidence_identity_with_scheme(
        WorthQueryEvidenceScope::RuntimeStateSnapshot,
        WorthQueryEvidenceIdentityScheme::V1,
    )
    .field_shape(WorthQueryEvidenceTag::new("state_kind"), "ready")
    .seal().expect("bounded evidence fixture");
    let right = worth_query_evidence_identity_with_scheme(
        WorthQueryEvidenceScope::RuntimeStateSnapshot,
        WorthQueryEvidenceIdentityScheme::V2,
    )
    .field_shape(WorthQueryEvidenceTag::new("state_kind"), "ready")
    .seal().expect("bounded evidence fixture");

    assert_eq!(
        left.compare_same_scheme(&right),
        Err(WorthQueryEvidenceIdentityComparisonError::SchemeMismatch {
            left: WorthQueryEvidenceIdentityScheme::V1,
            right: WorthQueryEvidenceIdentityScheme::V2,
        })
    );
}

#[test]
fn evidence_identity_typed_scalar_helpers_preserve_canonical_distinctions() {
    let left = worth_query_evidence_identity(WorthQueryEvidenceScope::RuntimePublicSupportMatrix)
        .field_bool(WorthQueryEvidenceTag::new("parallel_api_forbidden"), true)
        .field_usize(WorthQueryEvidenceTag::new("stable_row_count"), 4)
        .seal().expect("bounded evidence fixture");
    let same = worth_query_evidence_identity(WorthQueryEvidenceScope::RuntimePublicSupportMatrix)
        .field_bool(WorthQueryEvidenceTag::new("parallel_api_forbidden"), true)
        .field_usize(WorthQueryEvidenceTag::new("stable_row_count"), 4)
        .seal().expect("bounded evidence fixture");
    let changed_bool =
        worth_query_evidence_identity(WorthQueryEvidenceScope::RuntimePublicSupportMatrix)
            .field_bool(WorthQueryEvidenceTag::new("parallel_api_forbidden"), false)
            .field_usize(WorthQueryEvidenceTag::new("stable_row_count"), 4)
            .seal().expect("bounded evidence fixture");
    let changed_count =
        worth_query_evidence_identity(WorthQueryEvidenceScope::RuntimePublicSupportMatrix)
            .field_bool(WorthQueryEvidenceTag::new("parallel_api_forbidden"), true)
            .field_usize(WorthQueryEvidenceTag::new("stable_row_count"), 5)
            .seal().expect("bounded evidence fixture");

    assert_eq!(left.eq_same_scheme(&same), Ok(true));
    assert_eq!(left.eq_same_scheme(&changed_bool), Ok(false));
    assert_eq!(left.eq_same_scheme(&changed_count), Ok(false));
}

#[test]
fn evidence_identity_exports_explicit_bridge_boundary_categories() {
    let identity = worth_query_evidence_identity(WorthQueryEvidenceScope::EffectIntentReceipt)
        .field_shape(WorthQueryEvidenceTag::new("role"), "signal-routing")
        .field_value(WorthQueryEvidenceTag::new("receipt"), "receipt:v1")
        .seal().expect("bounded evidence fixture");

    let external = identity.bridge_external_identity_evidence();
    let query_evidence = identity.bridge_evidence_identity();

    assert_eq!(
        external.terminal_projection_for_reporting(),
        identity.terminal_projection_for_reporting()
    );
    assert_eq!(
        query_evidence.terminal_projection_for_reporting(),
        identity.scope().as_str()
    );
}

#[test]
fn operational_keys_preserve_identity_without_exposing_terminal_text() {
    let left =
        worth_query_evidence_identity(WorthQueryEvidenceScope::ProjectionConsumptionIdentity)
            .field_value(WorthQueryEvidenceTag::new("row"), "alpha")
            .seal().expect("bounded evidence fixture");
    let same =
        worth_query_evidence_identity(WorthQueryEvidenceScope::ProjectionConsumptionIdentity)
            .field_value(WorthQueryEvidenceTag::new("row"), "alpha")
            .seal().expect("bounded evidence fixture");
    let changed =
        worth_query_evidence_identity(WorthQueryEvidenceScope::ProjectionConsumptionIdentity)
            .field_value(WorthQueryEvidenceTag::new("row"), "beta")
            .seal().expect("bounded evidence fixture");

    assert_eq!(left.operational_key(), same.operational_key());
    assert_ne!(left.operational_key(), changed.operational_key());
    assert!(!format!("{:?}", left.operational_key()).contains(left.as_str()));
}

#[test]
fn operational_keys_are_hashable_and_keep_scope_and_scheme() {
    let identity =
        worth_query_evidence_identity(WorthQueryEvidenceScope::ProjectionConsumptionIdentity)
            .field_value(WorthQueryEvidenceTag::new("row"), "alpha")
            .seal().expect("bounded evidence fixture");
    let key = identity.operational_key();
    let mut hash = std::collections::HashSet::new();
    let mut ordered = std::collections::BTreeSet::new();

    assert!(hash.insert(key));
    assert!(ordered.insert(key));
    assert_eq!(key.scope(), identity.scope());
    assert_eq!(key.scheme(), identity.scheme());
    assert_eq!(key.correlation_digest().len(), 32);
}

#[test]
fn oversized_evidence_identity_returns_its_scope_and_entry_refusal() {
    let scope = WorthQueryEvidenceScope::IntentDenialEvidence;
    let maximum = worth_foundational::facade::CanonicalDigestWorkBudget::standard().maximum_entry_count() as usize;
    let denial = worth_query_evidence_identity(scope)
        .field_value_sequence(WorthQueryEvidenceTag::new("items"), (0..=maximum).map(|value| value.to_string()))
        .seal().expect_err("oversized identity must refuse");
    assert_eq!(denial.scope, scope);
    assert!(matches!(denial.cause, worth_foundational::facade::CanonicalDigestDerivationDenial::EntryLimitExceeded { maximum, actual } if actual > maximum));
}

#[test]
fn oversized_evidence_text_returns_its_scope_and_byte_refusal() {
    let scope = WorthQueryEvidenceScope::IntentDenialEvidence;
    let maximum = worth_foundational::facade::CanonicalDigestWorkBudget::standard().maximum_encoded_bytes();
    let denial = worth_query_evidence_identity(scope)
        .field_value(WorthQueryEvidenceTag::new("text"), "x".repeat(maximum))
        .seal().expect_err("oversized text must refuse");
    assert_eq!(denial.scope, scope);
    assert!(matches!(denial.cause, worth_foundational::facade::CanonicalDigestDerivationDenial::EncodedByteLimitExceeded { maximum, attempted } if attempted > maximum));
}
