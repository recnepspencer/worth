use super::{appearance_publication_support as support, test_support::portal_target};
use worth_ui_host_contract::{
    UiAppearanceClip, UiMountedAppearanceMechanic, UiMountedCanonicalBox,
    UiMountedCanonicalBoxInput, UiMountedCoordinateSpace, UiMountedInstanceIdentity,
    UiSemanticSurfaceIdentity,
};

#[derive(Clone, Copy)]
enum ClipCase {
    Mosaic,
    Scroll,
}

#[test]
fn mounted_geometry_carries_mosaic_clip() {
    assert_clip_case(ClipCase::Mosaic);
}

#[test]
fn mounted_geometry_carries_scroll_clip() {
    assert_clip_case(ClipCase::Scroll);
}

fn assert_clip_case(case: ClipCase) {
    use crate::capability::{MosaicClippingPosture as Mosaic, MosaicScrollOwnership as Scroll};
    let (mosaic, scroll) = match case {
        ClipCase::Mosaic => (Mosaic::clip_to_region(), Scroll::region_owned()),
        ClipCase::Scroll => (Mosaic::allow_overlay_escape(), Scroll::viewport_owned()),
    };
    let mosaic_source = matches!(case, ClipCase::Mosaic).then(|| {
        support::SOURCE
            .replace(
                "extent surface_viewport workspace.surface.overlay",
                "extent presented_mosaic_region workspace.surface.overlay workspace.region.primary",
            )
            .replace(
                "presence while portal overlay.menu presented",
                "presence always",
            )
    });
    let (mut session, _) = match case {
        ClipCase::Mosaic => support::appearance_overlay_session_with_source(
            mosaic_source.as_deref().expect("Mosaic source is present"),
        ),
        ClipCase::Scroll => {
            let region = crate::runtime::tests::source_ingress_boundary_test_support::source_backed_package_region()
                .with_clipping(mosaic)
                .with_scroll_ownership(scroll);
            support::appearance_overlay_session_with_region(region)
        }
    };
    let declaration = session
        .application
        .authored_overlay_material()
        .overlay_declaration_bindings()
        .surface_named("workspace.surface.overlay")
        .unwrap();
    let surface = session
        .create_declared_semantic_surface(declaration)
        .unwrap();
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
    let (graph, instance) = portal_target(&mut session, surface);
    support::establish_allocation(&mut session);
    install_geometry(&mut session, surface, graph, instance, case);
    match mosaic_source.as_deref() {
        Some(source) => support::close_source_with(&mut session, source),
        None => support::close_source(&mut session),
    }

    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::exact_surfaces(vec![surface]),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("the owner-completed clip must prepare"));
    let derived = frame
        .projection_rc_for_test()
        .appearance_clip_for_test(instance)
        .expect("the mounted occurrence retains its completed clip");
    let expected = match case {
        ClipCase::Mosaic => UiAppearanceClip::new(37_000, 49_000, 73_000, 41_000).unwrap(),
        ClipCase::Scroll => UiAppearanceClip::new(0, 0, 500_000, 400_000).unwrap(),
    };
    assert_eq!(
        derived,
        crate::mounting::UiMountedAppearanceClip::Ancestor(expected)
    );
    let output = frame.lower_unpublished_appearance_for_test();
    let mechanic = output
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.work().successor().mechanics())
        .find_map(|mechanic| match mechanic {
            UiMountedAppearanceMechanic::Surface(surface) => Some(surface),
            _ => None,
        })
        .expect("supported owner geometry emits the mounted surface mechanic");
    assert_eq!(mechanic.node_receipt().mounted_instance(), instance);
    assert_eq!(mechanic.clip(), expected);
    assert_eq!(
        (
            mechanic.bounds().x(),
            mechanic.bounds().y(),
            mechanic.bounds().width(),
            mechanic.bounds().height(),
        ),
        (20_000, 30_000, 200_000, 120_000),
    );
    assert_eq!(mechanic.projection().identity(), graph.digest());
    let _ = session.shutdown();
}

fn install_geometry(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    surface: UiSemanticSurfaceIdentity,
    graph: crate::graph::UiGraphNodeIdentity,
    instance: UiMountedInstanceIdentity,
    case: ClipCase,
) {
    let bindings = session
        .application
        .authored_overlay_material()
        .overlay_declaration_bindings();
    let surface_declaration = bindings.surface_named("workspace.surface.overlay").unwrap();
    let viewport = canonical(
        [0.0, 0.0, 500.0, 400.0],
        UiMountedCoordinateSpace::HostSurface,
    );
    let occurrence = [20.0, 30.0, 200.0, 120.0];
    let regions = if matches!(case, ClipCase::Mosaic) {
        let owner_regions = session
            .application
            .mounted_region_declarations(surface_declaration, graph)
            .0;
        let binding = owner_regions
            .iter()
            .next()
            .expect("the executed plan binds the authored region occurrence");
        assert_eq!(owner_regions.len(), 1);
        vec![crate::mounting::UiMountedMosaicRegionGeometry::new(
            instance,
            binding.declaration(),
            binding.executed_region(),
            canonical(
                [17.0, 19.0, 73.0, 41.0],
                UiMountedCoordinateSpace::GraphNodeLocal,
            ),
        )]
    } else {
        Vec::new()
    };
    let mut layout = session.begin_mounted_layout();
    let mut batch = crate::mounting::UiMountedSurfaceGeometryBatch::new(
        layout.basis(surface).unwrap(),
        crate::mounting::UiMountedLayoutRevision::new(1).unwrap(),
        viewport,
        vec![crate::mounting::UiMountedOccurrenceGeometry::surface(
            instance,
            canonical(occurrence, UiMountedCoordinateSpace::HostSurface),
        )],
    );
    if !regions.is_empty() {
        batch = batch.with_regions(regions);
    }
    layout
        .complete_surface_geometry(batch)
        .expect("Mosaic and Scroll resolve from exact mounted owners");
}

fn canonical(
    [x, y, width, height]: [f32; 4],
    coordinate_space: UiMountedCoordinateSpace,
) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space,
    })
    .unwrap()
}
