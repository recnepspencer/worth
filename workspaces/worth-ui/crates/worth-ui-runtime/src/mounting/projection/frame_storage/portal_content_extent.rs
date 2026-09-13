use super::{UiMountedProjectionDenial, UiMountedProjectionFrame};
use worth_ui_host_contract::{
    UiMountedAllocationProjection, UiMountedCanonicalBox, UiMountedCanonicalBoxInput,
    UiMountedCoordinateSpace, UiMountedInstanceIdentity, UiSurfaceGeometry,
};

impl UiMountedProjectionFrame {
    /// Opening cost follows the selected owner's indexed children, including
    /// currently suppressed content. No unrelated surface is scanned.
    pub(in crate::mounting) fn portal_content_extent(
        &self,
        owner: UiMountedInstanceIdentity,
    ) -> Result<Option<crate::runtime::portal::UiPortalContentBounds>, UiMountedProjectionDenial>
    {
        let (children, _) = self.semantic.portal_children_for_owners(&[owner]);
        if children.is_empty() {
            return Ok(None);
        }
        let owner = self
            .semantic
            .node(owner)
            .ok_or(UiMountedProjectionDenial::PortalOverlayOwnerMissing)?;
        let bounds = |allocation| match allocation {
            UiMountedAllocationProjection::Known { bounds, .. }
            | UiMountedAllocationProjection::PortalAnchorObservation { bounds, .. } => Ok(bounds),
            UiMountedAllocationProjection::Omitted(_) => {
                Err(UiMountedProjectionDenial::PortalOverlayOwnerMissing)
            }
        };
        let anchor = bounds(owner.occurrence_allocation)?;
        let mut extent = [0.0_f32; 2];
        let mut layout = [f32::MAX, f32::MAX, 0.0_f32, 0.0_f32];
        let mut has_shadow = false;
        for instance in children {
            let child = self
                .semantic
                .node(instance)
                .ok_or(UiMountedProjectionDenial::PortalOverlayOwnerMissing)?;
            let inset = match child.surface_geometry {
                UiSurfaceGeometry::SoftShadow(shadow) => {
                    has_shadow = true;
                    shadow.sigma().subpixels() as f32 * 3.0 / 1_000.0
                }
                _ => 0.0,
            };
            let child = bounds(child.occurrence_allocation)?;
            if child.coordinate_space() != anchor.coordinate_space()
                || child.x() < anchor.x()
                || child.y() < anchor.y()
            {
                return Err(UiMountedProjectionDenial::NonFiniteGeometry);
            }
            extent[0] = extent[0].max(child.x() + child.width() - anchor.x());
            extent[1] = extent[1].max(child.y() + child.height() - anchor.y());
            if child.width() <= inset * 2.0 || child.height() <= inset * 2.0 {
                return Err(UiMountedProjectionDenial::NonFiniteGeometry);
            }
            layout[0] = layout[0].min(child.x() - anchor.x() + inset);
            layout[1] = layout[1].min(child.y() - anchor.y() + inset);
            layout[2] = layout[2].max(child.x() + child.width() - anchor.x() - inset);
            layout[3] = layout[3].max(child.y() + child.height() - anchor.y() - inset);
        }
        if extent
            .iter()
            .any(|value| *value <= 0.0 || value.ceil() > f32::from(u16::MAX))
        {
            return Err(UiMountedProjectionDenial::NonFiniteGeometry);
        }
        let paint = local_bounds([0.0, 0.0, extent[0].ceil(), extent[1].ceil()])?;
        let layout = if has_shadow {
            local_bounds([
                layout[0].floor(),
                layout[1].floor(),
                layout[2].ceil() - layout[0].floor(),
                layout[3].ceil() - layout[1].floor(),
            ])?
        } else {
            paint
        };
        Ok(Some(crate::runtime::portal::UiPortalContentBounds {
            layout,
            paint,
        }))
    }
}

fn local_bounds(
    [x, y, width, height]: [f32; 4],
) -> Result<UiMountedCanonicalBox, UiMountedProjectionDenial> {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space: UiMountedCoordinateSpace::GraphNodeLocal,
    })
    .map_err(|_| UiMountedProjectionDenial::NonFiniteGeometry)
}
