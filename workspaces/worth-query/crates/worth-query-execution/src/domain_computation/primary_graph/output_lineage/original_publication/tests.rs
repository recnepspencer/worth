//! The actual lineage owner, not recorded coordinates, supplies output custody.
use super::*;
use crate::domain_computation::primary_graph::{
    output_lineage::own_write_fixture::with_committed_own_write,
    tests::{
        application_attempt::preimage_evidence::RetentionOutputs,
        fixture::recorded_receipt::read_original,
    },
};

#[test]
fn original_output_reuses_retained_owner_custody_after_derived_receipt_index_disposal() {
    with_committed_own_write(|world, fresh, _, _| {
        let original = read_original(world, &fresh);
        world
            .application
            .primary_provider
            .discard_completed_evidence_for_test();
        world
            .application
            .with_host_advancement(|phase| {
                let current = world.selected_product(&phase.test_host_request());
                let lineage = world
                    .application
                    .primary_provider
                    .graph
                    .output_lineage
                    .lock()
                    .unwrap();
                let resolved = lineage
                    .resolve_original_publication::<RetentionOutputs>(
                        &phase.borrowed_request_for_test(),
                        fresh.principal_scope(),
                        current.product().observation(),
                        fresh.idempotency_binding(),
                        &original,
                    )
                    .unwrap()
                    .expect("actual original output custody survives index disposal");
                assert_eq!(
                    resolved.idempotency_key_identity(),
                    *original.idempotency_key_identity()
                );
                assert_eq!(resolved.exact_settlement().address().2, 0);
                assert!(std::ptr::eq(
                    resolved.output_correspondence().as_ref(),
                    fresh.output_correspondence()
                ));
            })
            .unwrap();
    });
}

#[test]
fn original_receipt_from_another_world_cannot_resolve_this_owners_output() {
    with_committed_own_write(|first_world, first, _, _| {
        let foreign = read_original(first_world, &first);
        with_committed_own_write(|world, fresh, _, _| {
            world
                .application
                .with_host_advancement(|phase| {
                    let current = world.selected_product(&phase.test_host_request());
                    assert!(world
                        .application
                        .primary_provider
                        .graph
                        .output_lineage
                        .lock()
                        .unwrap()
                        .resolve_original_publication::<RetentionOutputs>(
                            &phase.borrowed_request_for_test(),
                            fresh.principal_scope(),
                            current.product().observation(),
                            fresh.idempotency_binding(),
                            &foreign
                        )
                        .unwrap()
                        .is_none());
                })
                .unwrap();
        });
    });
}
