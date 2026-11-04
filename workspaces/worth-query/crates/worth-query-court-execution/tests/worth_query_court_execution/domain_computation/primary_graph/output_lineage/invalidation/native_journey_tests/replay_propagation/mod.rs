//! A late re-registration that replays a change must reach consumers that were
//! registered against the earlier, current row.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;

use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;

#[test]
fn retained_re_registration_marks_existing_consumers_pending() {
    let world = installed_authorization_world(true);
    let selected = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    let entity = world
        .application
        .with_host_advancement(|phase| {
            selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id_for_test()
        })
        .expect("the fixture read owns its installed host call");
    let (_, product, _) = selected.into_parts_for_test();
    let coordinate = ProductCoordinate::new_for_test(
        product.observation_for_test().lifecycle_incarnation(),
        product.observation_for_test().reference_generation().get(),
    );
    let graph = (*world.application.runtime_for_test())
        .primary_graph()
        .unwrap();
    let handle = graph.integration_handle_for_test();
    let owner = &(*(*handle.source_owner_for_test()).invalidation_owner_for_test());
    let status_ref = AccountStatus::reference();
    let status = graph
        .layout_for_test()
        .field_locator_for_test(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let source = |output_binding| {
        SemanticSource::new_for_test((*world.application.runtime_for_test()).authority_identity().as_u64(), (*world.application.installed_schema_for_test()).binding_identity().clone(), worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity), output_binding)
    };
    let upstream = RecordedSettlementIdentity::retain_for_test(
        &source(OutputBindingIdentity::declared_for_test("StatusOutput")),
        coordinate,
        0,
    );
    let consumer = RecordedSettlementIdentity::retain_for_test(
        &source(OutputBindingIdentity::declared_for_test("DownstreamOutput")),
        coordinate,
        0,
    );
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            let request_execution = &host_custody.borrowed_request_for_test();
            handle.with_runtime_mut_for_test(request_execution, |runtime| {
                write_field(request_execution, runtime, entity, status.clone(), "prime");
                let (before_handle, before) = snapshot(runtime);
                let before_facts: Arc<[_]> = Arc::from([field_fact(
                    &host_custody.borrowed_request_for_test(),
                    runtime,
                    &before_handle,
                    entity,
                    status.clone(),
                )]);
                write_field(request_execution, runtime, entity, status.clone(), "closed");
                let (after_handle, after) = snapshot(runtime);
                let after_facts: Arc<[_]> = Arc::from([field_fact(
                    &host_custody.borrowed_request_for_test(),
                    runtime,
                    &after_handle,
                    entity,
                    status,
                )]);
                register(owner, upstream.clone(), after_facts, &after, OrdSet::new());
                register(
                    owner,
                    consumer.clone(),
                    Arc::from([]),
                    &after,
                    OrdSet::unit(upstream.clone()),
                );
                assert!(matches!(
                    currentness(owner, &after, &consumer),
                    SourceSettlementCurrentness::Clean
                ));
                // The same settlement now settles from the older read; replay marks it.
                register(
                    owner,
                    upstream.clone(),
                    before_facts,
                    &before,
                    OrdSet::new(),
                );
                assert!(matches!(
                    currentness(owner, &after, &upstream),
                    SourceSettlementCurrentness::Dirty(_)
                ));
                match currentness(owner, &after, &consumer) {
                    SourceSettlementCurrentness::PendingUpstream(pending) => {
                        assert_eq!(pending, OrdSet::unit(upstream))
                    }
                    _ => panic!("a consumer of a replayed-dirty row must not stay clean"),
                }
                for snapshot in [before_handle, after_handle] {
                    runtime.snapshots().release_snapshot(&snapshot).unwrap();
                }
            })
        })
        .expect("the fixture read owns its installed host call");
}
