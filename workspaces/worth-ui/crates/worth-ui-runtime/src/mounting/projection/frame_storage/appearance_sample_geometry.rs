use worth_ui_host_contract::{
    UiMountedAllocationProjection, UiMountedCanonicalBox, UiMountedCanonicalBoxInput,
    UiMountedCoordinateSpace, UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT,
};

use super::UiMountedProjectionFrameOwner;
use crate::mounting::projection::appearance::UiMountedAppearanceClip;

/// Viewport geometry of the painted surface of an appearance-only instance:
/// the target accepted Motion addresses when the instance owns no paint command.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiMountedAppearanceSurfaceSampleGeometry {
    bounds: UiMountedCanonicalBox,
    clip: UiMountedCanonicalBox,
}

impl UiMountedAppearanceSurfaceSampleGeometry {
    /// The visible surface: its allocation inside its ancestor clip.
    pub(crate) const fn bounds(self) -> UiMountedCanonicalBox {
        self.bounds
    }

    pub(crate) const fn clip(self) -> UiMountedCanonicalBox {
        self.clip
    }
}

impl UiMountedProjectionFrameOwner {
    /// `None` when the instance paints no surface, has no completed
    /// allocation, or is suppressed or unresolved by its ancestor clip.
    pub(crate) fn appearance_surface_sample_geometry(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<UiMountedAppearanceSurfaceSampleGeometry> {
        if !self.appearance.has_surface_paint_for_instance(instance) {
            return None;
        }
        let geometry = self
            .projection
            .semantic
            .node(instance)?
            .completed_appearance_geometry();
        let allocation = match geometry.allocation {
            UiMountedAllocationProjection::Known { bounds, .. }
            | UiMountedAllocationProjection::PortalAnchorObservation { bounds, .. } => bounds,
            UiMountedAllocationProjection::Omitted(_) => return None,
        };
        let clip = match geometry.clip {
            UiMountedAppearanceClip::Unclipped => allocation,
            UiMountedAppearanceClip::Ancestor(clip) => {
                canonical_clip(clip, allocation.coordinate_space())?
            }
            UiMountedAppearanceClip::Suppressed | UiMountedAppearanceClip::Unresolved(_) => {
                return None
            }
        };
        Some(UiMountedAppearanceSurfaceSampleGeometry {
            bounds: allocation.intersection(clip)?,
            clip,
        })
    }
}

/// Appearance clips quantize the allocation's own coordinates, so they read
/// back in the allocation's space.
fn canonical_clip(
    clip: worth_ui_host_contract::UiAppearanceClip,
    coordinate_space: UiMountedCoordinateSpace,
) -> Option<UiMountedCanonicalBox> {
    let units = UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT as f32;
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: clip.x() as f32 / units,
        y: clip.y() as f32 / units,
        width: clip.width() as f32 / units,
        height: clip.height() as f32 / units,
        coordinate_space,
    })
    .ok()
}
