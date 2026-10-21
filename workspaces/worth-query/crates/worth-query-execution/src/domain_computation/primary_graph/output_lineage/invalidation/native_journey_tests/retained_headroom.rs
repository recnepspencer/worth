//! Unchanged native rows share their retained index across source publications.
use super::*;

#[test]
fn unrelated_publications_keep_shared_upstream_and_consumer_without_hypothetical_copy() {
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let world = installed_authorization_world(true);
    world.application.with_host_advancement(|phase| {
        let custody = phase.test_host_request();
        let selected = world.selected_product(&custody);
        let entity = selected.resolve_entity(&phase, AccountStatus::reference(),
            "open".into(), &live_scope(), WorthQueryPrincipalResolutionMode::Ordinary)
            .unwrap().entity_id();
        let (_, product, _) = selected.into_parts();
        let coordinate = ProductCoordinate {
            occurrence: product.observation().lifecycle_incarnation(),
            generation: product.observation().reference_generation().get(),
        };
        let source = |binding| SemanticSource {
            runtime_authority: world.application.runtime.authority_identity().as_u64(),
            schema: world.application.installed_schema.binding_identity(),
            scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
            output_binding: OutputBindingIdentity::declared(binding),
        };
        let upstream = RecordedSettlementIdentity::retain(&source("StatusOutput"), coordinate, 0);
        let consumer = RecordedSettlementIdentity::retain(&source("DownstreamOutput"), coordinate, 0);
        let graph = world.application.runtime.primary_graph().unwrap();
        let status_ref = AccountStatus::reference();
        let status = graph.layout().field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
            .unwrap().clone();
        let label_ref = AccountLabel::reference();
        let label = graph.layout().field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field())
            .unwrap().clone();
        let handle = graph.integration_handle();
        let owner = &handle.source_owner.invalidation_owner;
        handle.with_runtime_mut(execution, |runtime| {
            let (before_handle, before) = snapshot(runtime);
            let original = field_fact(&custody.borrowed_request_for_test(), runtime,
                &before_handle, entity, status.clone());
            register(owner, upstream.clone(), Arc::from([original.clone()]), &before, OrdSet::new());
            register(owner, consumer.clone(), Arc::from([]), &before, OrdSet::unit(upstream.clone()));

            let cell = owner.cell_for_read(&before).unwrap().unwrap();
            let historical = cell.read_image();
            assert_eq!(historical.payload().current.settlements.len(), 2);
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
                assert_eq!(historical.payload().current.settlements.len(), 2);
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

#[path = "no_history_quota.rs"]
mod no_history_quota;
