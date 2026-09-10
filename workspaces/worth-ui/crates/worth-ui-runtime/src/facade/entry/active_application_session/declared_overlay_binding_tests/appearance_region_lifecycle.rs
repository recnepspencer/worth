use crate::facade::WorthUiActiveApplicationSession;
use crate::mounting::{UiMountedOccurrenceGeometryDenial as Denial, UiMountedSurfaceGeometryBatch};
use worth_ui_host_contract::{UiMountedInstanceIdentity, UiSemanticSurfaceIdentity};

pub(super) fn verify(
    session: &mut WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    surface: UiSemanticSurfaceIdentity,
    graph: crate::graph::UiGraphNodeIdentity,
    owner: UiMountedInstanceIdentity,
    region: worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
    executed: &str,
) {
    let accepted = session
        .overlay_composition_owners
        .current_for_test(surface)
        .unwrap()
        .current()
        .unwrap()
        .clone();
    let wrong_region = batch(
        session,
        surface,
        4,
        region,
        &[(owner, "not-an-executed-region")],
    );
    assert_eq!(
        session
            .begin_mounted_layout()
            .complete_surface_geometry(wrong_region),
        Err(Denial::UnknownExecutedRegion)
    );
    assert_eq!(
        session
            .mounted
            .current_region_extent(surface, session.generation_identity(), region),
        Some(super::bounds([70., 90., 320., 140.]))
    );

    let missing = batch(session, surface, 4, region, &[]);
    assert_eq!(
        session
            .begin_mounted_layout()
            .complete_surface_geometry(missing),
        Err(Denial::MissingMosaicRegionGeometry),
        "required Mosaic clip geometry is rejected at atomic layout completion"
    );
    assert_eq!(
        session
            .overlay_composition_owners
            .current_for_test(surface)
            .unwrap()
            .current()
            .unwrap(),
        &accepted
    );

    let second_decl = session
        .application
        .authored_overlay_material()
        .overlay_declaration_bindings()
        .surface_named("workspace.surface.secondary")
        .unwrap();
    let second = session
        .create_declared_semantic_surface(second_decl)
        .unwrap();
    let handle = session.mounted_graph_node(graph).unwrap();
    let foreign_owner = session.mount_instance(handle, second).unwrap();
    let foreign_region = batch(session, surface, 4, region, &[(foreign_owner, executed)]);
    assert_eq!(
        session
            .begin_mounted_layout()
            .complete_surface_geometry(foreign_region),
        Err(Denial::ForeignSurface)
    );

    let stale = batch(session, surface, 5, region, &[(owner, executed)]);
    let repeated = session.mount_instance(handle, surface).unwrap();
    assert_eq!(
        session
            .begin_mounted_layout()
            .complete_surface_geometry(stale),
        Err(Denial::StaleLayoutBasis)
    );
    let peers = (0..510)
        .map(|_| session.mount_instance(handle, surface).unwrap())
        .collect::<Vec<_>>();
    let repeated_regions = [owner, repeated]
        .into_iter()
        .chain(peers.iter().copied())
        .map(|owner| (owner, executed))
        .collect::<Vec<_>>();
    assert_eq!(repeated_regions.len(), 512);
    let duplicate = batch(session, surface, 5, region, &repeated_regions);
    let repeated_receipt = session
        .begin_mounted_layout()
        .complete_surface_geometry(duplicate)
        .unwrap();
    assert!(repeated_receipt.region_plan_rows_visited() > 0);
    assert_eq!(repeated_receipt.occurrence_index_rows(), 512);
    assert_eq!(
        repeated_receipt.occurrence_ancestry_steps(),
        1_024,
        "each root occurrence requires one indexed Mosaic and one indexed Scroll lookup"
    );
    assert_eq!(repeated_receipt.region_index_rows(), 1_024);
    assert_eq!(
        repeated_receipt.region_lookup_steps(),
        2_560,
        "each submitted region is checked once, then each required region is checked while resolving each service and translated once"
    );
    assert_eq!(
        session
            .mounted
            .current_region_extent(surface, session.generation_identity(), region),
        None,
        "a repeated region on one surface requires unambiguous occurrence evidence"
    );

    session.unmount_instance(owner).unwrap();
    for peer in peers {
        session.unmount_instance(peer).unwrap();
    }
    assert_eq!(
        session
            .mounted
            .current_region_extent(surface, session.generation_identity(), region),
        Some(super::bounds([140., 60., 240., 100.])),
        "retiring one occurrence removes its region row and restores the exact surviving occurrence"
    );
    let stale = batch(session, surface, 6, region, &[(repeated, executed)]);
    let rebound = session
        .inspect_mounted_identity()
        .surface_bindings()
        .iter()
        .find(|binding| binding.semantic_surface_identity() == surface)
        .copied()
        .unwrap();
    let replacement = session
        .rebind_host_surface(
            rebound.binding_generation(),
            rebound.presentation_mode(),
            rebound.profile(),
        )
        .unwrap();
    let replacements = [crate::mounting::UiMountedSurfaceReconciliationBinding::new(
        rebound.binding_generation(),
        replacement.binding_generation(),
    )];
    assert_eq!(
        session
            .begin_mounted_layout()
            .complete_surface_geometry(stale),
        Err(Denial::StaleLayoutBasis)
    );

    assert_eq!(
        session
            .mounted
            .current_region_extent(surface, session.generation_identity(), region),
        Some(super::bounds([140., 60., 240., 100.])),
        "surface rebind retains completed logical region geometry"
    );
    let retained_frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::exact_surfaces(vec![surface]),
            |_| {},
        )
        .unwrap_or_else(|_| {
            panic!("retained logical region geometry prepares after rebind before relayout")
        });
    drop(retained_frame);

    let repaired = batch(session, surface, 6, region, &[(repeated, executed)]);
    let single_receipt = session
        .begin_mounted_layout()
        .complete_surface_geometry(repaired)
        .unwrap();
    assert_eq!(
        repeated_receipt.region_plan_rows_visited(),
        single_receipt.region_plan_rows_visited(),
        "512 repeated mounted owners visit the same compiled subtree once, just like one owner"
    );
    assert_eq!(single_receipt.occurrence_index_rows(), 1);
    assert_eq!(single_receipt.occurrence_ancestry_steps(), 2);
    assert_eq!(single_receipt.region_index_rows(), 2);
    assert_eq!(single_receipt.region_lookup_steps(), 5);
    let frame = session
        .prepare_mounted_reconstruction_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::exact_surfaces(vec![surface]),
            &replacements,
            |_| {},
        )
        .unwrap_or_else(|_| panic!("surviving region prepares after binding repair"));
    host.push_native_display_presented();
    assert!(matches!(
        session
            .present_prepared_mounted_frame_for_reconciliation(
                frame,
                &replacements,
                worth_ui_host_contract::UiPresentationDeadline::at_tick(u64::MAX),
                2
            )
            .unwrap(),
        crate::mounting::UiMountedFrameOutcome::Reconciled(_)
    ));
    let output = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    worth_ui_host_headless::translate_unpublished_appearance_for_certification(output).unwrap();
}

fn batch(
    session: &mut WorthUiActiveApplicationSession,
    surface: UiSemanticSurfaceIdentity,
    revision: u64,
    region: worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
    regions: &[(UiMountedInstanceIdentity, &str)],
) -> UiMountedSurfaceGeometryBatch {
    let current = session.inspect_mounted_identity();
    let rows = current
        .mounted_instances()
        .iter()
        .filter(|instance| instance.basis().semantic_surface_identity() == surface)
        .enumerate()
        .map(|(index, instance)| {
            crate::mounting::UiMountedOccurrenceGeometry::surface(
                instance.identity(),
                super::bounds([8. + index as f32 * 100., 12., 28., 20.]),
            )
        })
        .collect::<Vec<_>>();
    let basis = session.begin_mounted_layout().basis(surface).unwrap();
    UiMountedSurfaceGeometryBatch::new(
        basis,
        crate::mounting::UiMountedLayoutRevision::new(revision).unwrap(),
        super::bounds([0., 0., 1280., 720.]),
        rows,
    )
    .with_regions(
        regions
            .iter()
            .map(|(owner, executed)| {
                crate::mounting::UiMountedMosaicRegionGeometry::new(
                    *owner,
                    region,
                    *executed,
                    worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
                        worth_ui_host_contract::UiMountedCanonicalBoxInput {
                            x: 32.,
                            y: 48.,
                            width: 240.,
                            height: 100.,
                            coordinate_space:
                                worth_ui_host_contract::UiMountedCoordinateSpace::GraphNodeLocal,
                        },
                    )
                    .unwrap(),
                )
            })
            .collect::<Vec<_>>(),
    )
}
