use worth_ui_host_contract::{
    UiMountedAllocationProjection, UiMountedFilledRectCompletionInput, UiMountedFilledRectMechanic,
    UiMountedParticipationStatus, UiSurfaceBindingGeneration,
};

use super::super::frame_storage::UiMountedSemanticProjection;
use super::super::UiMountedProjectionDenial;

#[cfg(test)]
pub(in crate::mounting::projection) fn complete_static_filled_rects(
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    receipt_basis: &super::super::super::UiMountedNodeReceiptBasis,
    semantic: &UiMountedSemanticProjection,
) -> Result<Vec<UiMountedFilledRectMechanic>, UiMountedProjectionDenial> {
    let mut rows = Vec::new();
    for node in semantic.nodes_in_order() {
        if rows.len() >= worth_ui_host_contract::UiMountedFilledRectTable::MAX_ROWS {
            return Err(UiMountedProjectionDenial::StaticPaintCapacityExceeded);
        }
        if let Some(row) = complete_static_filled_rect(frame, receipt_basis, semantic, node)? {
            rows.push(row);
        }
    }
    Ok(rows)
}

pub(in crate::mounting::projection) fn complete_static_filled_rect(
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    receipt_basis: &super::super::super::UiMountedNodeReceiptBasis,
    semantic: &UiMountedSemanticProjection,
    node: &super::super::frame_storage::UiMountedProjectionNodeRecord,
) -> Result<Option<UiMountedFilledRectMechanic>, UiMountedProjectionDenial> {
    let Some(seed) = node.static_paint else {
        return Ok(None);
    };
    require_static_paint_participation(node)?;
    let (bounds, allocation_basis) = require_static_paint_allocation(node)?;
    let surface = semantic
        .surface_for(node.receipt.semantic_surface())
        .ok_or(UiMountedProjectionDenial::MissingSurfaceBinding)?;
    let mounted_instance = node.receipt.mounted_instance();
    let node_receipt = receipt_basis
        .receipt_for(mounted_instance)
        .ok_or(UiMountedProjectionDenial::StaticPaintNodeReceiptMismatch)?;
    UiMountedFilledRectMechanic::complete_from_runtime_mounting(
        UiMountedFilledRectCompletionInput {
            frame,
            surface: surface.surface,
            binding: surface.binding,
            mounted_instance,
            node_receipt,
            allocation_basis,
            bounds,
            color: seed.color(),
            layer_semantic_order: seed.layer_semantic_order(),
            clip_bounds: bounds,
        },
    )
    .map(Some)
    .map_err(UiMountedProjectionDenial::StaticPaintCompletion)
}

pub(in crate::mounting) fn reattribute_filled_rect(
    row: UiMountedFilledRectMechanic,
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    receipts: &super::super::super::UiMountedNodeReceiptBasis,
) -> Result<UiMountedFilledRectMechanic, UiMountedProjectionDenial> {
    let node_receipt = receipts
        .receipt_for(row.mounted_instance())
        .ok_or(UiMountedProjectionDenial::StaticPaintNodeReceiptMismatch)?;
    UiMountedFilledRectMechanic::complete_from_runtime_mounting(
        UiMountedFilledRectCompletionInput {
            frame,
            surface: row.surface(),
            binding: row.binding(),
            mounted_instance: row.mounted_instance(),
            node_receipt,
            allocation_basis: row.allocation_basis(),
            bounds: row.bounds(),
            color: row.color(),
            layer_semantic_order: row.layer_semantic_order(),
            clip_bounds: row.clip_bounds(),
        },
    )
    .map_err(UiMountedProjectionDenial::StaticPaintCompletion)
}

fn require_static_paint_participation(
    node: &super::super::frame_storage::UiMountedProjectionNodeRecord,
) -> Result<(), UiMountedProjectionDenial> {
    if node.receipt.participation().paint().status() == UiMountedParticipationStatus::Admitted
        && node.receipt.participation().clip().status() == UiMountedParticipationStatus::Admitted
    {
        return Ok(());
    }
    Err(UiMountedProjectionDenial::StaticPaintParticipationWithheld(
        node.receipt.graph_node(),
    ))
}

fn require_static_paint_allocation(
    node: &super::super::frame_storage::UiMountedProjectionNodeRecord,
) -> Result<
    (
        worth_ui_host_contract::UiMountedCanonicalBox,
        worth_ui_host_contract::UiMountedAllocationBasis,
    ),
    UiMountedProjectionDenial,
> {
    match node.presentation_allocation() {
        UiMountedAllocationProjection::Known { bounds, basis } => Ok((bounds, basis)),
        UiMountedAllocationProjection::PortalAnchorObservation { .. } => Err(
            UiMountedProjectionDenial::UnsupportedStaticPaintAllocation(node.receipt.graph_node()),
        ),
        UiMountedAllocationProjection::Omitted(_) => Err(
            UiMountedProjectionDenial::MissingStaticPaintAllocation(node.receipt.graph_node()),
        ),
    }
}

pub(in crate::mounting::projection) fn rebind_filled_rects(
    rows: &mut [UiMountedFilledRectMechanic],
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
        *row = UiMountedFilledRectMechanic::complete_from_runtime_mounting(
            UiMountedFilledRectCompletionInput {
                frame: row.frame(),
                surface: row.surface(),
                binding: replacement.binding_generation(),
                mounted_instance: row.mounted_instance(),
                node_receipt: row.node_receipt(),
                allocation_basis: row.allocation_basis(),
                bounds: row.bounds(),
                color: row.color(),
                layer_semantic_order: row.layer_semantic_order(),
                clip_bounds: row.clip_bounds(),
            },
        )
        .map_err(UiMountedProjectionDenial::StaticPaintCompletion)?;
    }
    Ok(())
}
