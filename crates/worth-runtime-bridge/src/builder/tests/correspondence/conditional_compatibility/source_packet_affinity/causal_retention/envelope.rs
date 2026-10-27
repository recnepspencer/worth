//! The actual Native source remains indispensable after descriptive projection.
use super::*;
use crate::facade::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalEnvelopeDenialKind as Denial,
    BridgeCausalEvidenceBindingClass as Class, BridgeCausalEvidenceFamily as Family,
    BridgeCausalEvidenceOwner as Owner, BridgeCausalEvidenceReference as Reference,
    BridgeCausalEvidenceReferenceIdentity as Identity, BridgeCausalExplanationEnvelope,
    BridgeCausalInspectionAdmissionSummary as Summary,
    BridgeCausalInspectionAdmissionSummaryKind as Kind,
    BridgeConditionalCausalEnvelopeRequest as Request, BridgeConditionalCausalObservation,
    BridgeIdentityEvidence as Evidence,
};

pub(super) fn assemble_and_check(
    owner: &BridgeSealedRuntimeAssembly,
    observation: &BridgeConditionalCausalObservation,
) -> BridgeCausalExplanationEnvelope {
    let foreign_contacts = Arc::new(Contacts::default());
    let foreign_readers = Arc::new(AtomicUsize::new(0));
    let (foreign, _) = super::owner(Arc::clone(&foreign_contacts), Arc::clone(&foreign_readers));
    let denial = foreign
        .assemble_conditional_causal_envelope(request(observation))
        .unwrap_err();
    assert_eq!(denial.kind(), Denial::ForeignNativeObservation);
    assert_eq!(denial.counters().evidence_reference_count(), 0);
    assert_eq!(denial.counters().bridge_retained_lookup_count(), 0);
    assert_eq!(foreign_contacts.reads.load(Ordering::SeqCst), 0);
    assert_eq!(foreign_contacts.predicates.load(Ordering::SeqCst), 0);
    assert_eq!(foreign_contacts.computes.load(Ordering::SeqCst), 0);
    assert_eq!(foreign_readers.load(Ordering::SeqCst), 0);
    let envelope = owner
        .assemble_conditional_causal_envelope(request(observation))
        .unwrap();
    assert!(envelope.retains_native_observation(observation));
    assert!(!envelope
        .bindings()
        .iter()
        .any(|binding| binding.family() == Family::BridgeRoute));
    let native = envelope
        .bindings()
        .iter()
        .filter(|binding| binding.binding_class() == Class::RetainedNativeObservation)
        .count();
    assert_eq!(
        native, 3,
        "the genuine original backing still contains its performed invalidation"
    );
    assert_eq!(
        envelope
            .counters()
            .retained_native_observation_binding_count(),
        native
    );
    assert_eq!(envelope.counters().bridge_retained_lookup_count(), 0);
    assert_eq!(envelope.counters().retained_bridge_binding_count(), 0);
    assert_eq!(envelope.counters().external_authority_reference_count(), 1);
    reject_projection_only(&envelope);
    reject_mismatched_query_association();
    envelope
}

fn summary() -> Summary {
    Summary::for_native_observation(
        Kind::Admitted,
        Evidence::from_bridge_owner_external_authority("query-admission"),
        Evidence::from_bridge_owner_external_authority("causal-anchor"),
        Evidence::from_bridge_owner_external_authority("query-observation"),
    )
    .unwrap()
}

fn request(observation: &BridgeConditionalCausalObservation) -> Request {
    Request::new(summary(), observation.clone()).unwrap()
}

fn query_reference(identity: &str) -> Reference {
    Reference::new(
        Owner::Query,
        Family::QueryObservation,
        Identity::query_observation(Evidence::from_bridge_owner_external_authority(identity))
            .unwrap(),
    )
    .unwrap()
}

fn reject_projection_only(envelope: &BridgeCausalExplanationEnvelope) {
    let projected = envelope
        .bindings()
        .iter()
        .find(|binding| binding.family() == Family::BridgeConditionalObservation)
        .unwrap();
    let reference = Reference::new(
        Owner::RuntimeBridge,
        Family::BridgeConditionalObservation,
        Identity::runtime_bridge(
            Family::BridgeConditionalObservation,
            projected.reference_evidence_identity(),
        )
        .unwrap(),
    )
    .unwrap();
    let descriptor = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        summary(),
        vec![query_reference("query-observation"), reference],
    )
    .unwrap();
    let legacy = super::bridge_runtime(
        Vec::new(),
        Arc::new(Contacts::default()),
        Arc::new(AtomicUsize::new(0)),
    );
    let denial = legacy
        .diagnostics()
        .assemble_causal_explanation_envelope(descriptor)
        .unwrap_err();
    assert_eq!(denial.kind(), Denial::NativeObservationRequired);
    assert_eq!(denial.counters().bridge_retained_lookup_count(), 0);
    assert_eq!(
        denial
            .counters()
            .retained_native_observation_binding_count(),
        0
    );
    assert_eq!(denial.counters().missing_bridge_record_count(), 0);
}

fn reject_mismatched_query_association() {
    let denial = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        summary(),
        vec![query_reference("different-query-observation")],
    )
    .unwrap_err();
    assert_eq!(denial.kind(), Denial::QueryObservationAssociationMismatch);
}
