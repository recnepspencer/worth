use std::num::NonZeroUsize;

use crate::domain_computation::primary_graph::tests::inbound_admission::fixture::installed_world;
use crate::domain_computation::primary_graph::tests::inbound_admission::verifier::signed_envelope;
use worth_query_execution::facade::domain_computation::WorthQueryInboundAdmissionDenial;
use worth_query_execution::facade::domain_computation::WorthQueryInboundReceiptPosture;

#[test]
fn authenticated_callback_retries_before_acceptance_while_owner_index_is_unavailable() {
    let world = installed_world();
    let dispatched = { world.commit_dispatch(154, "unavailable-terminal-index") };
    let record = dispatched.dispatch_outbox().unwrap();
    let envelope = signed_envelope(record, [0xd4; 32], record.payload(), false);
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    world
        .application
        .primary_provider
        .destroy_completed_inbound_index_for_test();
    assert!(matches!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request),
        Err(WorthQueryInboundAdmissionDenial::RetryBeforeAcceptance)
    ));
    assert_eq!(
        world
            .application
            .observe_inbound_cost(&world.verifier)
            .unwrap()
            .accepted_occurrences(),
        0
    );
    let budget = NonZeroUsize::new(32).unwrap();
    assert!(world
        .application
        .with_host_advancement(|phase| world.application.repair_completed_inbound_index(
            &phase,
            &world.verifier,
            budget,
            budget,
            budget
        ))
        .expect("installed host call opens")
        .unwrap());
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request)
            .unwrap()
            .posture(),
        WorthQueryInboundReceiptPosture::Performed,
    );
}
