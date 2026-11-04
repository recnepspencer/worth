//! Native invalidation reaches every consumed edge in a four-node chain.
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;

#[test]
fn native_invalidation_marks_the_second_and_third_downstream_levels() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    world.application.with_host_advancement(|phase| {
    let execution = phase.borrowed_request_for_test();
    let selected = world.selected_product(&phase.test_host_request_for_test());
    let entity = selected
        .resolve_entity(
            &phase,
            AccountStatus::reference(),
            "open".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id_for_test();
    let (_, product, _) = selected.into_parts_for_test();
    let coordinate = ProductCoordinate::new_for_test(product.observation_for_test().lifecycle_incarnation(), product.observation_for_test().reference_generation().get());
    let graph = (*world.application.runtime_for_test()).primary_graph().unwrap();
    let handle = graph.integration_handle_for_test();
    let owner = &(*(*handle.source_owner_for_test()).invalidation_owner_for_test());
    let status_ref = AccountStatus::reference();
    let status = graph
        .layout_for_test()
        .field_locator_for_test(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let identities = [
        OutputBindingIdentity::declared_for_test("StatusOutput"),
        OutputBindingIdentity::declared_for_test("LabelOutput"),
        OutputBindingIdentity::declared_for_test("DownstreamOutput"),
        OutputBindingIdentity::declared_for_test("LateOutput"),
    ]
    .map(|output_binding| {
        RecordedSettlementIdentity::retain_for_test(
            &SemanticSource::new_for_test((*world.application.runtime_for_test()).authority_identity().as_u64(), (*world.application.installed_schema_for_test()).binding_identity(), worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity), output_binding),
            coordinate,
            0,
        )
    });
    handle.with_runtime_mut_for_test(&execution, |runtime| {
        write_field(&execution, runtime, entity, status.clone(), "prime");
        let (before_handle, before) = snapshot(runtime);
        for (level, identity) in identities.iter().enumerate() {
            let facts: Arc<[_]> = if level == 0 {
                Arc::from([field_fact(&execution, runtime, &before_handle, entity, status.clone())])
            } else {
                Arc::from([])
            };
            let upstream = level
                .checked_sub(1)
                .map(|prior| OrdSet::unit(Arc::clone(&identities[prior])))
                .unwrap_or_default();
            register(owner, Arc::clone(identity), facts, &before, upstream);
            assert!(matches!(
                currentness(owner, &before, identity),
                SourceSettlementCurrentness::Clean
            ));
        }
        write_field(&execution, runtime, entity, status, "closed");
        let (after_handle, after) = snapshot(runtime);
        assert!(matches!(
            currentness(owner, &after, &identities[0]),
            SourceSettlementCurrentness::Dirty(_)
        ));
        for level in 1..identities.len() {
            assert!(
                matches!(currentness(owner, &after, &identities[level]),
                    SourceSettlementCurrentness::PendingUpstream(edges)
                        if edges == OrdSet::unit(Arc::clone(&identities[level - 1]))),
                "upstream invalidation must propagate beyond one level: downstream level {level}"
            );
        }
        for selected in [before_handle, after_handle] {
            runtime.snapshots().release_snapshot(&selected).unwrap();
        }
    });
    }).expect("the native fixture borrows its installed host phase");
}
