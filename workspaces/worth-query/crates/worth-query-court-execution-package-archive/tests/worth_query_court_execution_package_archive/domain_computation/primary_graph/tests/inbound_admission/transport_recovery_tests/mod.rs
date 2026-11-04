//! Recorded transport completion publishes once, whoever reaches World first.

pub use std::sync::atomic::AtomicUsize;
pub use std::sync::atomic::Ordering;

pub use crate::domain_computation::primary_graph::tests::inbound_admission::fixture::installed_world;
pub use crate::domain_computation::primary_graph::tests::inbound_admission::verifier::signed_envelope;
pub use worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::dispatch::dispatch_external_effect;
pub use worth_query_execution::facade::domain_computation::WorthQueryExternalDispatchRequest;
pub use worth_query_execution::facade::domain_computation::WorthQueryExternalEffectTransport;
pub use worth_query_execution::facade::domain_computation::WorthQueryExternalTransportOutcome;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_runtime::inbound_publication::custody::InstalledTransportPendingReason;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_runtime::inbound_publication::installed_transport::InstalledTransportCompletion;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_runtime::inbound_publication::custody::InstalledTransportResumeOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryInboundReceiptPosture;

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
        .admit_external_dispatch_attempt_for_test(owner.clone())
        .unwrap();
    let in_flight = (*(*world
        .application
        .runtime_for_test())
        .primary_provider_for_test())
        .begin_external_dispatch_in_flight_for_test(&owner)
        .unwrap()
        .unwrap();
    let completed =
        dispatch_external_effect(&CompletingTransport(AtomicUsize::new(0)), admitted).unwrap();
    let evidence =
        InstalledTransportCompletion::from_observed_dispatch_for_test(owner, &completed).unwrap();
    world
        .application
        .record_installed_transport_completion_for_test(evidence, in_flight)
        .unwrap();
    let record = receipt.dispatch_outbox().unwrap();
    let envelope = signed_envelope(record, [0x96; 32], record.payload(), false);
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request)
            .unwrap()
            .posture(),
        WorthQueryInboundReceiptPosture::Performed,
    );
    let before = (*(*world
        .application
        .runtime_for_test())
        .primary_provider_for_test())
        .lookup_completed_inbound_for_test(&correlation)
        .unwrap()
        .unwrap()
        .completion_world_commit_for_test()
        .clone();
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            assert_eq!(
                world
                    .application
                    .resume_installed_transport_completion_for_test(
                        &phase.borrowed_request_for_test(),
                        phase,
                        &correlation,
                        &request
                    ),
                InstalledTransportResumeOutcome::Performed
            );
        })
        .expect("fixture owner admits its advancement");

    let after = (*(*world
        .application
        .runtime_for_test())
        .primary_provider_for_test())
        .lookup_completed_inbound_for_test(&correlation)
        .unwrap()
        .unwrap()
        .completion_world_commit_for_test()
        .clone();
    assert_eq!(before, after);
}

#[path = "publication_stop_tests/mod.rs"]
pub mod publication_stop_tests;
#[path = "transport_publication_obeys_shared_operation_capacity/mod.rs"]
pub mod transport_publication_obeys_shared_operation_capacity;
