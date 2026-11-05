//! The actual lineage owner, not recorded coordinates, supplies output custody.
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::own_write_fixture::with_committed_own_write;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::RetentionOutputs;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::recorded_receipt::read_original;

#[test]
fn original_output_reuses_retained_owner_custody_after_derived_receipt_index_disposal() {
    with_committed_own_write(|world, fresh, _, _| {
        let original = read_original(world, &fresh);
        world
            .application
            .runtime_for_test()
            .primary_provider_for_test()
            .discard_completed_evidence_for_test();
        world
            .application
            .with_host_advancement(|phase| {
                let current = world.selected_product(&phase.test_host_request_for_test());
                let lineage = world
                    .application
                    .runtime_for_test()
                    .primary_provider_for_test()
                    .graph_for_test()
                    .output_lineage
                    .lock()
                    .unwrap();
                let resolved = lineage
                    .resolve_original_publication_for_test::<RetentionOutputs>(
                        &phase.borrowed_request_for_test(),
                        fresh.principal_scope(),
                        current.product().observation(),
                        fresh.idempotency_binding(),
                        &original,
                    )
                    .unwrap()
                    .expect("actual original output custody survives index disposal");
                assert_eq!(
                    resolved.idempotency_key_identity_for_test(),
                    *original.idempotency_key_identity()
                );
                assert_eq!(resolved.exact_settlement_for_test().address_for_test().2, 0);
                assert!(std::ptr::eq(
                    resolved.output_correspondence_for_test().as_ref(),
                    fresh.output_correspondence_for_test()
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
                    let current = world.selected_product(&phase.test_host_request_for_test());
                    assert!(world
                        .application
                        .runtime_for_test()
                        .primary_provider_for_test()
                        .graph_for_test()
                        .output_lineage
                        .lock()
                        .unwrap()
                        .resolve_original_publication_for_test::<RetentionOutputs>(
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
