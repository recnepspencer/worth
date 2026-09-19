use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT,
};

use super::UiMountedProjectionFrameOwner;

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
        let surface = self.appearance.retained_surface(instance)?;
        let visual = surface.visual_bounds();
        let bounds = canonical_box(visual.x(), visual.y(), visual.width(), visual.height())?;
        let clip = surface.clip();
        let clip = canonical_box(clip.x(), clip.y(), clip.width(), clip.height())?;
        Some(UiMountedAppearanceSurfaceSampleGeometry {
            bounds: bounds.intersection(clip)?,
            clip,
        })
    }
}

/// Use the admitted mechanic's quantized, presented geometry. Both hosts
/// interpret appearance mechanics in viewport coordinates, including Portal children.
fn canonical_box(x: i32, y: i32, width: u32, height: u32) -> Option<UiMountedCanonicalBox> {
    let units = UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT as f32;
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: x as f32 / units,
        y: y as f32 / units,
        width: width as f32 / units,
        height: height as f32 / units,
        coordinate_space: UiMountedCoordinateSpace::Viewport,
    })
    .ok()
}
