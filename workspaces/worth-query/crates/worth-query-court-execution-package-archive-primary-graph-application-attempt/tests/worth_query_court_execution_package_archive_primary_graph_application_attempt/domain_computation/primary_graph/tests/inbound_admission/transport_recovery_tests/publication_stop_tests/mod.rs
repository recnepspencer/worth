use crate::domain_computation::primary_graph::tests::inbound_admission::transport_recovery_tests::*;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial as RequestDenial;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationInterruption as Interrupted;
#[test]
fn publication_stop_transport_completion_remains_ready_without_redispatch() {
    let world = installed_world();
    let receipt = world.commit_dispatch(97, "cancelled-transport-publication");
    let owner = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .observe_committed_dispatch_outbox(&phase, &receipt)
                .unwrap()
                .unwrap()
        })
        .unwrap();
    let correlation = *owner.record().correlation();
    let admitted = world
        .application
        .admit_external_dispatch_attempt_for_test(owner.clone())
        .unwrap();
    let in_flight = (*world
        .application
        .primary_provider_for_test())
        .begin_external_dispatch_in_flight_for_test(&owner)
        .unwrap()
        .unwrap();
    let transport = CompletingTransport(AtomicUsize::new(0));
    let completed = dispatch_external_effect(&transport, admitted).unwrap();
    let evidence = InstalledTransportCompletion::from_observed_dispatch_for_test(owner, &completed).unwrap();
    world
        .application
        .record_installed_transport_completion_for_test(evidence, in_flight)
        .unwrap();
    let source = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(None, source.token());
    source.cancel();
    let result = world
        .application
        .with_host_advancement(|phase| {
            world.application.resume_installed_transport_completion_for_test(
                &phase.borrowed_request_for_test(),
                &phase,
                &correlation,
                &request,
            )
        })
        .unwrap();
    assert_eq!(
        result,
        InstalledTransportResumeOutcome::Pending(
            InstalledTransportPendingReason::ExecutionRequest(RequestDenial::Interrupted(
                Interrupted::Cancelled
            ))
        )
    );
    assert!(world
        .application
        .has_retained_installed_transport_completion_for_test(&correlation));
    let live = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    assert_eq!(
        world
            .application
            .with_host_advancement(
                |phase| world.application.resume_installed_transport_completion_for_test(
                    &phase.borrowed_request_for_test(),
                    &phase,
                    &correlation,
                    &live
                )
            )
            .unwrap(),
        InstalledTransportResumeOutcome::Performed
    );
    assert_eq!(transport.0.load(Ordering::Acquire), 1);
}
