//! Synchronous transport completion converges on the World terminal owner.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use super::fixture::installed_world;
use super::verifier::signed_envelope;
use crate::domain_computation::application_aftermath::{
    dispatch_external_effect, WorthQueryExternalDispatchRequest, WorthQueryExternalEffectTransport,
    WorthQueryExternalTransportOutcome,
};
use crate::domain_computation::primary_graph::{
    InstalledTransportCompletion, InstalledTransportResumeOutcome,
    WorthQueryApplicationCommitOutcome, WorthQueryInboundReceiptPosture,
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
fn synchronous_completed_and_later_signed_callback_share_one_world_terminal() {
    let world = installed_world();
    let transport = Arc::new(CompletingTransport(AtomicUsize::new(0)));
    world
        .application
        .install_external_effect_transport(transport.clone())
        .unwrap();
    let receipt = world.commit_dispatch(91, "synchronous-rail-completed");
    assert_eq!(transport.0.load(Ordering::Acquire), 1);
    let record = receipt.dispatch_outbox().expect("co-committed dispatch");
    let terminal = world
        .application
        .primary_provider
        .lookup_completed_inbound(record.correlation())
        .expect("complete derived index")
        .expect("Completed enters World terminal owner");
    assert_eq!(terminal.operation(), record.operation_slot().unwrap());
    assert!(terminal.authenticated_message_identity().is_none());
    assert_eq!(
        terminal.original_world_commit(),
        receipt.committed_product_publication().composite_commit(),
    );
    assert_ne!(
        terminal.original_world_commit(),
        terminal.completion_world_commit()
    );

    let envelope = signed_envelope(record, [0x91; 32], record.payload(), false);
    let request = super::super::fixture::live_scope();
    let callback = world
        .application
        .receive_inbound_occurrence(&world.verifier, &envelope, &request)
        .expect("exact authenticated callback sees existing terminal");
    assert_eq!(
        callback.posture(),
        WorthQueryInboundReceiptPosture::AlreadyCompleted
    );
    let retry = world.attempt_dispatch_on(
        world.application.current_world(),
        91,
        "synchronous-rail-completed",
    );
    assert!(matches!(
        retry,
        WorthQueryApplicationCommitOutcome::AlreadyCommitted(_)
    ));
    assert_eq!(transport.0.load(Ordering::Acquire), 1);
    assert_eq!(
        world
            .application
            .primary_provider
            .lookup_completed_inbound(record.correlation())
            .unwrap()
            .unwrap()
            .completion_world_commit(),
        terminal.completion_world_commit(),
    );
}

#[test]
fn transport_candidate_rejects_another_owner_and_terminal_matches_later_attempt() {
    let world = installed_world();
    let first_receipt = world.commit_dispatch(92, "first-transport-owner");
    let first_owner = world
        .application
        .observe_committed_dispatch_outbox(&first_receipt)
        .unwrap()
        .unwrap();
    let binding = world
        .application
        .resolve_installed_transport_completion_binding(&first_owner)
        .unwrap();
    let first_attempt = world
        .application
        .admit_external_dispatch_attempt(first_owner.clone())
        .unwrap();
    let later_attempt = world
        .application
        .admit_external_dispatch_attempt(first_owner.clone())
        .unwrap();
    let transport = CompletingTransport(AtomicUsize::new(0));
    let completed = dispatch_external_effect(&transport, first_attempt).unwrap();
    let later = dispatch_external_effect(&transport, later_attempt).unwrap();
    assert!(completed.matches_committed_owner(binding.runtime(), &first_owner));
    assert_ne!(
        completed.causal_ladder().attempt().identity(),
        later.causal_ladder().attempt().identity()
    );

    let other_receipt = world.commit_dispatch(93, "other-transport-owner");
    let other_owner = world
        .application
        .observe_committed_dispatch_outbox(&other_receipt)
        .unwrap()
        .unwrap();
    let other_binding = world
        .application
        .resolve_installed_transport_completion_binding(&other_owner)
        .unwrap();
    assert!(!completed.matches_committed_owner(binding.runtime(), &other_owner));
    let lease = world
        .application
        .product_runtime()
        .admit_product_occurrence(
            other_owner
                .committed_product_publication()
                .product_incarnation(),
        )
        .unwrap();
    assert!(matches!(
        world.application.primary_provider.prepare_installed_transport_completion_candidate(
            lease.relational_basis(), &other_owner, &completed, &other_binding,
        ),
        Err(crate::domain_computation::primary_graph::provider::WorthQueryInboundCompletionPreparationDenial::DispatchOwnerMismatch)
    ));

    let evidence =
        InstalledTransportCompletion::from_observed_dispatch(first_owner.clone(), &completed)
            .unwrap();
    let in_flight = world
        .application
        .primary_provider
        .begin_external_dispatch_in_flight(&first_owner)
        .unwrap()
        .unwrap();
    world
        .application
        .record_installed_transport_completion(evidence, in_flight)
        .unwrap();
    let request = super::super::fixture::live_scope();
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            assert!(matches!(
                world.application.resume_installed_transport_completion(
                    phase,
                    first_owner.record().correlation(),
                    &request
                ),
                InstalledTransportResumeOutcome::Performed,
            ));
        })
        .expect("fixture owner admits its advancement");

    let terminal = world
        .application
        .primary_provider
        .lookup_completed_inbound(first_owner.record().correlation())
        .unwrap()
        .unwrap();
    let later_evidence =
        InstalledTransportCompletion::from_observed_dispatch(first_owner, &later).unwrap();
    assert!(terminal.matches_transport_observation(&later_evidence, &binding));
}

#[test]
fn two_real_completed_attempts_share_the_first_world_terminal() {
    let world = installed_world();
    let receipt = world.commit_dispatch(94, "two-physical-completions");
    let owner = world
        .application
        .observe_committed_dispatch_outbox(&receipt)
        .unwrap()
        .unwrap();
    let first_attempt = world
        .application
        .admit_external_dispatch_attempt(owner.clone())
        .unwrap();
    let second_attempt = world
        .application
        .admit_external_dispatch_attempt(owner.clone())
        .unwrap();
    let first_lease = world
        .application
        .primary_provider
        .begin_external_dispatch_in_flight(&owner)
        .unwrap()
        .unwrap();
    let second_lease = world
        .application
        .primary_provider
        .begin_external_dispatch_in_flight(&owner)
        .unwrap()
        .unwrap();
    let transport = CompletingTransport(AtomicUsize::new(0));
    let first = dispatch_external_effect(&transport, first_attempt).unwrap();
    let later = dispatch_external_effect(&transport, second_attempt).unwrap();
    assert_eq!(transport.0.load(Ordering::Acquire), 2);
    assert_ne!(
        first.causal_ladder().attempt().identity(),
        later.causal_ladder().attempt().identity(),
    );
    assert_ne!(
        first.causal_ladder().observation().unwrap().identity(),
        later.causal_ladder().observation().unwrap().identity(),
    );

    let first_evidence =
        InstalledTransportCompletion::from_observed_dispatch(owner.clone(), &first).unwrap();
    world
        .application
        .record_installed_transport_completion(first_evidence, first_lease)
        .unwrap();
    let request = super::super::fixture::live_scope();
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            assert!(matches!(
                world.application.resume_installed_transport_completion(
                    phase,
                    owner.record().correlation(),
                    &request
                ),
                InstalledTransportResumeOutcome::Performed,
            ));
        })
        .expect("fixture owner admits its advancement");

    let winner = world
        .application
        .primary_provider
        .lookup_completed_inbound(owner.record().correlation())
        .unwrap()
        .unwrap();

    assert_eq!(
        world
            .application
            .primary_provider
            .begin_external_dispatch_in_flight(&owner)
            .err(),
        Some(crate::domain_computation::primary_graph::provider::WorthQueryOutstandingInFlightDenial::TerminalReached),
        "a physical attempt after the World terminal names completion, not a mismatch"
    );
    let late_evidence =
        InstalledTransportCompletion::from_observed_dispatch(owner.clone(), &later).unwrap();
    world
        .application
        .record_installed_transport_completion(late_evidence, second_lease)
        .unwrap();
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            assert!(matches!(
                world.application.resume_installed_transport_completion(
                    phase,
                    owner.record().correlation(),
                    &request
                ),
                InstalledTransportResumeOutcome::Performed,
            ));
        })
        .expect("fixture owner admits its advancement");

    assert!(!world
        .application
        .has_retained_installed_transport_completion(owner.record().correlation()));
    let after = world
        .application
        .primary_provider
        .lookup_completed_inbound(owner.record().correlation())
        .unwrap()
        .unwrap();
    assert_eq!(
        after, winner,
        "the later attempt cannot replace winner provenance"
    );
    assert_eq!(transport.0.load(Ordering::Acquire), 2);
}
