//! Same-position native replacements share their unchanged history allocation.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;

use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;

#[test]
fn pinned_same_position_replacements_preserve_unchanged_history() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
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
                    AccountStatus::at("AccountStatus"),
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
    let source = SemanticSource::new_for_test((*world.application.runtime_for_test()).authority_identity_for_test().as_u64(), (*world.application.installed_schema_for_test()).binding_identity(), worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity), OutputBindingIdentity::declared_for_test("StatusOutput"));
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let handle = graph.integration_handle_for_test();
    let owner = &(*(*handle.source_owner_for_test()).invalidation_owner_for_test());
    let status_ref = AccountStatus::at("AccountStatus");
    let status = graph
        .layout_for_test()
        .field_locator_for_test(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let label_ref = AccountLabel::at("AccountLabel");
    let label = graph
        .layout_for_test()
        .field_locator_for_test(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            let request_execution = &host_custody.borrowed_request_for_test();
            handle.with_runtime_mut_for_test(request_execution, |runtime| {
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
                let cell = owner.cell_for_read_for_test(&basis).unwrap().unwrap();

                let original = cell.read_image();
                assert!(!(*original.payload().past_for_test()).is_empty());

                let historical_sources = (*original.payload().past_for_test())
                    .iter()
                    .map(|(position, row)| {
                        (
                            *position,
                            (*row.root_id_for_test()),
                            (*row.commit_id_for_test()),
                        )
                    })
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
                    let identity =
                        RecordedSettlementIdentity::retain_for_test(&source, coordinate, ordinal);
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
                        (*image.payload().past_for_test())
                            .iter()
                            .map(|(position, row)| (
                                *position,
                                (*row.root_id_for_test()),
                                (*row.commit_id_for_test())
                            ))
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

                assert_eq!(
                    (*last.payload().past_for_test()).len(),
                    historical_sources.len()
                );
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
