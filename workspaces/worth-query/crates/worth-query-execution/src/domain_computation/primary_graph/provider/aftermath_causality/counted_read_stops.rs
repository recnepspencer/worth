use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, publish_relational_mutation,
};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
#[test]
fn aftermath_resolution_door_preserves_stops_at_every_consult() {
    let world = installed_authorization_world(true);
    world
        .application
        .with_host_advancement(|phase| {
            let execution = phase.borrowed_request_for_test();
            let provider = &world.application.primary_provider;
            let selected = world.selected_product(&phase.test_host_request());
            let pending = WorthQueryPendingAftermathCausality::undo_of(
                selected
                    .product()
                    .relational_basis()
                    .observation()
                    .commit_receipt()
                    .unwrap()
                    .clone(),
            );
            let layout = provider.graph.layout.provider_aftermath_causality().clone();
            let identity = WorthQueryApplicationCommitOutcomeIdentity::restore(1).unwrap();
            publish_relational_mutation(
                &execution,
                &world,
                worth_relational::facade::transactions::WorkerIntentBatch::new(
                    "paid-aftermath-proof",
                )
                .push(aftermath_causality_create_intent(
                    &layout,
                    &pending,
                    identity,
                    worth_relational::facade::identity::PartitionId(1),
                )),
            );
            let selected = world.selected_product(&phase.test_host_request());
            let basis = selected.product().relational_basis();
            crate::test_read_stops::every_consult_under(
                &execution,
                |request| {
                    provider.resolve_aftermath_causality_at_basis(
                        request,
                        basis,
                        &pending,
                        Some(identity),
                    )
                },
                aftermath_read_stop(Cause::Cancelled),
                aftermath_read_stop(Cause::WorkExhausted),
            );
        })
        .unwrap();
}
