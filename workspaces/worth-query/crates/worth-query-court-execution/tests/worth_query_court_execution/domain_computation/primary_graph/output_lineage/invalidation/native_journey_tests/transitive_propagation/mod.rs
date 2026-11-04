//! Native invalidation reaches every consumed edge in a four-node chain.
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;

#[test]
fn native_invalidation_marks_the_second_and_third_downstream_levels() {
    let world = installed_authorization_world(true);
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
    let coordinate = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get(),
    };
    let graph = (*world.application.runtime_for_test()).primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let status_ref = AccountStatus::reference();
    let status = graph
        .layout()
        .field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
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
            &SemanticSource {
                runtime_authority: (*world.application.runtime_for_test()).authority_identity().as_u64(),
                schema: (*world.application.installed_schema_for_test()).binding_identity(),
                scope: worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity),
                output_binding,
            },
            coordinate,
            0,
        )
    });
    handle.with_runtime_mut(&execution, |runtime| {
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
