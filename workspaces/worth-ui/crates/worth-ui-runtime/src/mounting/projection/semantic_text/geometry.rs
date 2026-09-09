use worth_ui_host_contract::UiMountedAllocationProjection;

use super::super::{frame_storage::UiMountedProjectionNodeRecord, UiMountedProjectionDenial};

pub(super) fn row_origin(
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    index: usize,
    total: usize,
) -> f32 {
    bounds.y() + bounds.height() * (index as f32 / total as f32)
}

pub(super) fn require_allocation(
    node: &UiMountedProjectionNodeRecord,
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
            UiMountedProjectionDenial::UnsupportedSemanticTextAllocation(node.receipt.graph_node()),
        ),
        UiMountedAllocationProjection::Omitted(_) => Err(
            UiMountedProjectionDenial::MissingSemanticTextAllocation(node.receipt.graph_node()),
        ),
    }
}
