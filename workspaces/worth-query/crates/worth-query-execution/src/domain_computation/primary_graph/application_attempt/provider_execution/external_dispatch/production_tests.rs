use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;
use crate::domain_computation::application_aftermath::{
    WorthQueryExternalDispatchRequest, WorthQueryExternalTransportOutcome,
};
use crate::domain_computation::primary_graph::{
    recoverable_application_world, tests::fixture::live_scope,
};

struct RetryTransport(AtomicUsize);

impl WorthQueryExternalEffectTransport for RetryTransport {
    fn dispatch(
        &self,
        _request: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        match self.0.fetch_add(1, Ordering::AcqRel) {
            0 => WorthQueryExternalTransportOutcome::LostResponse,
            _ => WorthQueryExternalTransportOutcome::Completed,
        }
    }
}

#[test]
fn production_fresh_attempt_operation_distinguishes_safe_redispatch() {
    let (world, receipt) = recoverable_application_world(181, "dispatch-retry");
    let transport = RetryTransport(AtomicUsize::new(0));
    let original_observation = world
        .application
        .observe_committed_dispatch_outbox(&receipt)
        .unwrap()
        .unwrap();
    let retry_observation = original_observation.clone();
    assert_eq!(
        original_observation.record().correlation(),
        retry_observation.record().correlation()
    );
    let original = world
        .application
        .perform_committed_external_dispatch(&transport, original_observation, &live_scope())
        .expect("original dispatch");
    let retry = world
        .application
        .perform_committed_external_dispatch(&transport, retry_observation, &live_scope())
        .expect("safe redispatch");
    assert_eq!(
        original.causal_ladder().emission().identity(),
        retry.causal_ladder().emission().identity()
    );
    assert_ne!(
        original.causal_ladder().attempt().identity(),
        retry.causal_ladder().attempt().identity()
    );
    assert!(retry.is_external_completion());
}

#[test]
fn foreign_owner_observation_denies_before_transport_and_preserves_cause() {
    let (world, local_receipt) = recoverable_application_world(184, "local-dispatch");
    let (foreign_world, foreign_receipt) = recoverable_application_world(182, "foreign-dispatch");
    let transport = RetryTransport(AtomicUsize::new(0));
    let local = world
        .application
        .observe_committed_dispatch_outbox(&local_receipt)
        .unwrap()
        .unwrap();
    let foreign = foreign_world
        .application
        .observe_committed_dispatch_outbox(&foreign_receipt)
        .unwrap()
        .unwrap()
        .with_relational_runtime_instance_for_test(local.relational_runtime_instance_id());
    assert_ne!(
        foreign
            .committed_product_publication()
            .product_branch()
            .owner_identity(),
        local
            .committed_product_publication()
            .product_branch()
            .owner_identity()
    );

    assert_eq!(
        world
            .application
            .perform_committed_external_dispatch(&transport, foreign, &live_scope()),
        Err(WorthQueryExternalDispatchPreparationDenial::AttemptAdmissionDenied)
    );
    assert_eq!(transport.0.load(Ordering::Acquire), 0);
}

#[test]
fn unavailable_runtime_time_is_a_typed_dispatch_preparation_denial() {
    let (world, receipt) = recoverable_application_world(183, "unavailable-time");
    world.authorization_time.script([]);
    let transport = RetryTransport(AtomicUsize::new(0));
    let observation = world
        .application
        .observe_committed_dispatch_outbox(&receipt)
        .unwrap()
        .unwrap();

    assert_eq!(
        world.application.perform_committed_external_dispatch(
            &transport,
            observation,
            &live_scope()
        ),
        Err(WorthQueryExternalDispatchPreparationDenial::TimeObservationDenied)
    );
    assert_eq!(transport.0.load(Ordering::Acquire), 0);
}
