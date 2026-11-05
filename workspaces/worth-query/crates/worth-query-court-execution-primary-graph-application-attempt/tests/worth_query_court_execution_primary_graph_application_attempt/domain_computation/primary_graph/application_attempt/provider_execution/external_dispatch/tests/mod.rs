use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use worth_query_execution::facade::domain_computation::WorthQueryExternalDispatchPreparationDenial;
use worth_query_execution::facade::domain_computation::WorthQueryExternalEffectTransport;
use crate::domain_computation::primary_graph::application_attempt::provider_execution::external_dispatch::tests;
use worth_query_execution::facade::domain_computation::WorthQueryExternalDispatchRequest;
use worth_query_execution::facade::domain_computation::WorthQueryExternalTransportOutcome;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::recoverable_application_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;

pub struct RetryTransport(AtomicUsize);

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
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let phase = &active_phase;

        let transport = RetryTransport(AtomicUsize::new(0));
        let original_observation = world
            .application
            .observe_committed_dispatch_outbox(
                &active_phase.phase_for_application_for_test(&world.application),
                &receipt,
            )
            .unwrap()
            .unwrap();
        let retry_observation = original_observation.clone();
        assert_eq!(
            original_observation.record().correlation(),
            retry_observation.record().correlation()
        );
        let original = world
            .application
            .perform_committed_external_dispatch_for_test(
                &phase.borrowed_request_for_test(),
                &phase.phase_for_application_for_test(&world.application),
                &transport,
                original_observation,
                &live_scope(),
                |_| {},
            )
            .expect("original dispatch");
        let retry = world
            .application
            .perform_committed_external_dispatch_for_test(
                &phase.borrowed_request_for_test(),
                &phase.phase_for_application_for_test(&world.application),
                &transport,
                retry_observation,
                &live_scope(),
                |_| {},
            )
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
    });
}

#[test]
fn foreign_owner_observation_denies_before_transport_and_preserves_cause() {
    let (world, local_receipt) = recoverable_application_world(184, "local-dispatch");
    let (foreign_world, foreign_receipt) = recoverable_application_world(182, "foreign-dispatch");
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let phase = &active_phase;

        let transport = RetryTransport(AtomicUsize::new(0));
        let local = world
            .application
            .observe_committed_dispatch_outbox(
                &active_phase.phase_for_application_for_test(&world.application),
                &local_receipt,
            )
            .unwrap()
            .unwrap();
        let foreign = foreign_world
            .application
            .observe_committed_dispatch_outbox(
                &active_phase.phase_for_application_for_test(&foreign_world.application),
                &foreign_receipt,
            )
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
            world.application.perform_committed_external_dispatch_for_test(
                &phase.borrowed_request_for_test(),
                &phase.phase_for_application_for_test(&world.application),
                &transport,
                foreign,
                &live_scope(),
                |_| {}
            ),
            Err(
                WorthQueryExternalDispatchPreparationDenial::AttemptAdmissionDenied(
                    worth_query_execution::facade::domain_computation::WorthQueryExternalDispatchAttemptDenial::ForeignProductWorld
                )
            )
        );
        assert_eq!(transport.0.load(Ordering::Acquire), 0);
    });
}

#[test]
fn unavailable_runtime_time_is_a_typed_dispatch_preparation_denial() {
    let (world, receipt) = recoverable_application_world(183, "unavailable-time");
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let phase = &active_phase;

        world.authorization_time.script([]);
        let transport = RetryTransport(AtomicUsize::new(0));
        let observation = world
            .application
            .observe_committed_dispatch_outbox(
                &active_phase.phase_for_application_for_test(&world.application),
                &receipt,
            )
            .unwrap()
            .unwrap();

        assert_eq!(
            world.application.perform_committed_external_dispatch_for_test(
                &phase.borrowed_request_for_test(),
                &phase.phase_for_application_for_test(&world.application),
                &transport,
                observation,
                &live_scope(),
                |_| {}
            ),
            Err(WorthQueryExternalDispatchPreparationDenial::TimeObservationDenied)
        );
        assert_eq!(transport.0.load(Ordering::Acquire), 0);
    });
}
