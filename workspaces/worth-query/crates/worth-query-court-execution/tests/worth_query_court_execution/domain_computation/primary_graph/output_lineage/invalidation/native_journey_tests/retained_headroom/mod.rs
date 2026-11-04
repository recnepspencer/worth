//! Unchanged native rows share their retained index across source publications.
pub use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;

#[test]
fn unrelated_publications_keep_shared_upstream_and_consumer_without_hypothetical_copy() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    world.application.with_host_advancement(|phase| {
        let custody = phase.test_host_request_for_test();
        let execution = &custody.borrowed_request_for_test();
        let selected = world.selected_product(&custody);
        let entity = selected.resolve_entity(&phase, AccountStatus::reference(),
            "open".into(), &live_scope(), WorthQueryPrincipalResolutionMode::Ordinary)
            .unwrap().entity_id_for_test();
        let (_, product, _) = selected.into_parts_for_test();
        let coordinate = ProductCoordinate::new_for_test(product.observation_for_test().lifecycle_incarnation(), product.observation_for_test().reference_generation().get());
        let source = |binding| SemanticSource::new_for_test((*world.application.runtime_for_test()).authority_identity_for_test().as_u64(), (*world.application.installed_schema_for_test()).binding_identity(), worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity), OutputBindingIdentity::declared_for_test(binding));
        let upstream = RecordedSettlementIdentity::retain_for_test(&source("StatusOutput"), coordinate, 0);
        let consumer = RecordedSettlementIdentity::retain_for_test(&source("DownstreamOutput"), coordinate, 0);
        let graph = (*world.application.runtime_for_test()).primary_graph_for_test().unwrap();
        let status_ref = AccountStatus::reference();
        let status = graph.layout_for_test().field_locator_for_test(status_ref.entity(), status_ref.aspect(), status_ref.field())
            .unwrap().clone();
        let label_ref = AccountLabel::reference();
        let label = graph.layout_for_test().field_locator_for_test(label_ref.entity(), label_ref.aspect(), label_ref.field())
            .unwrap().clone();
        let handle = graph.integration_handle_for_test();
        let owner = &(*(*handle.source_owner_for_test()).invalidation_owner_for_test());
        handle.with_runtime_mut_for_test(execution, |runtime| {
            let (before_handle, before) = snapshot(runtime);
            let original = field_fact(&custody.borrowed_request_for_test(), runtime,
                &before_handle, entity, status.clone());
            register(owner, upstream.clone(), Arc::from([original.clone()]), &before, OrdSet::new());
            register(owner, consumer.clone(), Arc::from([]), &before, OrdSet::unit(upstream.clone()));

            let cell = owner.cell_for_read_for_test(&before).unwrap().unwrap();
            let historical = cell.read_image();
            assert_eq!((*(*historical.payload().current_for_test()).settlements_for_test()).len(), 2);
            for value in ["unrelated-a", "unrelated-b"] {
                write_field(execution, runtime, entity, label.clone(), value);
                let (after_handle, after) = snapshot(runtime);


                let upstream_currentness = currentness(owner, &after, &upstream);
                let consumer_currentness = currentness(owner, &after, &consumer);
                if let SourceSettlementCurrentness::FullVerificationRequired(reason) = &upstream_currentness {
                    eprintln!("upstream full-verification reason: {reason:?}");
                }
                assert!(matches!(upstream_currentness, SourceSettlementCurrentness::Clean),
                    "unrelated publication lost the genuine upstream registration");
                assert!(matches!(consumer_currentness, SourceSettlementCurrentness::Clean),
                    "unrelated publication lost the genuine consumer registration");
                assert_eq!((*(*historical.payload().current_for_test()).settlements_for_test()).len(), 2);
                assert_eq!(field_fact(&custody.borrowed_request_for_test(), runtime,
                    &before_handle, entity, status.clone()), original);
                runtime.snapshots().release_snapshot(&after_handle).unwrap();
            }
            write_field(execution, runtime, entity, status.clone(), "closed");
            let (changed_handle, changed) = snapshot(runtime);
            assert!(matches!(currentness(owner, &changed, &upstream), SourceSettlementCurrentness::Dirty(_)));
            assert!(matches!(currentness(owner, &changed, &consumer),
                SourceSettlementCurrentness::PendingUpstream(ref pending) if pending == &OrdSet::unit(upstream)));
            assert_ne!(field_fact(&custody.borrowed_request_for_test(), runtime,
                &changed_handle, entity, status), original);
            for handle in [before_handle, changed_handle] {
                runtime.snapshots().release_snapshot(&handle).unwrap();
            }
        });
    }).expect("the native court owns its installed host call");
}

#[path = "no_history_quota/mod.rs"]
pub mod no_history_quota;
