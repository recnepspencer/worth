use super::{
    UiMountedPresentationDelta, UiMountedPresentationDeltaInput, UiMountedPresentationInitial,
    UiMountedPresentationInitialInput, UiMountedPresentationOpacity, UiMountedPresentationSample,
    UiMountedPresentationSampleChange, UiMountedPresentationSampleConstructionDenial,
    UiMountedPresentationSampleInput, UiMountedPresentationTransform,
    UiMountedPresentationUnchanged, UiMountedPresentationUnchangedInput,
    UiMountedPresentationWorkView,
};

#[test]
fn inert_initial_delta_and_unchanged_mechanics_remain_distinct() {
    let predecessor = crate::UiMountedFrameIdentity::mint_unbound().expect("predecessor");
    let successor = crate::UiMountedFrameIdentity::mint_unbound().expect("successor");
    let surface = crate::UiSemanticSurfaceIdentity::mint_unbound().expect("surface");
    let binding = crate::UiSurfaceBindingGeneration::mint_unbound().expect("binding");
    let content = crate::UiMountedContentGeneration::mint_unbound().expect("content");
    let baseline = crate::UiHostSurfaceBaselineIdentity::from_surface_binding(
        surface,
        crate::UiHostSurfaceIdentity::mint_unbound().expect("host surface"),
        binding,
        crate::WorthUiHostCapabilityObservationGeneration::new(7),
        11,
        crate::UiHostSurfacePresentationMode::RecordOnly,
    );
    let projection = empty_projection(successor, surface, binding, content);
    let empty_order = Vec::new();
    let empty_order_integrity = super::UiMountedPaintOrderIntegrity::for_order(&empty_order);

    let initial =
        UiMountedPresentationInitial::from_inert_mechanics(UiMountedPresentationInitialInput {
            successor,
            surface,
            binding,
            content,
            baseline,
            projection,
            commands: Vec::new(),
            order: empty_order.clone(),
            order_integrity: empty_order_integrity,
            damage: Vec::new(),
            production_cost: Default::default(),
        });
    assert!(matches!(
        UiMountedPresentationWorkView::Initial(&initial),
        UiMountedPresentationWorkView::Initial(_)
    ));

    let delta = UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
        predecessor,
        successor,
        surface,
        binding,
        content,
        baseline,
        changes: Vec::new(),
        nodes: Vec::new(),
        order: Vec::new(),
        order_integrity: empty_order_integrity,
        damage: Vec::new(),
        auxiliary: Some(
            super::UiMountedPresentationAuxiliaryState::from_runtime_mounting(&empty_projection(
                successor, surface, binding, content,
            )),
        ),
        production_cost: Default::default(),
    });
    assert!(matches!(
        UiMountedPresentationWorkView::Delta(&delta),
        UiMountedPresentationWorkView::Delta(_)
    ));
    let unchanged =
        UiMountedPresentationUnchanged::from_inert_mechanics(UiMountedPresentationUnchangedInput {
            predecessor,
            successor,
            surface,
            binding,
            content,
            baseline,
            production_cost: Default::default(),
        });
    assert!(matches!(
        UiMountedPresentationWorkView::Unchanged(&unchanged),
        UiMountedPresentationWorkView::Unchanged(_)
    ));
}

#[test]
fn sample_contract_brands_finite_same_frame_unique_work() {
    let frame = crate::UiMountedFrameIdentity::mint_unbound().expect("frame");
    let surface = crate::UiSemanticSurfaceIdentity::mint_unbound().expect("surface");
    let binding = crate::UiSurfaceBindingGeneration::mint_unbound().expect("binding");
    let content = crate::UiMountedContentGeneration::mint_unbound().expect("content");
    let baseline = crate::UiHostSurfaceBaselineIdentity::from_surface_binding(
        surface,
        crate::UiHostSurfaceIdentity::mint_unbound().expect("host surface"),
        binding,
        crate::WorthUiHostCapabilityObservationGeneration::new(9),
        13,
        crate::UiHostSurfacePresentationMode::RecordOnly,
    );
    let command = crate::UiMountedPaintCommandIdentity::semantic_text_from_correspondence(
        crate::UiMountedInstanceIdentity::mint_unbound().expect("instance"),
        0,
        None,
    );
    let source = canonical_box(crate::UiMountedCoordinateSpace::Viewport, 1.0, 2.0);
    let sampled = canonical_box(crate::UiMountedCoordinateSpace::Viewport, 3.0, 4.0);
    let opacity = UiMountedPresentationOpacity::from_runtime_composition(32_768);
    let change = UiMountedPresentationSampleChange::from_runtime_sampling(
        command,
        Some(UiMountedPresentationTransform::from_runtime_sampling(source, sampled).unwrap()),
        opacity,
    );
    let input = |changes| UiMountedPresentationSampleInput {
        frame,
        surface,
        binding,
        content,
        baseline,
        changes,
        damage: vec![super::UiMountedLogicalDamage::from_runtime_mounting(
            sampled,
        )],
        production_cost: Default::default(),
    };

    let sample = UiMountedPresentationSample::from_inert_mechanics(input(vec![change])).unwrap();
    assert_eq!(sample.affinity().predecessor(), Some(frame));
    assert_eq!(sample.affinity().successor(), frame);
    assert_eq!(sample.changes(), &[change]);
    assert_eq!(
        UiMountedPresentationSample::from_inert_mechanics(input(Vec::new())),
        Err(UiMountedPresentationSampleConstructionDenial::EmptyChanges)
    );
    assert_eq!(
        UiMountedPresentationSample::from_inert_mechanics(input(vec![change, change])),
        Err(UiMountedPresentationSampleConstructionDenial::DuplicateCommandIdentity)
    );
}

#[test]
fn sample_values_reject_mixed_coordinate_spaces() {
    assert_eq!(
        UiMountedPresentationTransform::from_runtime_sampling(
            canonical_box(crate::UiMountedCoordinateSpace::Viewport, 0.0, 0.0),
            canonical_box(crate::UiMountedCoordinateSpace::HostSurface, 0.0, 0.0),
        ),
        Err(UiMountedPresentationSampleConstructionDenial::CoordinateSpaceMismatch)
    );
}

#[test]
fn presentation_opacity_preserves_all_canonical_units() {
    for units in 0..=u16::MAX {
        assert_eq!(
            UiMountedPresentationOpacity::from_runtime_composition(units).units(),
            units
        );
    }
}

fn canonical_box(
    coordinate_space: crate::UiMountedCoordinateSpace,
    x: f32,
    y: f32,
) -> crate::UiMountedCanonicalBox {
    crate::UiMountedCanonicalBox::canonicalize(crate::UiMountedCanonicalBoxInput {
        x,
        y,
        width: 10.0,
        height: 10.0,
        coordinate_space,
    })
    .unwrap()
}

fn empty_projection(
    frame: crate::UiMountedFrameIdentity,
    surface: crate::UiSemanticSurfaceIdentity,
    binding: crate::UiSurfaceBindingGeneration,
    content_generation: crate::UiMountedContentGeneration,
) -> crate::UiMountedProjectionView {
    crate::UiMountedProjectionView::new(crate::UiMountedProjectionViewInput {
        frame,
        surface,
        binding,
        content_generation,
        nodes: Vec::new(),
        clips: crate::UiMountedClipTable::produced(Vec::new()),
        layers: crate::UiMountedLayerTable::produced(Vec::new()),
        filled_rects: crate::UiMountedFilledRectTable::empty(),
        portal_overlays: crate::UiMountedPortalOverlayTable::empty(),
        semantic_text: crate::UiMountedSemanticTextTable::empty(),
        hit_tests: crate::UiMountedHitTestTable::empty(),
        paint_batches: crate::UiMountedPaintBatchTable::new(Vec::new()),
        spatial_batches: crate::UiMountedSpatialBatchTable::new(Vec::new()),
        realtime_batches: crate::UiMountedRealtimeBatchTable::new(Vec::new()),
        resources: crate::UiMountedResourceTable::new(Vec::new()),
        authored_paint_commands: Vec::new(),
        authored_paint_order: Vec::new(),
    })
}
