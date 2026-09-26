use worth_ui_host_contract::UiMountedCanonicalBox;

use super::UiMountedProjectionFrameOwner;

/// Viewport geometry of the painted surface of an appearance-only instance:
/// the target accepted Motion addresses when the instance owns no paint command.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiMountedAppearanceSurfaceSampleGeometry {
    bounds: UiMountedCanonicalBox,
    clip: UiMountedCanonicalBox,
}

impl UiMountedAppearanceSurfaceSampleGeometry {
    /// Full retained visual bounds, including paint currently outside the clip.
    pub(crate) const fn bounds(self) -> UiMountedCanonicalBox {
        self.bounds
    }

    pub(crate) const fn clip(self) -> UiMountedCanonicalBox {
        self.clip
    }

    /// A surface drawn at `bounds`, clipped to `clip`, as a test binds it.
    #[cfg(test)]
    pub(crate) const fn for_sampling_test(
        bounds: UiMountedCanonicalBox,
        clip: UiMountedCanonicalBox,
    ) -> Self {
        Self { bounds, clip }
    }
}

impl UiMountedProjectionFrameOwner {
    /// `None` when the instance paints no surface, has no completed
    /// allocation, or its ancestor clip is suppressed or unresolved. A surface
    /// outside a nonempty clip stays sampleable so Scroll can reveal it.
    pub(crate) fn appearance_surface_sample_geometry(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<UiMountedAppearanceSurfaceSampleGeometry> {
        let surface = self.appearance.retained_surface(instance)?;
        // The admitted mechanic's quantized, presented geometry.
        Some(UiMountedAppearanceSurfaceSampleGeometry {
            bounds: surface.visual_bounds().canonical_box()?,
            clip: surface.clip().canonical_box()?,
        })
    }
}
