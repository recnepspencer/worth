//! Native invalidation reaches every consumed edge in a four-node chain.
use super::*;

#[test]
fn native_invalidation_marks_the_second_and_third_downstream_levels() {
    let world = installed_authorization_world(true);
    let selected = world.selected_product();
    let entity = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id();
    let (_, product, _) = selected.into_parts();
    let coordinate = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get(),
    };
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let status_ref = AccountStatus::reference();
    let status = graph
        .layout()
        .field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let identities = [
        OutputBindingIdentity::declared("StatusOutput"),
        OutputBindingIdentity::declared("LabelOutput"),
        OutputBindingIdentity::declared("DownstreamOutput"),
        OutputBindingIdentity::declared("LateOutput"),
    ]
    .map(|output_binding| {
        RecordedSettlementIdentity::retain(
            &SemanticSource {
                runtime_authority: world.application.runtime.authority_identity().as_u64(),
                schema: world.application.installed_schema.binding_identity(),
                scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
                output_binding,
            },
            coordinate,
            0,
        )
    });
    handle.with_runtime_mut(|runtime| {
        write_field(runtime, entity, status.clone(), "prime");
        let (before_handle, before) = snapshot(runtime);
        for (level, identity) in identities.iter().enumerate() {
            let facts: Arc<[_]> = if level == 0 {
                Arc::from([field_fact(runtime, &before_handle, entity, status.clone())])
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
        write_field(runtime, entity, status, "closed");
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
}
