use worth_ui_host_contract::{
    UiMountedAllocationProjection, UiMountedHitTestCompletionInput, UiMountedHitTestMechanic,
    UiMountedParticipationStatus, UiSurfaceBindingGeneration,
};

use super::super::frame_storage::UiMountedSemanticProjection;
use super::super::UiMountedProjectionDenial;

pub(in crate::mounting::projection) fn complete_hit_test(
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    receipt_basis: &super::super::super::UiMountedNodeReceiptBasis,
    semantic: &UiMountedSemanticProjection,
    node: &super::super::frame_storage::UiMountedProjectionNodeRecord,
) -> Result<Option<UiMountedHitTestMechanic>, UiMountedProjectionDenial> {
    let Some(seed) = node.hit_test else {
        if node.receipt.participation().hit_test().status()
            == UiMountedParticipationStatus::Admitted
        {
            return Err(UiMountedProjectionDenial::MissingHitTestOrder(
                node.receipt.graph_node(),
            ));
        }
        return Ok(None);
    };
    if node.receipt.participation().hit_test().status() != UiMountedParticipationStatus::Admitted {
        return Err(UiMountedProjectionDenial::HitTestParticipationWithheld(
            node.receipt.graph_node(),
        ));
    }
    let bounds = match node.presentation_allocation() {
        UiMountedAllocationProjection::Known { bounds, .. } => bounds,
        UiMountedAllocationProjection::PortalAnchorObservation { .. } => {
            return Err(UiMountedProjectionDenial::UnsupportedHitTestAllocation(
                node.receipt.graph_node(),
            ));
        }
        UiMountedAllocationProjection::Omitted(_) => {
            return Err(UiMountedProjectionDenial::MissingHitTestAllocation(
                node.receipt.graph_node(),
            ));
        }
    };
    let surface = semantic
        .surface_for(node.receipt.semantic_surface())
        .ok_or(UiMountedProjectionDenial::MissingSurfaceBinding)?;
    let mounted_instance = node.receipt.mounted_instance();
    let node_receipt = receipt_basis
        .receipt_for(mounted_instance)
        .ok_or(UiMountedProjectionDenial::HitTestNodeReceiptMismatch)?;
    let clip_bounds = complete_clip_bounds(bounds, seed.clip())?;
    UiMountedHitTestMechanic::complete_from_runtime_mounting(UiMountedHitTestCompletionInput {
        frame,
        surface: surface.surface,
        binding: surface.binding,
        mounted_instance,
        node_receipt,
        bounds,
        clip_bounds,
        order: seed.order(),
    })
    .map(Some)
    .map_err(UiMountedProjectionDenial::HitTestCompletion)
}

pub(in crate::mounting) fn reattribute_hit_test(
    row: UiMountedHitTestMechanic,
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    receipts: &super::super::super::UiMountedNodeReceiptBasis,
) -> Result<UiMountedHitTestMechanic, UiMountedProjectionDenial> {
    reattribute_hit_test_with_probes(row, frame, receipts).map(|(row, _)| row)
}

pub(in crate::mounting) fn reattribute_hit_test_with_probes(
    row: UiMountedHitTestMechanic,
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    receipts: &super::super::super::UiMountedNodeReceiptBasis,
) -> Result<(UiMountedHitTestMechanic, usize), UiMountedProjectionDenial> {
    let (receipt, probes) = receipts.receipt_for_with_probes(row.mounted_instance());
    let node_receipt = receipt.ok_or(UiMountedProjectionDenial::HitTestNodeReceiptMismatch)?;
    UiMountedHitTestMechanic::complete_from_runtime_mounting(UiMountedHitTestCompletionInput {
        frame,
        surface: row.surface(),
        binding: row.binding(),
        mounted_instance: row.mounted_instance(),
        node_receipt,
        bounds: row.bounds(),
        clip_bounds: row.clip_bounds(),
        order: row.order(),
    })
    .map(|row| (row, probes))
    .map_err(UiMountedProjectionDenial::HitTestCompletion)
}

fn complete_clip_bounds(
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    clip: crate::capability::ComponentHitTestClipContract,
) -> Result<worth_ui_host_contract::UiMountedCanonicalBox, UiMountedProjectionDenial> {
    let crate::capability::ComponentHitTestClipContract::Inset(inset) = clip else {
        return Ok(bounds);
    };
    let horizontal = f32::from(inset.horizontal_logical_points());
    let vertical = f32::from(inset.vertical_logical_points());
    worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: bounds.x() + horizontal,
            y: bounds.y() + vertical,
            width: (bounds.width() - 2.0 * horizontal).max(0.0),
            height: (bounds.height() - 2.0 * vertical).max(0.0),
            coordinate_space: bounds.coordinate_space(),
        },
    )
    .map_err(|denial| match denial {
        worth_ui_host_contract::UiMountedGeometryDenial::NonFinite => {
            UiMountedProjectionDenial::NonFiniteGeometry
        }
        worth_ui_host_contract::UiMountedGeometryDenial::NegativeExtent => {
            UiMountedProjectionDenial::NegativeExtent
        }
    })
}

pub(in crate::mounting::projection) fn rebind_hit_tests(
    rows: &mut [UiMountedHitTestMechanic],
    replacements: &[(
        UiSurfaceBindingGeneration,
        super::super::super::UiSurfaceBindingIdentityView,
    )],
) -> Result<(), UiMountedProjectionDenial> {
    for row in rows {
        let Some((_, replacement)) = replacements
            .iter()
            .find(|(affected, _)| *affected == row.binding())
        else {
            continue;
        };
        if replacement.semantic_surface_identity() != row.surface() {
            return Err(UiMountedProjectionDenial::MissingSurfaceBinding);
        }
        *row = UiMountedHitTestMechanic::complete_from_runtime_mounting(
            UiMountedHitTestCompletionInput {
                frame: row.frame(),
                surface: row.surface(),
                binding: replacement.binding_generation(),
                mounted_instance: row.mounted_instance(),
                node_receipt: row.node_receipt(),
                bounds: row.bounds(),
                clip_bounds: row.clip_bounds(),
                order: row.order(),
            },
        )
        .map_err(UiMountedProjectionDenial::HitTestCompletion)?;
    }
    Ok(())
}
