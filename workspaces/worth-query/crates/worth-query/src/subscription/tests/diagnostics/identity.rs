use crate::live::LiveQueryFamily;

use super::world::certified_subscription_identity;
use super::*;

#[test]
fn policy_tenant_and_relationship_proof_context_change_subscription_meaning() {
    let tenant_alpha =
        certified_subscription_identity("policy-alpha", "tenant-alpha", "proof-alpha");
    let tenant_beta = certified_subscription_identity("policy-alpha", "tenant-beta", "proof-alpha");
    let proof_beta = certified_subscription_identity("policy-alpha", "tenant-alpha", "proof-beta");

    assert_ne!(
        tenant_alpha.declaration_digest,
        tenant_beta.declaration_digest
    );
    assert_ne!(
        tenant_alpha.basis_request_digest,
        tenant_beta.basis_request_digest
    );
    assert_ne!(
        tenant_alpha.declaration_digest,
        proof_beta.declaration_digest
    );
    assert_ne!(
        tenant_alpha.basis_request_digest,
        proof_beta.basis_request_digest
    );
    assert_ne!(
        tenant_beta.declaration_digest,
        proof_beta.declaration_digest
    );
    assert_ne!(
        tenant_beta.basis_request_digest,
        proof_beta.basis_request_digest
    );
}

#[test]
fn relationship_proof_drift_denies_before_declaration_or_bridge_lowering() {
    let live = LiveQueryAdmissionArtifact::for_test_with_relationship_proof_posture(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::SavedExactReuse,
        QuerySubscriptionRelationshipProofPosture::Drifted,
    )
    .expect("fixture digest must fit");

    let error = select_query_subscription_family(live, roomy_budget()).unwrap_err();

    assert_eq!(
        error.failure_class(),
        &QuerySubscriptionFamilySelectionFailureClass::RelationshipProofAdmissionDrift
    );
    assert_eq!(
        error.diagnostic().expect("fixture digest must fit").stage(),
        &QuerySubscriptionDiagnosticStage::RelationshipProofDrift
    );
    assert_eq!(error.counters().family_selection_count(), 0);
    assert_eq!(error.counters().family_registry_lookup_count(), 0);
    assert_eq!(error.counters().view_family_registry_lookup_count(), 0);
    assert_eq!(error.counters().family_denial_count(), 1);
    assert_eq!(error.counters().declaration_count(), 0);
    assert_eq!(error.counters().bridge_lowering_count(), 0);
    assert_eq!(error.counters().relationship_proof_drift_denial_count(), 1);
    assert_eq!(
        error
            .diagnostic()
            .expect("fixture digest must fit")
            .counter_projection()
            .label()
            .as_str(),
        error.counters().counter_projection().label()
    );
    assert!(error.message().contains("relationship proof posture"));
    assert!(!error
        .diagnostic()
        .expect("fixture digest must fit")
        .source_projection()
        .label()
        .is_empty());
}
