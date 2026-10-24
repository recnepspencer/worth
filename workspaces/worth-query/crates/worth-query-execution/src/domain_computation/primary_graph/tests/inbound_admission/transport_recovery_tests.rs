//! Recorded transport completion publishes once, whoever reaches World first.

use std::sync::atomic::{AtomicUsize, Ordering};

use super::fixture::installed_world;
use super::verifier::signed_envelope;
use crate::domain_computation::application_aftermath::{
    dispatch_external_effect, WorthQueryExternalDispatchRequest, WorthQueryExternalEffectTransport,
    WorthQueryExternalTransportOutcome,
};
use crate::domain_computation::primary_graph::application_runtime::InstalledTransportPendingReason;
use crate::domain_computation::primary_graph::{
    InstalledTransportCompletion, InstalledTransportResumeOutcome, WorthQueryInboundReceiptPosture,
};

struct CompletingTransport(AtomicUsize);

impl WorthQueryExternalEffectTransport for CompletingTransport {
    fn dispatch(
        &self,
        _: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        self.0.fetch_add(1, Ordering::AcqRel);
        WorthQueryExternalTransportOutcome::Completed
    }
}

#[test]
fn callback_winner_is_recognized_before_transport_world_publication() {
    let world = installed_world();
    let receipt = { world.commit_dispatch(96, "callback-winner") };
    let owner = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .observe_committed_dispatch_outbox(&phase, &receipt)
                .unwrap()
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let correlation = *owner.record().correlation();
    let admitted = world
        .application
        .admit_external_dispatch_attempt(owner.clone())
        .unwrap();
    let in_flight = world
        .application
        .primary_provider
        .begin_external_dispatch_in_flight(&owner)
        .unwrap()
        .unwrap();
    let completed =
        dispatch_external_effect(&CompletingTransport(AtomicUsize::new(0)), admitted).unwrap();
    let evidence = InstalledTransportCompletion::from_observed_dispatch(owner, &completed).unwrap();
    world
        .application
        .record_installed_transport_completion(evidence, in_flight)
        .unwrap();
    let record = receipt.dispatch_outbox().unwrap();
    let envelope = signed_envelope(record, [0x96; 32], record.payload(), false);
    let request = super::super::fixture::live_scope();
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request)
            .unwrap()
            .posture(),
        WorthQueryInboundReceiptPosture::Performed,
    );
    let before = world
        .application
        .primary_provider
        .lookup_completed_inbound(&correlation)
        .unwrap()
        .unwrap()
        .completion_world_commit()
        .clone();
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            assert_eq!(
                world.application.resume_installed_transport_completion(
                    &phase.borrowed_request_for_test(),
                    phase,
                    &correlation,
                    &request
                ),
                InstalledTransportResumeOutcome::Performed
            );
        })
        .expect("fixture owner admits its advancement");

    let after = world
        .application
        .primary_provider
        .lookup_completed_inbound(&correlation)
        .unwrap()
        .unwrap()
        .completion_world_commit()
        .clone();
    assert_eq!(before, after);
}

mod transport_publication_obeys_shared_operation_capacity;

mod publication_stop_tests;
