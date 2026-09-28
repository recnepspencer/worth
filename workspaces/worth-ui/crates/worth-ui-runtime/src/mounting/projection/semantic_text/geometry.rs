use worth_ui_host_contract::UiMountedAllocationProjection;

use super::super::{frame_storage::UiMountedProjectionNodeRecord, UiMountedProjectionDenial};

pub(super) fn row_origin(
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    index: usize,
    total: usize,
) -> f32 {
    bounds.y() + bounds.height() * (index as f32 / total as f32)
}

pub(in crate::mounting::projection) fn require_allocation(
    node: &UiMountedProjectionNodeRecord,
    surface: super::super::frame_storage::UiMountedProjectionSurface,
) -> Result<
    (
        worth_ui_host_contract::UiMountedCanonicalBox,
        worth_ui_host_contract::UiMountedAllocationBasis,
    ),
    UiMountedProjectionDenial,
> {
    match *node.occurrence_allocation.in_layout_space() {
        UiMountedAllocationProjection::Known { bounds, basis } => {
            let bounds = if node.portal_child_owner.is_none() {
                super::super::frame_storage::surface_coordinates::viewport_bounds(
                    bounds,
                    surface.coordinate_posture,
                )?
            } else {
                bounds
            };
            Ok((bounds, basis))
        }
        UiMountedAllocationProjection::PortalAnchorObservation { .. } => Err(
            UiMountedProjectionDenial::UnsupportedSemanticTextAllocation(node.receipt.graph_node()),
        ),
        UiMountedAllocationProjection::Omitted(_) => Err(
            UiMountedProjectionDenial::MissingSemanticTextAllocation(node.receipt.graph_node()),
        ),
    }
}
