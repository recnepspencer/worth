//! Scrolling an admitted region past the viewport origin omits its hit row,
//! and scrolling it off the device grid keeps both of its boxes.
//!
//! The geometry stays well formed: only its posture changes. A frame that
//! failed instead would end the application every time a list scrolled a
//! control out of view.
use super::*;
use crate::mounting::spatial_index::UiMountedSpatialBudget;

#[test]
fn regions_scrolled_past_the_viewport_origin_are_omitted_not_denied() {
    let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let fonts = Arc::new(fonts);
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let onscreen = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let scrolled_away = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let clipped_to_nothing = UiMountedInstanceIdentity::mint_unbound().unwrap();

    let semantic = projection(
        vec![
            node(onscreen, surface, 0, [40.0, 12.0, 15.0, 15.0]),
            // The 15x15 control the Pulse dashboard scrolled off its left edge.
            node(scrolled_away, surface, 1, [-26.333_31, 649.0, 15.0, 15.0]),
            // Touching the origin from the left is still entirely outside it.
            node(clipped_to_nothing, surface, 2, [-15.0, 30.0, 15.0, 15.0]),
        ],
        surface,
        binding,
    );
    let mut source = UiMountedMechanicSource::default();
    let mutation = attempt(
        &mut source,
        &fonts,
        &semantic,
        &[onscreen, scrolled_away, clipped_to_nothing],
    )
    .expect("an offscreen hit region omits its row rather than failing the frame");

    assert_eq!(
        mutation.hit_tests, 1,
        "only the region still inside the viewport contributes a hit row"
    );
    assert_eq!(query(&source, binding, [47.0, 19.0]), [onscreen]);
    assert!(
        query(&source, binding, [-19.0, 656.0]).is_empty(),
        "an omitted region is absent from the index, not merely unreachable"
    );
}

#[test]
fn a_row_scrolled_off_the_device_grid_hits_where_recorded_and_paints_on_the_grid() {
    let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let fonts = Arc::new(fonts);
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let scrolled = UiMountedInstanceIdentity::mint_unbound().unwrap();
    // A third-of-a-point scroll offset: the host paints the row on the device
    // grid, while its offset leaves it where hit testing must find it.
    let mut row = node(scrolled, surface, 0, [40.0, 177.5, 80.0, 32.0]);
    let recorded = viewport_box([40.0, 177.333_33, 80.0, 32.0]);
    row.recorded_bounds = Some(recorded);
    let semantic = projection(vec![row], surface, binding);
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let receipts = receipts_for(frame, &[scrolled]);
    let mut source = UiMountedMechanicSource::default();
    source
        .apply(completion(
            frame,
            UiMountedContentGeneration::mint_unbound().unwrap(),
            &receipts,
            &semantic,
            &fonts,
            &[scrolled],
            1,
        ))
        .expect("a grid-corrected row completes");

    let row = *source
        .hit_test_for_instance(scrolled, surface, binding, frame, &receipts)
        .unwrap()
        .expect("the scrolled row is admitted")
        .in_layout_space();

    assert_eq!(row.bounds(), recorded, "hit testing reads the recorded box");
    assert_eq!(
        row.painted_bounds(),
        Some(viewport_box([40.0, 177.5, 80.0, 32.0])),
        "the row carries the box the host paints"
    );
}

fn query(
    source: &UiMountedMechanicSource,
    binding: UiSurfaceBindingGeneration,
    point: [f64; 2],
) -> Vec<UiMountedInstanceIdentity> {
    source
        .allocation_hit_candidates(
            binding,
            UiMountedCoordinateSpace::Viewport,
            point,
            UiMountedSpatialBudget {
                node_visits: 1024,
                candidates: 128,
            },
        )
        .unwrap()
        .instances
}

fn attempt(
    source: &mut UiMountedMechanicSource,
    fonts: &Arc<worth_ui_text::UiGlobalFontCollection>,
    semantic: &UiMountedSemanticProjection,
    changed: &[UiMountedInstanceIdentity],
) -> Result<
    super::super::mechanic_source::UiMountedMechanicMutation,
    crate::mounting::UiMountedProjectionDenial,
> {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let receipts = receipts_for(frame, changed);
    source.apply(completion(
        frame,
        UiMountedContentGeneration::mint_unbound().unwrap(),
        &receipts,
        semantic,
        fonts,
        changed,
        1,
    ))
}

fn receipts_for(
    frame: UiMountedFrameIdentity,
    changed: &[UiMountedInstanceIdentity],
) -> crate::mounting::UiMountedNodeReceiptBasis {
    let mut instances = crate::runtime::persistent_index::UiPersistentOrdSet::default();
    for instance in changed {
        instances.insert(*instance);
    }
    crate::mounting::UiMountedNodeReceiptBasis::mint(frame, instances).unwrap()
}

fn viewport_box(edges: [f32; 4]) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: edges[0],
        y: edges[1],
        width: edges[2],
        height: edges[3],
        coordinate_space: UiMountedCoordinateSpace::Viewport,
    })
    .unwrap()
}

fn projection(
    nodes: Vec<UiMountedProjectionNodeRecord>,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
) -> UiMountedSemanticProjection {
    UiMountedSemanticProjection::initial(
        nodes,
        vec![UiMountedProjectionSurface {
            coordinate_posture: crate::mounting::UiSurfaceBindingCoordinatePosture::LogicalPoints,
            surface,
            binding,
            audience: UiMountedProjectionAudience::full(),
        }],
    )
}

fn node(
    instance: UiMountedInstanceIdentity,
    surface: UiSemanticSurfaceIdentity,
    rank: u32,
    edges: [f32; 4],
) -> UiMountedProjectionNodeRecord {
    let bounds = viewport_box(edges);
    let allocation = UiMountedAllocationProjection::Known {
        bounds,
        basis: UiMountedAllocationBasis::new(1, 2, 3, UiMountedTransformProjection::Identity),
    };
    UiMountedProjectionNodeRecord {
        surface_geometry: worth_ui_host_contract::UiSurfaceGeometry::default(),
        receipt: UiMountedNodeReceipt::from_input(UiMountedNodeReceiptInput {
            mounted_instance: instance,
            graph_node: crate::graph::UiGraphNodeIdentity::new(70_000 + u64::from(rank)),
            semantic_surface: surface,
            incarnation: UiMountIncarnation::mint_unbound().unwrap(),
            plan_digest: 7,
            role: UiMountedMechanicalRole::Control,
            participation: admitted_participation(),
            allocation,
        }),
        plan_index: Some(rank),
        recorded_bounds: None,
        occurrence_allocation: crate::mounting::UiLaidOut::from_layout(allocation),
        appearance_geometry:
            crate::mounting::projection::frame_storage::UiMountedAppearanceGeometry::in_place(
                crate::mounting::UiLaidOut::from_layout(allocation),
                crate::mounting::projection::appearance::UiMountedAppearanceClip::Unclipped,
            ),
        surface_paint_order: Some(0),
        portal_surface_appearance: true,
        has_appearance_attachment: false,
        appearance_clip:
            crate::mounting::projection::appearance::UiMountedAppearanceClip::Unclipped,
        semantic_text: None,
        hit_test: Some(UiMountedHitTestSeed::for_test(rank)),
        focus_support: crate::capability::ComponentFocusSupport::not_focusable(),
        focus_scope: None,
        focus_container_owner: None,
        component_id: None,
        portal_child_owner: None,
    }
}
