use super::same_geometry_input;
use crate::mounting::projection::{
    frame_storage::{
        UiMountedProjectionNodeRecord, UiMountedProjectionSurface, UiMountedSemanticProjection,
    },
    node_receipt::{UiMountedNodeReceipt, UiMountedNodeReceiptInput},
};
use worth_ui_host_contract::*;

#[test]
fn unchanged_geometry_cannot_suppress_a_changed_plan() {
    let node = node();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let predecessor = projection(node.clone(), Some(binding));
    let unchanged = projection(node.clone(), Some(binding));
    assert!(same_geometry_input(&predecessor, &unchanged, &node).0);

    let mut changed = node.clone();
    let receipt = node.receipt();
    changed.receipt = UiMountedNodeReceipt::from_input(UiMountedNodeReceiptInput {
        mounted_instance: receipt.mounted_instance(),
        graph_node: receipt.graph_node(),
        semantic_surface: receipt.semantic_surface(),
        incarnation: receipt.incarnation(),
        plan_digest: receipt.plan_digest() + 1,
        role: receipt.role(),
        participation: receipt.participation(),
        allocation: receipt.allocation(),
    });
    let successor = projection(changed.clone(), Some(binding));
    assert!(!same_geometry_input(&predecessor, &successor, &changed).0);
}

#[test]
fn missing_binding_is_not_geometry_equivalence() {
    let node = node();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    for (old, new) in [(None, None), (Some(binding), None), (None, Some(binding))] {
        let predecessor = projection(node.clone(), old);
        let successor = projection(node.clone(), new);
        assert!(!same_geometry_input(&predecessor, &successor, &node).0);
    }
}

fn projection(
    node: UiMountedProjectionNodeRecord,
    binding: Option<UiSurfaceBindingGeneration>,
) -> UiMountedSemanticProjection {
    let surface = node.receipt().semantic_surface();
    UiMountedSemanticProjection::initial(
        vec![node],
        binding
            .map(|binding| UiMountedProjectionSurface {
                surface,
                binding,
                audience: UiMountedProjectionAudience::full(),
            })
            .into_iter()
            .collect(),
    )
}

// Local equivalence evidence: no presentation or authority admission is claimed.
fn node() -> UiMountedProjectionNodeRecord {
    let withheld = UiMountedParticipationFact::new(UiMountedParticipationStatus::Withheld);
    UiMountedProjectionNodeRecord {
        receipt: UiMountedNodeReceipt::from_input(UiMountedNodeReceiptInput {
            mounted_instance: UiMountedInstanceIdentity::mint_unbound().unwrap(),
            graph_node: crate::graph::UiGraphNodeIdentity::new(1),
            semantic_surface: UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
            incarnation: UiMountIncarnation::mint_unbound().unwrap(),
            plan_digest: 7,
            role: UiMountedMechanicalRole::Control,
            participation: UiMountedParticipation::new(UiMountedParticipationInput {
                paint: withheld,
                clip: withheld,
                input: withheld,
                focus: withheld,
                hit_test: withheld,
                accessibility: withheld,
                motion: withheld,
                diagnostic: withheld,
            }),
            allocation: UiMountedAllocationProjection::Omitted(
                UiMountedOmissionReason::NoCommittedAllocation,
            ),
        }),
        plan_index: Some(0),
        surface_paint_order: Some(0),
        has_appearance_attachment: true,
        appearance_clip:
            crate::mounting::projection::appearance::UiMountedAppearanceClip::Unclipped,
        occurrence_allocation: UiMountedAllocationProjection::Omitted(
            UiMountedOmissionReason::NoCommittedAllocation,
        ),
        appearance_geometry:
            crate::mounting::projection::frame_storage::UiMountedAppearanceGeometry::from_occurrence(
                UiMountedAllocationProjection::Omitted(
                    UiMountedOmissionReason::NoCommittedAllocation,
                ),
                crate::mounting::projection::appearance::UiMountedAppearanceClip::Unclipped,
            ),
        static_paint: None,
        semantic_text: None,
        hit_test: None,
        focus_support: crate::capability::ComponentFocusSupport::not_focusable(),
        focus_scope: None,
        focus_container_owner: None,
        component_id: None,
        portal_child_owner: None,
    }
}
