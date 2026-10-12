//! Pre-effect cancellation reports its cause and retains accepted completion custody.
use super::{
    fixture::installed_world,
    tests::{completion_records, owner_commits, product_commit},
    verifier::signed_envelope,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementDenial as RequestDenial, WorthQueryInboundAdmissionDenial as Denial,
    WorthQueryInboundReceiptPosture as Posture,
    WorthQueryManagedComputationInterruption as Interrupted,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

#[test]
fn publication_stop_inbound_recovery_keeps_the_cancelled_request_and_custody() {
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let world = installed_world();
    let dispatch = world.commit_dispatch(71, "cancelled-inbound-recovery");
    let record = dispatch.dispatch_outbox().unwrap();
    let envelope = signed_envelope(record, [0xb1; 32], record.payload(), false);
    let live = super::super::fixture::live_scope();
    world
        .application
        .fail_next_durable_append_for_test(execution);
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &live)
            .unwrap()
            .posture(),
        Posture::AcceptedPending
    );
    let before = (
        owner_commits(execution, &world),
        product_commit(&world),
        completion_records(execution, &world),
    );
    let source = WorthQueryCancellationSource::new();
    let cancelled = WorthQueryRequestScope::new(None, source.token());
    source.cancel();
    let denial = world
        .application
        .with_host_advancement(|phase| {
            world.application.progress_retained_inbound_occurrence(
                &phase.borrowed_request_for_test(),
                &phase,
                *record.correlation().bytes(),
                &cancelled,
            )
        })
        .unwrap()
        .unwrap_err();
    assert_eq!(
        denial,
        Denial::ExecutionDenied(RequestDenial::Interrupted(Interrupted::Cancelled))
    );
    assert_eq!(
        (
            owner_commits(execution, &world),
            product_commit(&world),
            completion_records(execution, &world)
        ),
        before
    );
    assert_eq!(
        world
            .application
            .with_host_advancement(
                |phase| world.application.progress_retained_inbound_occurrence(
                    &phase.borrowed_request_for_test(),
                    &phase,
                    *record.correlation().bytes(),
                    &live
                )
            )
            .unwrap()
            .unwrap(),
        Posture::Performed
    );
    assert_eq!(owner_commits(execution, &world), before.0);
    assert_eq!(completion_records(execution, &world), before.2 + 1);
}
