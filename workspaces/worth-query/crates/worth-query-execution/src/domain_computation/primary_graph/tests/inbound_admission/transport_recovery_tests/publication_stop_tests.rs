use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementDenial as RequestDenial,
    WorthQueryManagedComputationInterruption as Interrupted,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
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
    let source = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(None, source.token());
    source.cancel();
    let result = world
        .application
        .with_host_advancement(|phase| {
            world.application.resume_installed_transport_completion(
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
        .has_retained_installed_transport_completion(&correlation));
    let live = super::super::super::fixture::live_scope();
    assert_eq!(
        world
            .application
            .with_host_advancement(
                |phase| world.application.resume_installed_transport_completion(
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
