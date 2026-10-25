//! Same-position native replacements share their unchanged history allocation.
use crate::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;

use super::*;

#[test]
fn pinned_same_position_replacements_preserve_unchanged_history() {
    let world = installed_authorization_world(true);
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
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
                .entity_id()
        })
        .expect("the fixture read owns its installed host call");
    let (_, product, _) = selected.into_parts();
    let coordinate = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get(),
    };
    let source = SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
        output_binding: OutputBindingIdentity::declared("StatusOutput"),
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
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            let request_execution = &host_custody.borrowed_request_for_test();
            handle.with_runtime_mut(request_execution, |runtime| {
                for ordinal in 0..20 {
                    write_field(
                        request_execution,
                        runtime,
                        entity,
                        label.clone(),
                        &format!("history-{ordinal}"),
                    );
                }
                let (basis_handle, basis) = snapshot(runtime);
                let cell = owner.cell_for_read(&basis).unwrap().unwrap();

                let original = cell.read_image();
                assert!(!original.payload().past.is_empty());

                let historical_sources = original
                    .payload()
                    .past
                    .iter()
                    .map(|(position, row)| (*position, row.root_id, row.commit_id))
                    .collect::<Vec<_>>();

                let facts = Arc::from([field_fact(
                    &host_custody.borrowed_request_for_test(),
                    runtime,
                    &basis_handle,
                    entity,
                    status,
                )]);
                let mut images = vec![original];
                let mut identities = Vec::new();
                for ordinal in 0..8 {
                    let identity = RecordedSettlementIdentity::retain(&source, coordinate, ordinal);
                    register(
                        owner,
                        Arc::clone(&identity),
                        Arc::clone(&facts),
                        &basis,
                        OrdSet::new(),
                    );
                    assert!(matches!(
                        currentness(owner, &basis, &identity),
                        SourceSettlementCurrentness::Clean
                    ));
                    let image = cell.read_image();
                    assert_eq!(image.root_id(), basis.root_id());
                    assert_eq!(image.commit_id(), basis.commit_id());
                    assert_eq!(image.position(), basis.position());

                    assert_eq!(
                        image
                            .payload()
                            .past
                            .iter()
                            .map(|(position, row)| (*position, row.root_id, row.commit_id))
                            .collect::<Vec<_>>(),
                        historical_sources
                    );
                    images.push(image);
                    identities.push(identity);
                }

                // The live cell still holds the final replacement. Releasing old native
                // image leases reclaims their objects without releasing its shared map.
                images.clear();

                let last = cell.read_image();
                write_field(request_execution, runtime, entity, label, "edited-history");
                let next = cell.read_image();
                assert_ne!(next.position(), last.position());

                assert_eq!(last.payload().past.len(), historical_sources.len());
                for identity in identities {
                    assert!(
                        matches!(
                            currentness(owner, &basis, &identity),
                            SourceSettlementCurrentness::Clean
                        ),
                        "a later publication preserves the exact retained read basis"
                    );
                }

                drop(last);

                runtime.snapshots().release_snapshot(&basis_handle).unwrap();
            })
        })
        .expect("the fixture read owns its installed host call");
}
