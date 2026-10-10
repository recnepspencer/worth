//! Equal field writes preserve native stamps and the installed actor's marks.

use super::*;

#[test]
fn equal_native_field_write_keeps_revision_and_downstream_clean_until_a_real_change() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let selected = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.selected_product(&(phase).test_host_request())
        })
        .expect("the fixture read owns its installed host call");
    let entity = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id()
        })
        .expect("the fixture read owns its installed host call");
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
    let label_ref = AccountLabel::reference();
    let label = graph
        .layout()
        .field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    let source = |output_binding| {
        SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
        output_binding,
    }
    };
    let output =
        RecordedSettlementIdentity::retain(&source(TypeId::of::<StatusOutput>()), coordinate, 0);
    let downstream = RecordedSettlementIdentity::retain(
        &source(TypeId::of::<DownstreamOutput>()),
        coordinate,
        0,
    );

    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            handle.with_runtime_mut(request_execution, |runtime| {
                write_field(runtime, entity, label, "prime");
                let (before_handle, before) = snapshot(runtime);
                let before_fact = field_fact(
                    &host_custody.borrowed_request_for_test(),
                    runtime,
                    &before_handle,
                    entity,
                    status.clone(),
                );
                register(
                    owner,
                    Arc::clone(&output),
                    Arc::from([before_fact.clone()]),
                    &before,
                    OrdSet::new(),
                );
                register(
                    owner,
                    Arc::clone(&downstream),
                    Arc::from([]),
                    &before,
                    OrdSet::unit(Arc::clone(&output)),
                );

                write_field(runtime, entity, status.clone(), "open");
                let (equal_handle, equal) = snapshot(runtime);
                let equal_fact = field_fact(
                    &host_custody.borrowed_request_for_test(),
                    runtime,
                    &equal_handle,
                    entity,
                    status.clone(),
                );
                assert_eq!(revision(&equal_fact), revision(&before_fact));
                assert!(matches!(
                    currentness(owner, &equal, &output),
                    SourceSettlementCurrentness::Clean
                ));
                assert!(matches!(
                    currentness(owner, &equal, &downstream),
                    SourceSettlementCurrentness::Clean
                ));

                write_field(runtime, entity, status.clone(), "closed");
                let (changed_handle, changed) = snapshot(runtime);
                let changed_fact = field_fact(
                    &host_custody.borrowed_request_for_test(),
                    runtime,
                    &changed_handle,
                    entity,
                    status,
                );
                assert_ne!(revision(&changed_fact), revision(&equal_fact));
                assert!(matches!(
                    currentness(owner, &changed, &output),
                    SourceSettlementCurrentness::Dirty(_)
                ));
                match currentness(owner, &changed, &downstream) {
                    SourceSettlementCurrentness::PendingUpstream(upstream) => {
                        assert_eq!(upstream, OrdSet::unit(output))
                    }
                    _ => panic!("a real native field change must mark the actual downstream"),
                }
                for snapshot in [before_handle, equal_handle, changed_handle] {
                    runtime.snapshots().release_snapshot(&snapshot).unwrap();
                }
            })
        })
        .expect("the fixture read owns its installed host call");
}

fn revision(
    fact: &WorthQueryApplicationObservedFact,
) -> worth_relational::facade::runtime::RelationalFieldRevision {
    match fact {
        WorthQueryApplicationObservedFact::SourceFieldRevision {
            native_revision, ..
        } => native_revision.expect("the actual native snapshot must issue a field revision"),
        _ => panic!("the real field reader must issue a field revision"),
    }
}
