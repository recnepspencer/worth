use super::*;

pub(super) fn semantic_projection(
    graph_node: crate::graph::UiGraphNodeIdentity,
    instance: UiMountedInstanceIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    seed: UiMountedSemanticTextSeed,
) -> UiMountedSemanticProjection {
    semantic_projection_in_coordinates(
        graph_node,
        instance,
        surface,
        binding,
        seed,
        UiMountedCoordinateSpace::Viewport,
    )
}

pub(super) fn semantic_projection_in_coordinates(
    graph_node: crate::graph::UiGraphNodeIdentity,
    instance: UiMountedInstanceIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    seed: UiMountedSemanticTextSeed,
    coordinate_space: UiMountedCoordinateSpace,
) -> UiMountedSemanticProjection {
    let viewport = canonical_bounds();
    let bounds = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: viewport.x(),
        y: viewport.y(),
        width: viewport.width(),
        height: viewport.height(),
        coordinate_space,
    })
    .unwrap();
    UiMountedSemanticProjection::initial(
        vec![UiMountedProjectionNodeRecord {
            surface_geometry: worth_ui_host_contract::UiSurfaceGeometry::default(),
            receipt: UiMountedNodeReceipt::from_input(UiMountedNodeReceiptInput {
                mounted_instance: instance,
                graph_node,
                semantic_surface: surface,
                incarnation: UiMountIncarnation::mint_unbound().unwrap(),
                plan_digest: 7,
                role: UiMountedMechanicalRole::Control,
                participation: admitted_participation(),
                allocation: UiMountedAllocationProjection::Known {
                    bounds,
                    basis: UiMountedAllocationBasis::new(
                        1,
                        2,
                        3,
                        UiMountedTransformProjection::Identity,
                    ),
                },
            }),
            plan_index: Some(0),
        occurrence_allocation: UiMountedAllocationProjection::Known {
                    bounds,
                    basis: UiMountedAllocationBasis::new(
                        1,
                        2,
                        3,
                        UiMountedTransformProjection::Identity,
                    ),
                },
        appearance_geometry: crate::mounting::projection::frame_storage::UiMountedAppearanceGeometry::from_occurrence(
                UiMountedAllocationProjection::Known {
                    bounds,
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
        portal_surface_appearance: true,
        has_appearance_attachment: false,
            appearance_clip:
                crate::mounting::projection::appearance::UiMountedAppearanceClip::Unclipped,
            semantic_text: Some(seed),
            hit_test: Some(UiMountedHitTestSeed::for_test(0)),
            focus_support: crate::capability::ComponentFocusSupport::not_focusable(),
            focus_scope: None,
            focus_container_owner: None,
            component_id: None,
            portal_child_owner: None,
        }],
        vec![UiMountedProjectionSurface {
                coordinate_posture: crate::mounting::UiSurfaceBindingCoordinatePosture::LogicalPoints,
            surface,
            binding,
            audience: UiMountedProjectionAudience::full(),
        }],
    )
}
