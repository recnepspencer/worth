use worth_ui_host_contract::{
    UiMountedAppearanceColor, UiMountedBackdropIdentity, UiMountedBackdropMechanic,
};

use super::damage::UiNativeAppearanceDamageRect;
use super::geometry::{
    UiNativeAppearanceScale, UiNativeGeometryDenial, UiNativePhysicalPixelRect,
    UiNativePhysicalRect,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiNativeBackdropPrimitive {
    identity: UiMountedBackdropIdentity,
    semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ordinal: u32,
    extent: UiNativePhysicalRect,
    clip: UiNativePhysicalPixelRect,
    background: UiMountedAppearanceColor,
    opacity: u16,
}

pub(crate) struct UiNativeBackdropPipeline;

impl UiNativeBackdropPipeline {
    pub(crate) fn prepare(
        mechanic: &UiMountedBackdropMechanic,
        scale: UiNativeAppearanceScale,
    ) -> Result<UiNativeBackdropPrimitive, UiNativeGeometryDenial> {
        Ok(UiNativeBackdropPrimitive {
            identity: mechanic.identity().clone(),
            semantic_surface: mechanic.semantic_surface(),
            ordinal: mechanic.placement().ordinal(),
            extent: UiNativePhysicalRect::from_extent(mechanic.extent(), scale)?,
            clip: UiNativePhysicalRect::from_clip(mechanic.clip(), scale)?.pixel_bounds(),
            background: mechanic.background(),
            opacity: mechanic.opacity().units(),
        })
    }
}

impl UiNativeBackdropPrimitive {
    pub(crate) fn identity(&self) -> &UiMountedBackdropIdentity {
        &self.identity
    }

    pub(crate) fn semantic_surface(&self) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.semantic_surface
    }

    pub(crate) fn ordinal(&self) -> u32 {
        self.ordinal
    }

    pub(crate) fn extent(&self) -> UiNativePhysicalRect {
        self.extent
    }

    pub(crate) fn clip(&self) -> UiNativePhysicalPixelRect {
        self.clip
    }

    pub(crate) fn background(&self) -> UiMountedAppearanceColor {
        self.background
    }

    pub(crate) fn opacity(&self) -> u16 {
        self.opacity
    }

    pub(crate) fn damage_rect(&self) -> UiNativeAppearanceDamageRect {
        UiNativeAppearanceDamageRect::from_pixel_rect(self.extent.pixel_bounds())
    }

    pub(crate) fn paints_in_order(&self, pixel_x: i64, pixel_y: i64) -> bool {
        self.clip.contains(pixel_x, pixel_y)
            && self.extent.pixel_bounds().contains(pixel_x, pixel_y)
    }
}
