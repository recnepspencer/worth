//! Transport publication obeys shared operation capacity.
use crate::domain_computation::primary_graph::tests::inbound_admission::transport_recovery_tests::*;

#[test]
fn transport_publication_obeys_shared_operation_capacity() {
    let world = installed_world();
    let receipt = { world.commit_dispatch(99, "transport-publication-capacity") };
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
    let binding = world
        .application
        .resolve_installed_transport_completion_binding_for_test(&owner)
        .unwrap();
    let admitted = world
        .application
        .admit_external_dispatch_attempt_for_test(owner.clone())
        .unwrap();
    let in_flight = (*(*world.application.runtime_for_test()).primary_provider_for_test())
        .begin_external_dispatch_in_flight_for_test(&owner)
        .unwrap()
        .unwrap();
    let transport = CompletingTransport(AtomicUsize::new(0));
    let completed = dispatch_external_effect(&transport, admitted, || {}).unwrap();
    world
        .application
        .record_installed_transport_completion_for_test(
            InstalledTransportCompletion::from_observed_dispatch_for_test(owner, &completed)
                .unwrap(),
            in_flight,
        )
        .unwrap();
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let held = world
        .application
        .reserve_installed_transport_publication_for_test(&binding)
        .unwrap();
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

    assert_eq!(transport.0.load(Ordering::Acquire), 1);
}
