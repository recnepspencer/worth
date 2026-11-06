use super::*;
use worth_foundational::facade::{CanonicalDigestDerivationDenial, CanonicalDigestWorkBudget};

#[test]
fn envelope_limit_returns_the_artifact_subject_and_typed_refusal() {
    let maximum = CanonicalDigestWorkBudget::standard().maximum_entry_count() as usize;
    let parts = (0..=maximum).map(|_| value_part("entry")).collect::<Vec<_>>();
    let artifact = BridgeCausalEnvelopeDigestArtifact::BindingSet;
    let denial = compose_bridge_causal_envelope_evidence_identity(artifact, &parts).expect_err("over the entry limit");
    assert_eq!(denial.subject, artifact.digest_domain());
    assert!(matches!(denial.cause, CanonicalDigestDerivationDenial::EntryLimitExceeded { maximum, actual } if actual > maximum));
}

#[test]
fn retained_mapping_limit_returns_the_artifact_subject_and_typed_refusal() {
    use super::retained_mapping::digest_basis::{compose_retained_causal_mapping_evidence_identity, retained_mapping_shape_part, RetainedCausalMappingDigestArtifact};
    let maximum = CanonicalDigestWorkBudget::standard().maximum_entry_count() as usize;
    let parts = (0..=maximum).map(|_| retained_mapping_shape_part("entry")).collect::<Vec<_>>();
    let artifact = RetainedCausalMappingDigestArtifact::RouteRecord;
    let denial = compose_retained_causal_mapping_evidence_identity(artifact, &parts).expect_err("over the entry limit");
    assert_eq!(denial.subject, artifact.digest_domain());
    assert!(matches!(denial.cause, CanonicalDigestDerivationDenial::EntryLimitExceeded { maximum, actual } if actual > maximum));
}
