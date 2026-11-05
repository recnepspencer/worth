//! Synchronous transport completion converges on the World terminal owner.

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::domain_computation::primary_graph::tests::inbound_admission::fixture::installed_world;
use crate::domain_computation::primary_graph::tests::inbound_admission::verifier::signed_envelope;
use worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::dispatch::dispatch_external_effect;
use worth_query_execution::facade::domain_computation::WorthQueryExternalDispatchRequest;
use worth_query_execution::facade::domain_computation::WorthQueryExternalEffectTransport;
use worth_query_execution::facade::domain_computation::WorthQueryExternalTransportOutcome;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_runtime::inbound_publication::installed_transport::InstalledTransportCompletion;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_runtime::inbound_publication::custody::InstalledTransportResumeOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryInboundReceiptPosture;

pub struct CompletingTransport(AtomicUsize);

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
    let receipt = { world.commit_dispatch(91, "synchronous-rail-completed") };
    assert_eq!(transport.0.load(Ordering::Acquire), 1);
    let record = receipt.dispatch_outbox().expect("co-committed dispatch");
    let terminal = (*(*world.application.runtime_for_test()).primary_provider_for_test())
        .lookup_completed_inbound_for_test(record.correlation())
        .expect("complete derived index")
        .expect("Completed enters World terminal owner");
    assert_eq!(
        terminal.operation_for_test(),
        record.operation_slot().unwrap()
    );
    assert!(terminal.authenticated_message_identity_for_test().is_none());
    assert_eq!(
        terminal.original_world_commit_for_test(),
        receipt.committed_product_publication().composite_commit(),
    );
    assert_ne!(
        terminal.original_world_commit_for_test(),
        terminal.completion_world_commit_for_test()
    );

    let envelope = signed_envelope(record, [0x91; 32], record.payload(), false);
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let callback = world
        .application
        .receive_inbound_occurrence(&world.verifier, &envelope, &request)
        .expect("exact authenticated callback sees existing terminal");
    assert_eq!(
        callback.posture(),
        WorthQueryInboundReceiptPosture::AlreadyCompleted
    );
    let retry = {
        world.attempt_dispatch_on(
            world.application.current_world(),
            91,
            "synchronous-rail-completed",
        )
    };
    assert!(matches!(
        retry,
        WorthQueryApplicationCommitOutcome::AlreadyCommitted(_)
    ));
    assert_eq!(transport.0.load(Ordering::Acquire), 1);
    assert_eq!(
        (*(*world.application.runtime_for_test()).primary_provider_for_test())
            .lookup_completed_inbound_for_test(record.correlation())
            .unwrap()
            .unwrap()
            .completion_world_commit_for_test(),
        terminal.completion_world_commit_for_test(),
    );
}

#[test]
fn transport_candidate_rejects_another_owner_and_terminal_matches_later_attempt() {
    let world = installed_world();
    let first_receipt = { world.commit_dispatch(92, "first-transport-owner") };
    let first_owner = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .observe_committed_dispatch_outbox(&phase, &first_receipt)
                .unwrap()
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let binding = world
        .application
        .resolve_installed_transport_completion_binding_for_test(&first_owner)
        .unwrap();
    let first_attempt = world
        .application
        .admit_external_dispatch_attempt_for_test(first_owner.clone())
        .unwrap();
    let later_attempt = world
        .application
        .admit_external_dispatch_attempt_for_test(first_owner.clone())
        .unwrap();
    let transport = CompletingTransport(AtomicUsize::new(0));
    let completed = dispatch_external_effect(&transport, first_attempt).unwrap();
    let later = dispatch_external_effect(&transport, later_attempt).unwrap();
    assert!(completed.matches_committed_owner_for_test(binding.runtime_for_test(), &first_owner));
    assert_ne!(
        completed.causal_ladder().attempt().identity(),
        later.causal_ladder().attempt().identity()
    );

    let other_receipt = { world.commit_dispatch(93, "other-transport-owner") };
    let other_owner = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .observe_committed_dispatch_outbox(&phase, &other_receipt)
                .unwrap()
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let other_binding = world
        .application
        .resolve_installed_transport_completion_binding_for_test(&other_owner)
        .unwrap();
    assert!(!completed.matches_committed_owner_for_test(binding.runtime_for_test(), &other_owner));
    let lease = world
        .application
        .product_runtime()
        .admit_product_occurrence_for_test(
            other_owner
                .committed_product_publication()
                .product_incarnation(),
        )
        .unwrap();
    world.application.with_host_advancement(|phase| { let host_custody = phase.test_host_request_for_test(); assert!(matches!(
        (*(*world.application.runtime_for_test()).primary_provider_for_test()).prepare_installed_transport_completion_candidate_for_test(&host_custody.borrowed_request_for_test(),
            lease.relational_basis_for_test(), &other_owner, &completed, &other_binding,
        ),
        Err(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::inbound_completion::WorthQueryInboundCompletionPreparationDenial::DispatchOwnerMismatch)
    )) }).expect("the fixture read owns its installed host call");

    let evidence = InstalledTransportCompletion::from_observed_dispatch_for_test(
        first_owner.clone(),
        &completed,
    )
    .unwrap();
    let in_flight = (*(*world.application.runtime_for_test()).primary_provider_for_test())
        .begin_external_dispatch_in_flight_for_test(&first_owner)
        .unwrap()
        .unwrap();
    world
        .application
        .record_installed_transport_completion_for_test(evidence, in_flight)
        .unwrap();
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            assert!(matches!(
                world
                    .application
                    .resume_installed_transport_completion_for_test(
                        &phase.borrowed_request_for_test(),
                        phase,
                        first_owner.record().correlation(),
                        &request
                    ),
                InstalledTransportResumeOutcome::Performed,
            ));
        })
        .expect("fixture owner admits its advancement");

    let terminal = (*(*world.application.runtime_for_test()).primary_provider_for_test())
        .lookup_completed_inbound_for_test(first_owner.record().correlation())
        .unwrap()
        .unwrap();
    let later_evidence =
        InstalledTransportCompletion::from_observed_dispatch_for_test(first_owner, &later).unwrap();
    assert!(terminal.matches_transport_observation_for_test(&later_evidence, &binding));
}

#[test]
fn two_real_completed_attempts_share_the_first_world_terminal() {
    let world = installed_world();
    let receipt = { world.commit_dispatch(94, "two-physical-completions") };
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
    let first_attempt = world
        .application
        .admit_external_dispatch_attempt_for_test(owner.clone())
        .unwrap();
    let second_attempt = world
        .application
        .admit_external_dispatch_attempt_for_test(owner.clone())
        .unwrap();
    let first_lease = (*(*world.application.runtime_for_test()).primary_provider_for_test())
        .begin_external_dispatch_in_flight_for_test(&owner)
        .unwrap()
        .unwrap();
    let second_lease = (*(*world.application.runtime_for_test()).primary_provider_for_test())
        .begin_external_dispatch_in_flight_for_test(&owner)
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
        InstalledTransportCompletion::from_observed_dispatch_for_test(owner.clone(), &first)
            .unwrap();
    world
        .application
        .record_installed_transport_completion_for_test(first_evidence, first_lease)
        .unwrap();
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            assert!(matches!(
                world
                    .application
                    .resume_installed_transport_completion_for_test(
                        &phase.borrowed_request_for_test(),
                        phase,
                        owner.record().correlation(),
                        &request
                    ),
                InstalledTransportResumeOutcome::Performed,
            ));
        })
        .expect("fixture owner admits its advancement");

    let winner = (*(*world.application.runtime_for_test()).primary_provider_for_test())
        .lookup_completed_inbound_for_test(owner.record().correlation())
        .unwrap()
        .unwrap();

    assert_eq!(
        (*(*world
            .application
            .runtime_for_test()).primary_provider_for_test())
            .begin_external_dispatch_in_flight_for_test(&owner)
            .err(),
        Some(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::outstanding_dispatch::in_flight::WorthQueryOutstandingInFlightDenial::TerminalReached),
        "a physical attempt after the World terminal names completion, not a mismatch"
    );
    let late_evidence =
        InstalledTransportCompletion::from_observed_dispatch_for_test(owner.clone(), &later)
            .unwrap();
    world
        .application
        .record_installed_transport_completion_for_test(late_evidence, second_lease)
        .unwrap();
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            assert!(matches!(
                world
                    .application
                    .resume_installed_transport_completion_for_test(
                        &phase.borrowed_request_for_test(),
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
        .has_retained_installed_transport_completion_for_test(owner.record().correlation()));
    let after = (*(*world.application.runtime_for_test()).primary_provider_for_test())
        .lookup_completed_inbound_for_test(owner.record().correlation())
        .unwrap()
        .unwrap();
    assert_eq!(
        after, winner,
        "the later attempt cannot replace winner provenance"
    );
    assert_eq!(transport.0.load(Ordering::Acquire), 2);
}
