//! The retained registration consumes the equal successor at the live image.
use super::*;

#[test]
fn retained_registration_consumes_the_equal_successor_at_the_live_image() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    with_generated_own_write(|world, _, performed, _| {
        let graph = (*world.application.runtime_for_test())
            .primary_graph_for_test()
            .unwrap();
        let handle = graph.integration_handle_for_test();
        let owner = &(*(*handle.source_owner_for_test()).invalidation_owner_for_test());
        let record = performed.get().unwrap();
        let entity = world
            .application
            .with_host_advancement(|host_phase| {
                world
                    .selected_product(&host_phase.test_host_request_for_test())
                    .resolve_entity(
                        &host_phase,
                        AccountStatus::at("AccountStatus"),
                        "2".to_owned(),
                        &live_scope(),
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap()
                    .entity_id_for_test()
            })
            .expect("installed host call opens");
        let status = AccountStatus::at("AccountStatus");
        let status = graph
            .layout_for_test()
            .field_locator_for_test(status.entity(), status.aspect(), status.field())
            .unwrap()
            .clone();
        let mut source = (*record.settlement_identity_for_test())
            .source_for_test()
            .clone();
        (*source.output_binding_mut_for_test()) =
            OutputBindingIdentity::declared_for_test("Upstream");
        let a0 = RecordedSettlementIdentity::retain_for_test(
            &source,
            (*record.settlement_identity_for_test()).coordinate_for_test(),
            0,
        );
        let a1 = RecordedSettlementIdentity::retain_for_test(
            &source,
            (*record.settlement_identity_for_test()).coordinate_for_test(),
            1,
        );
        (*source.output_binding_mut_for_test()) =
            OutputBindingIdentity::declared_for_test("Consumer");
        let consumer = RecordedSettlementIdentity::retain_for_test(
            &source,
            (*record.settlement_identity_for_test()).coordinate_for_test(),
            0,
        );
        handle.with_runtime_mut_for_test(request_execution, |runtime| {
            let (before_handle, before) = snapshot(runtime);
            register(
                owner,
                Arc::clone(&a0),
                Arc::from([field_fact(
                    request_execution,
                    runtime,
                    &before_handle,
                    entity,
                    status.clone(),
                )]),
                &before,
                OrdSet::new(),
            );
            write_field(
                request_execution,
                runtime,
                entity,
                status.clone(),
                "retained-upstream-changed",
            );
            let (live_handle, live) = snapshot(runtime);
            register_alias(
                request_execution,
                owner,
                runtime,
                &live_handle,
                &live,
                entity,
                status,
                &a0,
                &a1,
            );
            assert_ne!(
                before, live,
                "registration must exercise Retained alignment"
            );
            assert!(matches!(
                currentness(owner, &live, &a0),
                SourceSettlementCurrentness::Dirty(_)
            ));
            // No registrant facts changed; only its historical upstream edge
            // could incorrectly reconstruct Pending from the old Dirty row.
            register(
                owner,
                Arc::clone(&consumer),
                Arc::from([]),
                &before,
                OrdSet::unit(Arc::clone(&a0)),
            );
            assert!(
                matches!(
                    currentness(owner, &live, &consumer),
                    SourceSettlementCurrentness::Clean
                ),
                "retained registration must consume the certified live consequence"
            );
            runtime
                .snapshots()
                .release_snapshot(&before_handle)
                .unwrap();
            runtime.snapshots().release_snapshot(&live_handle).unwrap();
        });
    });
}
