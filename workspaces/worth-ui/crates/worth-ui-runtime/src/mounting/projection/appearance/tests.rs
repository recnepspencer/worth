use worth_ui_host_contract::{
    UiAppearanceAllocationBounds, UiAppearanceClip, UiAppearanceLogicalLength,
    UiAppearanceNormalizedLogicalRadii, UiAppearanceOutlineGeometry, UiHostPointerIdentity,
    UiMountedAppearanceColor, UiMountedAppearanceMechanic, UiMountedAppearanceMechanicIdentity,
    UiMountedAppearanceOpacity, UiMountedAppearanceWorkPosture,
    UiMountedBackdropAppearanceAttribution, UiMountedBackdropIdentity, UiMountedBackdropScope,
    UiMountedFrameIdentity, UiMountedInstanceIdentity, UiMountedLayerProjection,
    UiMountedLayerReference, UiMountedNodeAppearanceAttribution, UiMountedNodeReceiptIssuer,
    UiMountedOverlayOrderMechanic, UiMountedSurfacePaint, UiMountedTextPaintSpanIdentity,
    UiOverlayParticipantIdentity, UiOverlayPlacementReceipt, UiPointerAffordanceFamily,
    UiSemanticSurfaceIdentity,
};

use super::fact::{
    UiMountedAppearanceBackdropInput, UiMountedAppearanceLoweringInput,
    UiMountedAppearanceNodeInput, UiMountedAppearanceOutlineInput, UiMountedAppearanceOverlayInput,
    UiMountedAppearancePointerInput, UiMountedAppearanceTextForegroundInput,
};
use super::UiMountedAppearanceSidecar;

mod overlay;

struct NodeIds {
    frame: UiMountedFrameIdentity,
    issuer: UiMountedNodeReceiptIssuer,
    surface: UiSemanticSurfaceIdentity,
    presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    instance: UiMountedInstanceIdentity,
    receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    projection: UiMountedNodeAppearanceAttribution,
    bounds: UiAppearanceAllocationBounds,
    outline: UiAppearanceOutlineGeometry,
}

fn node_input(
    color: [u8; 4],
    semantic_digest: u64,
    include_outline: bool,
) -> (UiMountedAppearanceLoweringInput, NodeIds) {
    node_input_for(color, semantic_digest, include_outline, None)
}

fn node_input_for(
    color: [u8; 4],
    semantic_digest: u64,
    include_outline: bool,
    stable: Option<&NodeIds>,
) -> (UiMountedAppearanceLoweringInput, NodeIds) {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let surface = stable
        .map(|stable| stable.surface)
        .unwrap_or_else(|| UiSemanticSurfaceIdentity::mint_unbound().unwrap());
    let presentation = stable.map(|stable| stable.presentation).unwrap_or_else(|| {
        worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap()
    });
    let instance = stable
        .map(|stable| stable.instance)
        .unwrap_or_else(|| UiMountedInstanceIdentity::mint_unbound().unwrap());
    let receipt = issuer.receipt_for(instance);
    let bounds = UiAppearanceAllocationBounds::new(10, 20, 100, 80).unwrap();
    let zero = UiAppearanceLogicalLength::ZERO;
    let radii = UiAppearanceNormalizedLogicalRadii::normalize(bounds, [zero; 4]);
    let outline = UiAppearanceOutlineGeometry::admit(
        bounds,
        radii,
        UiAppearanceLogicalLength::new(4).unwrap(),
        UiAppearanceLogicalLength::new(2).unwrap(),
        UiAppearanceLogicalLength::new(1).unwrap(),
    )
    .unwrap();
    let projection =
        UiMountedNodeAppearanceAttribution::from_runtime_mounting(issuer, 11, 3).unwrap();
    let node = UiMountedAppearanceNodeInput {
        issuer,
        semantic_surface: surface,
        node_receipt: receipt,
        projection,
        bounds,
        clip: UiAppearanceClip::new(0, 0, 90, 70).unwrap(),
        layer: UiMountedLayerProjection::Layer(UiMountedLayerReference::new(0)),
        radii,
        surface_paint: Some(UiMountedSurfacePaint::Fill(
            UiMountedAppearanceColor::from_straight_srgba(color),
        )),
        outline: include_outline.then_some(UiMountedAppearanceOutlineInput {
            geometry: outline,
            color: UiMountedAppearanceColor::from_straight_srgba([4, 5, 6, 255]),
        }),
        text_foregrounds: vec![UiMountedAppearanceTextForegroundInput {
            span: UiMountedTextPaintSpanIdentity::from_runtime_mounting([8; 32]),
            foreground: UiMountedAppearanceColor::from_straight_srgba([20, 30, 40, 255]),
        }]
        .into_boxed_slice(),
        pointer: Some(UiMountedAppearancePointerInput {
            pointer: UiHostPointerIdentity::new(5),
            surface,
            target: instance,
            family: UiPointerAffordanceFamily::Activation,
        }),
        appearance_opacity: UiMountedAppearanceOpacity::from_units(40_000),
        motion_opacity: Some(UiMountedAppearanceOpacity::from_units(32_768)),
        semantic_digest,
        portal_instance: None,
    };
    let overlay = UiMountedAppearanceOverlayInput {
        semantic_surface: surface,
        presentation,
        portal_revision: 1,
        backdrop_revision: 1,
        bottom_to_top: Box::new([]),
    };
    (
        UiMountedAppearanceLoweringInput {
            frame,
            semantic_surface: surface,
            presentation,
            nodes: vec![node],
            backdrops: Vec::new(),
            overlay,
        },
        NodeIds {
            frame,
            issuer,
            surface,
            presentation,
            instance,
            receipt,
            projection,
            bounds,
            outline,
        },
    )
}

pub(crate) struct MountedAppearanceReconstructionTestFixture {
    pub(crate) sidecar: UiMountedAppearanceSidecar,
    pub(crate) frame: UiMountedFrameIdentity,
    pub(crate) issuer: UiMountedNodeReceiptIssuer,
    pub(crate) surface: UiSemanticSurfaceIdentity,
    pub(crate) instance: UiMountedInstanceIdentity,
    pub(crate) receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    pub(crate) graph_node: crate::graph::UiGraphNodeIdentity,
}

pub(crate) fn mounted_sidecar_with_retained_facts_for_test(
) -> MountedAppearanceReconstructionTestFixture {
    let (mut input, ids) = node_input([1, 2, 3, 255], 7, false);
    let node = &mut input.nodes[0];
    node.clip = UiAppearanceClip::new(10, 20, 100, 80).unwrap();
    node.layer = UiMountedLayerProjection::Omitted(
        worth_ui_host_contract::UiMountedOmissionReason::NotDefinedByCurrentRuntime,
    );
    node.text_foregrounds = Box::new([]);
    node.pointer = None;
    node.appearance_opacity = UiMountedAppearanceOpacity::ONE;
    node.motion_opacity = None;
    node.surface_paint = Some(UiMountedSurfacePaint::Fill(
        UiMountedAppearanceColor::from_straight_srgba([1, 2, 3, 255]),
    ));
    input.overlay.portal_revision = 0;
    input.overlay.backdrop_revision = 0;
    let mut sidecar = UiMountedAppearanceSidecar::default();
    sidecar
        .mount(input)
        .expect("test appearance facts should mount before reconstruction");
    MountedAppearanceReconstructionTestFixture {
        sidecar,
        frame: ids.frame,
        issuer: ids.issuer,
        surface: ids.surface,
        instance: ids.instance,
        receipt: ids.receipt,
        graph_node: crate::graph::UiGraphNodeIdentity::new(11),
    }
}

#[test]
fn initial_mount_records_attribution_and_visual_bounds_without_host_commands() {
    let (input, ids) = node_input([12, 34, 56, 255], 7, true);
    let mut sidecar = UiMountedAppearanceSidecar::default();

    let work = sidecar.mount(input).unwrap();

    assert_eq!(work.posture(), UiMountedAppearanceWorkPosture::Initial);
    assert_structural_appearance_work(&work, 4);
    assert_eq!(work.changes().len(), 4);
    assert_eq!(work.damage().len(), 1);
    assert_eq!(work.damage()[0].x(), ids.outline.visual_bounds().x());
    assert_eq!(work.damage()[0].y(), ids.outline.visual_bounds().y());
    assert_eq!(
        work.damage()[0].width(),
        ids.outline.visual_bounds().width()
    );
    assert_eq!(
        work.damage()[0].height(),
        ids.outline.visual_bounds().height()
    );
    let facts = sidecar.current().unwrap();
    let surface = facts
        .record(&UiMountedAppearanceMechanicIdentity::Surface(ids.instance))
        .unwrap();
    assert_eq!(surface.node_receipt(), Some(ids.receipt));
    assert_eq!(surface.projection(), Some(ids.projection));
    assert_eq!(surface.projection().unwrap().identity(), 11);
    assert!(matches!(
        surface.mechanic(),
        UiMountedAppearanceMechanic::Surface(mechanic)
            if mechanic.node_receipt() == ids.receipt
                && mechanic.bounds() == ids.bounds
                && mechanic.projection() == ids.projection
                && mechanic.opacity().units() == 20_000
    ));
    let outline = facts
        .record(&UiMountedAppearanceMechanicIdentity::Outline(ids.instance))
        .unwrap();
    assert_eq!(outline.node_receipt(), Some(ids.receipt));
    assert!(matches!(
        outline.damage(),
        super::fact::UiMountedAppearanceDamageShape::Visual { bounds, .. }
            if bounds.x() == ids.outline.visual_bounds().x()
                && bounds.width() == ids.outline.visual_bounds().width()
    ));
    let text = facts
        .records()
        .iter()
        .find(|record| {
            matches!(
                record.mechanic(),
                UiMountedAppearanceMechanic::TextForeground(_)
            )
        })
        .unwrap();
    assert_eq!(text.node_receipt(), Some(ids.receipt));
    assert!(matches!(
        text.mechanic(),
        UiMountedAppearanceMechanic::TextForeground(mechanic)
            if mechanic.node_receipt() == ids.receipt
                && mechanic.projection() == ids.projection
    ));
}

#[test]
fn lowering_denies_a_foreign_node_receipt_lineage_before_effects() {
    let (mut input, _) = node_input([12, 34, 56, 255], 7, false);
    let foreign_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    input.nodes[0].issuer = UiMountedNodeReceiptIssuer::mint_for(foreign_frame).unwrap();
    let mut sidecar = UiMountedAppearanceSidecar::default();

    assert_eq!(
        sidecar.mount(input),
        Err(super::UiMountedAppearanceLoweringDenial::NodeReceiptFrameMismatch)
    );
    assert!(sidecar.current().is_none());
}

#[test]
fn unchanged_mount_suppresses_equal_physical_output_but_keeps_new_receipts() {
    let (first, ids) = node_input([12, 34, 56, 255], 7, false);
    let (second, second_ids) = node_input_for([12, 34, 56, 255], 7, false, Some(&ids));
    let mut sidecar = UiMountedAppearanceSidecar::default();
    sidecar.mount(first).unwrap();

    let work = sidecar.mount(second).unwrap();

    assert_eq!(work.posture(), UiMountedAppearanceWorkPosture::Unchanged);
    assert!(work.changes().is_empty());
    assert!(work.damage().is_empty());
    let summary = sidecar.last_delta().unwrap();
    assert_eq!(summary.semantic_facts_changed(), 0);
    assert!(!summary.mechanics_changed());
    assert!(summary.output_suppressed());
    assert_ne!(ids.frame, second_ids.frame);
    assert_eq!(
        sidecar
            .current()
            .unwrap()
            .record(&UiMountedAppearanceMechanicIdentity::Surface(
                second_ids.instance
            ))
            .unwrap()
            .node_receipt(),
        Some(second_ids.receipt)
    );
}

#[test]
fn paint_change_is_mechanical_and_semantic_change_can_still_suppress_output() {
    let (first, ids) = node_input([12, 34, 56, 255], 7, false);
    let (mut color_change, _) = node_input_for([80, 90, 100, 255], 7, false, Some(&ids));
    let (semantic_change, _) = node_input_for([80, 90, 100, 255], 99, false, Some(&ids));
    let mut sidecar = UiMountedAppearanceSidecar::default();
    sidecar.mount(first).unwrap();

    let color_work = sidecar.mount(color_change).unwrap();
    let color_summary = sidecar.last_delta().unwrap();
    assert_eq!(color_work.posture(), UiMountedAppearanceWorkPosture::Delta);
    assert_eq!(color_work.changes().len(), 1);
    assert_eq!(color_work.damage().len(), 1);
    assert_eq!(color_summary.semantic_facts_changed(), 0);
    assert!(color_summary.mechanics_changed());
    assert!(!color_summary.output_suppressed());

    color_change = semantic_change;
    let semantic_work = sidecar.mount(color_change).unwrap();
    let semantic_summary = sidecar.last_delta().unwrap();
    assert_eq!(
        semantic_work.posture(),
        UiMountedAppearanceWorkPosture::Unchanged
    );
    assert!(semantic_work.changes().is_empty());
    assert_eq!(semantic_summary.semantic_facts_changed(), 3);
    assert!(!semantic_summary.mechanics_changed());
    assert!(semantic_summary.output_suppressed());
    assert_eq!(ids.surface, second_surface(&sidecar));
}

#[test]
fn reconstruction_rebuilds_from_current_receipt_without_semantic_replay() {
    let (input, ids) = node_input([12, 34, 56, 255], 7, true);
    let (successor, successor_ids) = node_input_for([12, 34, 56, 255], 7, true, Some(&ids));
    let mut sidecar = UiMountedAppearanceSidecar::default();
    sidecar.mount(input).unwrap();

    let work = sidecar
        .reconstruct(successor)
        .expect("current receipt should rebuild retained appearance facts");
    let manifest = work
        .predecessor_manifest()
        .expect("reconstruction keeps the retained predecessor manifest");
    let retained_identities = sidecar
        .current()
        .unwrap()
        .records()
        .iter()
        .map(|record| record.identity().clone())
        .collect::<Vec<_>>();

    assert_eq!(
        work.posture(),
        UiMountedAppearanceWorkPosture::Reconstruction
    );
    assert_eq!(work.predecessor(), Some(ids.frame));
    assert_eq!(work.successor().frame(), successor_ids.frame);
    assert_eq!(
        manifest.mechanic_identities(),
        retained_identities.as_slice()
    );
    assert_eq!(
        manifest.overlay_order(),
        work.successor().overlay_order().bottom_to_top()
    );
    assert!(work.changes().is_empty());
    assert!(work.damage().is_empty());
    assert!(!work.order_changed());
    assert_eq!(
        sidecar
            .current()
            .unwrap()
            .record(&UiMountedAppearanceMechanicIdentity::Surface(
                successor_ids.instance,
            ))
            .unwrap()
            .node_receipt(),
        Some(successor_ids.receipt)
    );
}

fn second_surface(sidecar: &UiMountedAppearanceSidecar) -> UiSemanticSurfaceIdentity {
    sidecar.current().unwrap().frame().semantic_surface()
}

fn assert_structural_appearance_work(
    work: &worth_ui_host_contract::UiMountedAppearanceWork,
    expected: usize,
) {
    // Gate 1 carries semantic appearance mechanics and change records only;
    // the actual work graph is the independent no-host oracle. This checks the
    // produced structure rather than the literal no-host accessors.
    assert_eq!(work.successor().mechanics().len(), expected);
    assert_eq!(work.changes().len(), expected);
    assert!(work.changes().iter().all(|change| match change {
        worth_ui_host_contract::UiMountedAppearanceMechanicChange::Insert(mechanic)
        | worth_ui_host_contract::UiMountedAppearanceMechanicChange::Replace {
            successor: mechanic,
            ..
        } => work.successor().mechanics().contains(mechanic),
        worth_ui_host_contract::UiMountedAppearanceMechanicChange::Remove(_) => false,
    }));
}
