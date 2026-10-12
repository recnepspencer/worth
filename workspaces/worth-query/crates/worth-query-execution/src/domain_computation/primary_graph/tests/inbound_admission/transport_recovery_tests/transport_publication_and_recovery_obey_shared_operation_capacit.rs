//! transport publication and recovery obey shared operation capacity.
use super::*;

#[test]
fn transport_publication_and_recovery_obey_shared_operation_capacity() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

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
    let request = super::super::super::fixture::live_scope();
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
        .fail_next_durable_append_for_test(request_execution);
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

    assert_eq!(transport.0.load(Ordering::Acquire), 1);
}
