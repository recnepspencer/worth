//! A late re-registration that replays a change must reach consumers that were
//! registered against the earlier, current row.

use super::*;

#[test]
fn retained_re_registration_marks_existing_consumers_pending() {
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
    let source = |output_binding| {
        SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity().clone(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
        output_binding,
    }
    };
    let upstream =
        RecordedSettlementIdentity::retain(&source(TypeId::of::<StatusOutput>()), coordinate, 0);
    let consumer = RecordedSettlementIdentity::retain(
        &source(TypeId::of::<DownstreamOutput>()),
        coordinate,
        0,
    );
    handle.with_runtime_mut(|runtime| {
        write_field(runtime, entity, status.clone(), "prime");
        let (before_handle, before) = snapshot(runtime);
        let before_facts: Arc<[_]> =
            Arc::from([field_fact(runtime, &before_handle, entity, status.clone())]);
        write_field(runtime, entity, status.clone(), "closed");
        let (after_handle, after) = snapshot(runtime);
        let after_facts: Arc<[_]> = Arc::from([field_fact(runtime, &after_handle, entity, status)]);
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
    });
}
