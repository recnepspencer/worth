//! Actual completed attempt held across a World unpublished transition.

use std::num::NonZeroUsize;
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
fn completed_transport_recovery_publishes_without_a_second_physical_attempt() {
    let world = installed_world();
    let receipt = world.commit_dispatch(95, "unpublished-transport");
    let owner = world
        .application
        .observe_committed_dispatch_outbox(&receipt)
        .unwrap()
        .unwrap();
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
    let transport = CompletingTransport(AtomicUsize::new(0));
    let completed = dispatch_external_effect(&transport, admitted).unwrap();
    let evidence = InstalledTransportCompletion::from_observed_dispatch(owner, &completed).unwrap();
    world
        .application
        .record_installed_transport_completion(evidence, in_flight)
        .unwrap();
    let request = super::super::fixture::live_scope();
    world.application.fail_next_durable_append_for_test();
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            assert_eq!(
                world.application.resume_installed_transport_completion(
                    phase,
                    &correlation,
                    &request
                ),
                InstalledTransportResumeOutcome::Pending(
                    InstalledTransportPendingReason::ProductRecoveryRequired
                ),
            );
        })
        .expect("fixture owner admits its advancement");

    assert_eq!(transport.0.load(Ordering::Acquire), 1);
    assert!(world
        .application
        .has_retained_installed_transport_completion(&correlation));
    assert!(world
        .application
        .primary_provider
        .lookup_completed_inbound(&correlation)
        .is_err());
    assert!(
        world
            .application
            .resolve_guarded_workflow_external_settlement(&receipt)
            .is_err(),
        "transport Completed before World settlement grants no workflow completion"
    );
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            assert_eq!(
                world.application.resume_installed_transport_completion(
                    phase,
                    &correlation,
                    &request
                ),
                InstalledTransportResumeOutcome::Performed,
            );
        })
        .expect("fixture owner admits its advancement");

    assert_eq!(transport.0.load(Ordering::Acquire), 1);
    assert!(world
        .application
        .primary_provider
        .lookup_completed_inbound(&correlation)
        .unwrap()
        .is_some());
    assert!(world
        .application
        .resolve_guarded_workflow_external_settlement(&receipt)
        .expect("World-performed terminal settles the exact operation"));
}

#[test]
fn host_maintenance_recovers_unpublished_transport_after_request_is_gone() {
    let world = installed_world();
    let receipt = world.commit_dispatch(0xa1, "transport-host-continuation");
    let owner = world
        .application
        .observe_committed_dispatch_outbox(&receipt)
        .unwrap()
        .unwrap();
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
    let transport = CompletingTransport(AtomicUsize::new(0));
    let completed = dispatch_external_effect(&transport, admitted).unwrap();
    world
        .application
        .record_installed_transport_completion(
            InstalledTransportCompletion::from_observed_dispatch(owner, &completed).unwrap(),
            in_flight,
        )
        .unwrap();
    world.application.fail_next_durable_append_for_test();
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            {
                let original_request = super::super::fixture::live_scope();
                assert_eq!(
                    world.application.resume_installed_transport_completion(
                        phase,
                        &correlation,
                        &original_request
                    ),
                    InstalledTransportResumeOutcome::Pending(
                        InstalledTransportPendingReason::ProductRecoveryRequired
                    ),
                );
            }
            assert_eq!(transport.0.load(Ordering::Acquire), 1);
        })
        .expect("fixture owner admits its advancement");

    assert!(world
        .application
        .primary_provider
        .lookup_completed_inbound(&correlation)
        .is_err());

    let maintenance_request = super::super::fixture::live_scope();
    let report = world
        .application
        .maintain_inbound_occurrences(
            &world.verifier,
            NonZeroUsize::new(1).unwrap(),
            &maintenance_request,
        )
        .unwrap();
    assert_eq!(report.available_before(), 1);
    assert_eq!(report.selected(), 1);
    assert_eq!(report.performed(), 1);
    assert_eq!(report.remaining(), 0);
    assert_eq!(transport.0.load(Ordering::Acquire), 1);
    assert!(!world
        .application
        .has_retained_installed_transport_completion(&correlation));
    assert!(world
        .application
        .primary_provider
        .lookup_completed_inbound(&correlation)
        .unwrap()
        .is_some());
}

#[test]
fn callback_winner_is_recognized_before_transport_world_publication() {
    let world = installed_world();
    let receipt = world.commit_dispatch(96, "callback-winner");
    let owner = world
        .application
        .observe_committed_dispatch_outbox(&receipt)
        .unwrap()
        .unwrap();
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

#[test]
fn transport_publication_and_recovery_obey_shared_operation_capacity() {
    let world = installed_world();
    let receipt = world.commit_dispatch(99, "transport-publication-capacity");
    let owner = world
        .application
        .observe_committed_dispatch_outbox(&receipt)
        .unwrap()
        .unwrap();
    let correlation = *owner.record().correlation();
    let binding = world
        .application
        .resolve_installed_transport_completion_binding(&owner)
        .unwrap();
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
    let transport = CompletingTransport(AtomicUsize::new(0));
    let completed = dispatch_external_effect(&transport, admitted).unwrap();
    world
        .application
        .record_installed_transport_completion(
            InstalledTransportCompletion::from_observed_dispatch(owner, &completed).unwrap(),
            in_flight,
        )
        .unwrap();
    let request = super::super::fixture::live_scope();
    let held = world
        .application
        .reserve_installed_transport_publication(&binding)
        .unwrap();
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            assert_eq!(
                world.application.resume_installed_transport_completion(
                    phase,
                    &correlation,
                    &request
                ),
                InstalledTransportResumeOutcome::Pending(
                    InstalledTransportPendingReason::PublicationAtCapacity
                )
            );
        })
        .expect("fixture owner admits its advancement");

    drop(held);
    world.application.fail_next_durable_append_for_test();
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            assert_eq!(
                world.application.resume_installed_transport_completion(
                    phase,
                    &correlation,
                    &request
                ),
                InstalledTransportResumeOutcome::Pending(
                    InstalledTransportPendingReason::ProductRecoveryRequired
                )
            );
        })
        .expect("fixture owner admits its advancement");

    let held = world
        .application
        .reserve_installed_transport_publication(&binding)
        .unwrap();
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            assert_eq!(
                world.application.resume_installed_transport_completion(
                    phase,
                    &correlation,
                    &request
                ),
                InstalledTransportResumeOutcome::Pending(
                    InstalledTransportPendingReason::PublicationAtCapacity
                )
            );
        })
        .expect("fixture owner admits its advancement");

    drop(held);
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            assert_eq!(
                world.application.resume_installed_transport_completion(
                    phase,
                    &correlation,
                    &request
                ),
                InstalledTransportResumeOutcome::Performed
            );
        })
        .expect("fixture owner admits its advancement");

    assert_eq!(transport.0.load(Ordering::Acquire), 1);
}
