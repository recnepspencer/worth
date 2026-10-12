use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, publish_relational_mutation,
};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
#[test]
fn durable_idempotency_door_preserves_stops_at_every_consult() {
    let world = installed_authorization_world(true);
    world
        .application
        .with_host_advancement(|phase| {
            let execution = phase.borrowed_request_for_test();
            let provider = &world.application.primary_provider;
            let layout = provider.graph.layout.provider_idempotency().clone();
            let binding = WorthQueryApplicationIdempotencyBinding::new([9; 32], [7; 32]);
            publish_relational_mutation(
                &execution,
                &world,
                worth_relational::facade::transactions::WorkerIntentBatch::new(
                    "paid-idempotency-proof",
                )
                .push(super::super::idempotency_create_intent(
                    &layout,
                    binding,
                    WorthQueryApplicationCommitOutcomeIdentity::restore(1).unwrap(),
                    0,
                    worth_relational::facade::identity::PartitionId(1),
                )),
            );
            let selected = world.selected_product(&phase.test_host_request());
            let snapshot = selected.application_basis().snapshot_handle();
            let affinity = WorthQueryProductIdempotencyAffinity::from_observation(
                selected.product().observation(),
            );
            provider.graph.with_runtime(&execution, |runtime| {
                crate::test_read_stops::every_consult(
                    |request| {
                        resolve_at_snapshot(
                            request, provider, runtime, snapshot, &layout, &affinity, binding,
                        )
                        .map(|answer| {
                            matches!(answer, WorthQueryProviderIdempotencyResolution::Absent)
                        })
                    },
                    idempotency_read_stop(Cause::Cancelled),
                    idempotency_read_stop(Cause::WorkExhausted),
                );
            });
        })
        .unwrap();
}
