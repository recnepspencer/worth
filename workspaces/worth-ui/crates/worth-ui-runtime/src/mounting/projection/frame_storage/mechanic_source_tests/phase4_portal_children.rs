use std::sync::Arc;

mod appearance_geometry;
mod clipping;
mod lifecycle;
mod membership;
mod motion_index;

use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostPresentationEpoch, UiHostSurfaceIdentity,
    UiMountIncarnation, UiMountedAllocationBasis, UiMountedAllocationProjection,
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedContentGeneration,
    UiMountedCoordinateSpace, UiMountedFrameIdentity, UiMountedInstanceIdentity,
    UiMountedMechanicalRole, UiMountedPaintCommand, UiMountedProjectionAudience,
    UiMountedTransformProjection, UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
    WorthUiHostCapabilityObservationGeneration,
};

use super::super::{
    UiMountedProjectionFrame, UiMountedProjectionFrameInput, UiMountedProjectionNodeRecord,
    UiMountedProjectionSurface, UiMountedSemanticProjection,
};
use crate::mounting::projection::node_receipt::{UiMountedNodeReceipt, UiMountedNodeReceiptInput};
use crate::mounting::projection::{
    hit_test::UiMountedHitTestSeed, semantic_text::UiMountedSemanticTextSeed,
    static_paint::UiMountedStaticPaintSeed,
};

fn projection_frame(
    semantic: UiMountedSemanticProjection,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    owner: UiMountedInstanceIdentity,
    child: UiMountedInstanceIdentity,
    fonts: Arc<worth_ui_text::UiGlobalFontCollection>,
    mechanics: super::UiMountedMechanicSource,
    overlays: Vec<crate::mounting::UiMountedPortalOverlayProjectionInput>,
    generation: u64,
) -> UiMountedProjectionFrame {
    projection_frame_with_identity(
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        semantic,
        surface,
        binding,
        owner,
        child,
        fonts,
        mechanics,
        overlays,
        generation,
    )
}

#[allow(clippy::too_many_arguments)]
fn projection_frame_with_identity(
    frame: UiMountedFrameIdentity,
    semantic: UiMountedSemanticProjection,
    _surface: UiSemanticSurfaceIdentity,
    _binding: UiSurfaceBindingGeneration,
    owner: UiMountedInstanceIdentity,
    child: UiMountedInstanceIdentity,
    fonts: Arc<worth_ui_text::UiGlobalFontCollection>,
    mechanics: super::UiMountedMechanicSource,
    overlays: Vec<crate::mounting::UiMountedPortalOverlayProjectionInput>,
    generation: u64,
) -> UiMountedProjectionFrame {
    let receipts = receipt_basis(frame, owner, child);
    let changed: std::rc::Rc<[UiMountedInstanceIdentity]> = std::rc::Rc::from([owner, child]);
    let mut projection = UiMountedProjectionFrame::new(UiMountedProjectionFrameInput {
        frame,
        content_generation: UiMountedContentGeneration::mint_unbound().unwrap(),
        receipt_basis: receipts,
        plan_digest: 0x3154,
        semantic,
        counters: crate::mounting::UiMountStageCounters::begin(
            crate::mounting::UiMountWorkClass::InitialMount,
        ),
        capability_generation: WorthUiHostCapabilityObservationGeneration::new(generation),
        capability_profile_digest: generation,
        font_collection: fonts,
        mechanics,
        presentation_effects: Default::default(),
        diagnostics: Default::default(),
        portal_overlays: overlays.into(),
        portal_overlays_changed: true,
        changed_instances: changed,
    });
    projection.complete_mechanics().unwrap();
    projection
}

fn portal_semantic_projection(
    owner: UiMountedInstanceIdentity,
    child: UiMountedInstanceIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
) -> UiMountedSemanticProjection {
    let owner_component = crate::capability::ComponentId::new("phase4.portal.trigger").unwrap();
    let child_component = crate::capability::ComponentId::new("phase4.portal.content").unwrap();
    UiMountedSemanticProjection::initial(
        vec![
            node(
                owner,
                4_151,
                surface,
                surface_bounds([20.0, 20.0, 80.0, 32.0]),
                Some(owner_component.clone()),
                None,
                false,
            ),
            node(
                child,
                4_152,
                surface,
                surface_bounds([8.0, 12.0, 220.0, 120.0]),
                Some(child_component),
                Some(owner_component),
                true,
            ),
        ],
        vec![UiMountedProjectionSurface {
            surface,
            binding,
            audience: UiMountedProjectionAudience::full(),
        }],
    )
}

fn node(
    instance: UiMountedInstanceIdentity,
    graph: u64,
    surface: UiSemanticSurfaceIdentity,
    allocation: UiMountedCanonicalBox,
    component_id: Option<crate::capability::ComponentId>,
    portal_child_owner: Option<crate::capability::ComponentId>,
    semantic_text: bool,
) -> UiMountedProjectionNodeRecord {
    UiMountedProjectionNodeRecord {
        receipt: UiMountedNodeReceipt::from_input(UiMountedNodeReceiptInput {
            mounted_instance: instance,
            graph_node: crate::graph::UiGraphNodeIdentity::new(graph),
            semantic_surface: surface,
            incarnation: UiMountIncarnation::mint_unbound().unwrap(),
            plan_digest: 0x3154,
            role: UiMountedMechanicalRole::Control,
            participation: super::admitted_participation(),
            allocation: UiMountedAllocationProjection::Known {
                bounds: bounds([
                    allocation.x(),
                    allocation.y(),
                    allocation.width(),
                    allocation.height(),
                ]),
                basis: UiMountedAllocationBasis::new(
                    1,
                    2,
                    3,
                    UiMountedTransformProjection::Identity,
                ),
            },
        }),
        plan_index: Some((graph - 4_151) as u32),
        occurrence_allocation: UiMountedAllocationProjection::Known {
            bounds: allocation,
            basis: UiMountedAllocationBasis::new(1, 2, 3, UiMountedTransformProjection::Identity),
        },
        appearance_geometry:
            crate::mounting::projection::frame_storage::UiMountedAppearanceGeometry::from_occurrence(
                UiMountedAllocationProjection::Known {
                    bounds: allocation,
                    basis: UiMountedAllocationBasis::new(
                        1,
                        2,
                        3,
                        UiMountedTransformProjection::Identity,
                    ),
                },
                crate::mounting::projection::appearance::UiMountedAppearanceClip::Unclipped,
            ),
        surface_paint_order: Some(0),
        has_appearance_attachment: true,
        appearance_clip:
            crate::mounting::projection::appearance::UiMountedAppearanceClip::Unclipped,
        static_paint: Some(UiMountedStaticPaintSeed::for_test(
            worth_ui_host_contract::UiMountedRgba8::new(42, 36, 68, 255),
        )),
        semantic_text: semantic_text.then(UiMountedSemanticTextSeed::scalar_for_test),
        hit_test: Some(UiMountedHitTestSeed::for_test((graph - 4_151) as u32)),
        focus_support: crate::capability::ComponentFocusSupport::not_focusable(),
        focus_scope: None,
        focus_container_owner: None,
        component_id,
        portal_child_owner,
    }
}

fn portal_overlay(
    frame: UiMountedFrameIdentity,
    owner: UiMountedInstanceIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
) -> crate::mounting::UiMountedPortalOverlayProjectionInput {
    portal_overlay_for_graph(frame, owner, surface, binding, 4_151)
}

fn portal_overlay_for_graph(
    frame: UiMountedFrameIdentity,
    owner: UiMountedInstanceIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    graph: u64,
) -> crate::mounting::UiMountedPortalOverlayProjectionInput {
    let presentation = UiHostObservationPresentationBasis::new(
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        frame,
        binding,
        UiHostPresentationEpoch::issued_by_host(1),
    );
    let identity = crate::runtime::portal::UiPortalIdentity::for_owner(
        crate::runtime::portal::UiPortalOwnerIdentity::from_mounted_owner(
            crate::graph::UiGraphNodeIdentity::new(graph),
            owner,
        ),
    );
    let request = crate::runtime::portal::UiPortalServiceRequest::open(
        identity,
        crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(1, 1),
        crate::runtime::interaction::UiPresentedInteractionGeometry::for_test_with_components(
            presentation,
            [20.0, 20.0, 80.0, 32.0],
            [20.0, 20.0, 80.0, 32.0],
        ),
        Some(
            crate::runtime::interaction::UiPresentedViewportGeometry::for_test(
                bounds([0.0, 0.0, 960.0, 600.0]),
                presentation,
            ),
        ),
        surface,
    );
    let placement = crate::runtime::portal::UiPreparedPortalPlacement::for_request(&request, None)
        .expect("the exact presented owner admits Portal placement")
        .expect("an open request prepares Portal placement geometry");
    crate::mounting::UiMountedPortalOverlayProjectionInput::new(
        identity.diagnostic_value(),
        owner,
        placement,
        crate::runtime::portal::UiPortalLifecyclePosture::Visible,
    )
}

fn assert_child_suppressed(
    frame: &UiMountedProjectionFrame,
    child: UiMountedInstanceIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
) {
    assert!(frame
        .presentation_commands_for_instance(child, surface, binding)
        .is_empty());
    assert!(frame
        .portal_presentation_affinity_for_instance(child, surface, binding)
        .is_none());
    assert!(!frame
        .presentation_instance_order(surface, binding)
        .iter()
        .any(|instance| *instance == child));
    assert!(!frame
        .visual_region_basis()
        .for_binding(binding, frame.receipt_basis.clone())
        .hit_test()
        .iter()
        .any(|hit| hit.mechanic().mounted_instance() == child));
}

fn owner_filled_rect(
    frame: &UiMountedProjectionFrame,
    owner: UiMountedInstanceIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
) -> worth_ui_host_contract::UiMountedFilledRectMechanic {
    frame
        .presentation_commands_for_instance(owner, surface, binding)
        .iter()
        .find_map(|command| match command {
            UiMountedPaintCommand::FilledRect { mechanic, .. } => Some(*mechanic),
            _ => None,
        })
        .unwrap()
}

fn command_bounds(command: &UiMountedPaintCommand) -> UiMountedCanonicalBox {
    match command {
        UiMountedPaintCommand::FilledRect { mechanic, .. } => mechanic.bounds(),
        UiMountedPaintCommand::SemanticText { mechanic, .. } => mechanic.bounds(),
        UiMountedPaintCommand::PortalOverlay { mechanic, .. } => mechanic.bounds(),
    }
}

fn receipt_basis(
    frame: UiMountedFrameIdentity,
    owner: UiMountedInstanceIdentity,
    child: UiMountedInstanceIdentity,
) -> crate::mounting::UiMountedNodeReceiptBasis {
    let mut instances = crate::runtime::persistent_index::UiPersistentOrdSet::default();
    instances.insert(owner);
    instances.insert(child);
    crate::mounting::UiMountedNodeReceiptBasis::mint(frame, instances).unwrap()
}

fn bounds(components: [f32; 4]) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: components[0],
        y: components[1],
        width: components[2],
        height: components[3],
        coordinate_space: UiMountedCoordinateSpace::Viewport,
    })
    .unwrap()
}

fn surface_bounds(components: [f32; 4]) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: components[0],
        y: components[1],
        width: components[2],
        height: components[3],
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .unwrap()
}
