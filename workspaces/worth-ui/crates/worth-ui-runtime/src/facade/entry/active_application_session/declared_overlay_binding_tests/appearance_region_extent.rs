use super::appearance_publication_support::{
    appearance_overlay_session_with_source, close_source_with, establish_geometry, SOURCE,
};
use super::test_support::portal_target;
use worth_ui_host_contract::*;

#[path = "appearance_region_lifecycle.rs"]
mod lifecycle;

#[test]
fn authored_region_backdrop_uses_completed_occurrence_and_tracks_resize() {
    let source = SOURCE
        .replace(
            "extent surface_viewport workspace.surface.overlay",
            "extent presented_mosaic_region workspace.surface.overlay workspace.region.primary",
        )
        .replace(
            "presence while portal overlay.menu presented",
            "presence always",
        )
        .replace(
            "place immediately_before portal overlay.menu",
            "place above_surface_content",
        );
    let (mut session, host) = appearance_overlay_session_with_source(&source);
    let bindings = session
        .application
        .authored_overlay_material()
        .overlay_declaration_bindings();
    let declared = bindings.surface_named("workspace.surface.overlay").unwrap();
    let region = bindings
        .region_named("workspace.surface.overlay", "workspace.region.primary")
        .unwrap();
    let surface = session.create_declared_semantic_surface(declared).unwrap();
    session
        .register_host_surface(
            surface,
            crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
            crate::facade::mounted::UiSurfaceBindingProfile::new(
                1_000,
                crate::facade::mounted::UiSurfaceBindingCoordinatePosture::LogicalPoints,
                1,
            )
            .unwrap(),
        )
        .unwrap();
    let (graph, owner) = portal_target(&mut session, surface);
    establish_geometry(&mut session, surface);
    close_source_with(&mut session, &source);
    let executed_region = concat!(
        "component|module:app/main.wui|identity:workspace.component.overlay|",
        "regions[region(workspace.region.primary)sizing(workspace.sizing.mosaic_support)children[]mounts[]|]",
        "::region::workspace.region.primary#0",
    );
    assert_eq!(
        session
            .application
            .mounted_region_declarations(declared, graph)
            .0
            .iter()
            .find(|binding| binding.executed_region() == executed_region)
            .map(|binding| binding.declaration()),
        Some(region),
        "exact authored occurrence correspondence: {:?}",
        session
            .application
            .mounted_region_declarations(declared, graph)
    );
    for (revision, expected) in [(2, [40., 60., 240., 100.]), (3, [70., 90., 320., 140.])] {
        let current = session.inspect_mounted_identity();
        let rows = current
            .mounted_instances()
            .iter()
            .map(|instance| {
                let rect = [8., 12., 28., 20.];
                crate::mounting::UiMountedOccurrenceGeometry::surface(
                    instance.identity(),
                    bounds(rect),
                )
            })
            .collect::<Vec<_>>();
        let mut layout = session.begin_mounted_layout();
        let basis = layout.basis(surface).unwrap();
        layout
            .complete_surface_geometry(
                crate::mounting::UiMountedSurfaceGeometryBatch::new(
                    basis,
                    crate::mounting::UiMountedLayoutRevision::new(revision).unwrap(),
                    bounds([0., 0., 1280., 720.]),
                    rows,
                )
                .with_regions(vec![
                    crate::mounting::UiMountedMosaicRegionGeometry::new(
                        owner,
                        region,
                        executed_region,
                        UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
                            x: expected[0] - 8.,
                            y: expected[1] - 12.,
                            width: expected[2],
                            height: expected[3],
                            coordinate_space: UiMountedCoordinateSpace::GraphNodeLocal,
                        })
                        .unwrap(),
                    ),
                ]),
            )
            .unwrap();
        assert_eq!(
            session
                .mounted
                .current_region_extent(surface, session.generation_identity(), region),
            Some(bounds(expected))
        );
        let frame = session
            .prepare_mounted_frame_with_application_presentation(
                crate::mounting::UiMountedFrameRequest::exact_surfaces(vec![surface]),
                |_| {},
            )
            .unwrap_or_else(|_| panic!("region frame prepares"));
        if revision == 2 {
            host.push_native_display_presented();
        } else {
            host.push_native_display_settled_without_effects();
        }
        let outcome = session.present_prepared_mounted_frame_internal(
            frame,
            UiPresentationDeadline::at_tick(u64::MAX),
            0,
        );
        if let crate::mounting::UiMountedFrameOutcome::AdmissionDenied(rejection) = &outcome {
            panic!("region frame denied: {:?}", rejection.denial());
        }
        assert!(matches!(
            outcome,
            crate::mounting::UiMountedFrameOutcome::Published(_)
                | crate::mounting::UiMountedFrameOutcome::Unchanged(_)
        ));
        let output = session
            .mounted
            .current_unpublished_appearance()
            .unwrap()
            .unwrap();
        let overlay = output
            .fragments()
            .iter()
            .find(|fragment| {
                fragment.identity()
                    == UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(surface)
            })
            .unwrap();
        let backdrops = overlay
            .work()
            .successor()
            .mechanics()
            .iter()
            .filter_map(|mechanic| match mechanic {
                UiMountedAppearanceMechanic::Backdrop(row) => Some(row),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(backdrops.len(), 1);
        let extent = backdrops[0].extent();
        assert_eq!(
            (extent.x(), extent.y(), extent.width(), extent.height()),
            (
                (expected[0] * 1000.) as i32,
                (expected[1] * 1000.) as i32,
                (expected[2] * 1000.) as u32,
                (expected[3] * 1000.) as u32
            )
        );
        assert!(overlay
            .work()
            .successor()
            .mechanics()
            .iter()
            .all(|mechanic| !matches!(mechanic, UiMountedAppearanceMechanic::PortalSurface(_))));
        let damage = overlay.work().damage();
        let expected_damage = if revision == 2 {
            (40_000, 60_000, 240_000, 100_000)
        } else {
            (40_000, 60_000, 350_000, 170_000)
        };
        assert_eq!(damage.len(), 1);
        assert_eq!(
            (
                damage[0].x(),
                damage[0].y(),
                damage[0].width(),
                damage[0].height()
            ),
            expected_damage
        );
        assert_eq!(
            session
                .overlay_composition_owners
                .backdrop_work_for_test(surface)
                .unwrap()
                .roles_resolved,
            usize::from(revision == 2)
        );
        worth_ui_host_headless::translate_unpublished_appearance_for_certification(output).unwrap();
    }
    lifecycle::verify(
        &mut session,
        &host,
        surface,
        graph,
        owner,
        region,
        executed_region,
    );
    let _ = session.shutdown();
}

#[test]
fn viewport_clipped_owner_counts_plan_traversal_without_region_geometry() {
    let (mut session, _) =
        super::appearance_publication_support::appearance_overlay_session_with_region_clipping(
            SOURCE,
            crate::capability::MosaicClippingPosture::viewport_clipped(),
        );
    let declared = session
        .application
        .authored_overlay_material()
        .overlay_declaration_bindings()
        .surface_named("workspace.surface.overlay")
        .unwrap();
    let surface = session.create_declared_semantic_surface(declared).unwrap();
    session
        .register_host_surface(
            surface,
            crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
            crate::facade::mounted::UiSurfaceBindingProfile::new(
                1_000,
                crate::facade::mounted::UiSurfaceBindingCoordinatePosture::LogicalPoints,
                1,
            )
            .unwrap(),
        )
        .unwrap();
    let (_, owner) = portal_target(&mut session, surface);
    close_source_with(&mut session, SOURCE);
    let mut layout = session.begin_mounted_layout();
    let receipt = layout
        .complete_surface_geometry(crate::mounting::UiMountedSurfaceGeometryBatch::new(
            layout.basis(surface).unwrap(),
            crate::mounting::UiMountedLayoutRevision::new(1).unwrap(),
            bounds([0., 0., 640., 480.]),
            vec![crate::mounting::UiMountedOccurrenceGeometry::surface(
                owner,
                bounds([8., 12., 320., 240.]),
            )],
        ))
        .unwrap();
    assert!(receipt.region_plan_rows_visited() > 0);
    assert_eq!(receipt.region_index_rows(), 0);
    assert_eq!(receipt.region_lookup_steps(), 0);
    let _ = session.shutdown();
}

fn bounds([x, y, width, height]: [f32; 4]) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .unwrap()
}
