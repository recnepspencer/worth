use worth_ui_host_contract::{
    UiMountedAllocationProjection, UiMountedGeometryPosture, UiMountedHitTestCompletionInput,
    UiMountedHitTestMechanic, UiMountedParticipationStatus, UiSurfaceBindingGeneration,
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
        UiMountedAllocationProjection::Known { bounds, .. } => off_the_grid(bounds, node)?,
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
    let bounds = if node.portal_child_owner.is_none() {
        super::super::frame_storage::surface_coordinates::viewport_bounds(
            bounds,
            surface.coordinate_posture,
        )?
    } else {
        bounds
    };
    let mounted_instance = node.receipt.mounted_instance();
    let node_receipt = receipt_basis
        .receipt_for(mounted_instance)
        .ok_or(UiMountedProjectionDenial::HitTestNodeReceiptMismatch)?;
    let clip_bounds = complete_clip_bounds(bounds, seed.clip())?;
    // Scrolling carries admitted regions past the viewport origin, and an inset
    // clip can consume a small one entirely. Neither can ever be hit, and the
    // posture the geometry already carries says so, so the frame omits the row
    // instead of failing: only malformed geometry is a denial.
    if bounds.posture() != UiMountedGeometryPosture::Area
        || clip_bounds.posture() != UiMountedGeometryPosture::Area
    {
        return Ok(None);
    }
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

/// The box on record under a painted allocation: scrolled content is painted
/// on the device grid, but hit testing reads it exactly where its offset put
/// it, as every later pose moves it.
fn off_the_grid(
    painted: worth_ui_host_contract::UiMountedCanonicalBox,
    node: &super::super::frame_storage::UiMountedProjectionNodeRecord,
) -> Result<worth_ui_host_contract::UiMountedCanonicalBox, UiMountedProjectionDenial> {
    let Some(correction) = node.grid_correction else {
        return Ok(painted);
    };
    correction
        .off_grid(painted)
        .map_err(UiMountedProjectionDenial::OccurrenceGeometry)
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

#[cfg(test)]
mod coordinate_tests {
    use super::*;
    use worth_ui_host_contract::{
        UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    };

    #[test]
    fn physical_surface_coordinates_cannot_be_relabelled_as_logical_pointer_bounds() {
        let bounds = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x: 80.0,
            y: 100.0,
            width: 360.0,
            height: 120.0,
            coordinate_space: UiMountedCoordinateSpace::HostSurface,
        })
        .unwrap();
        assert!(matches!(
            crate::mounting::projection::frame_storage::surface_coordinates::viewport_bounds(
                bounds,
                crate::mounting::UiSurfaceBindingCoordinatePosture::PhysicalPixels
            ),
            Err(UiMountedProjectionDenial::CoordinateBasisMismatch)
        ));
    }
}
