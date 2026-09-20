//! Preparing one painted scroll-chrome rectangle for the native raster path.
//!
//! The runtime already snapped the rectangle to the device grid and already
//! intersected it with the region's clip, so nothing here moves it: this scales
//! logical millipoints to physical pixels exactly as the backdrop pipeline
//! does, and hands the result to the surface fill path so a thumb's corner
//! radii are rasterized by the same rounded-rectangle coverage the authored
//! surfaces use.

use worth_ui_host_contract::{
    UiMountedAppearanceColor, UiMountedScrollChromeIdentity, UiMountedScrollChromeMechanic,
};

use super::damage::UiNativeAppearanceDamageRect;
use super::geometry::{
    physical_radii, UiNativeAppearanceScale, UiNativeGeometryDenial, UiNativePhysicalPixelRect,
    UiNativePhysicalRect,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativeScrollChromePrimitive {
    identity: UiMountedScrollChromeIdentity,
    semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    paint_ordinal: u32,
    rect: UiNativePhysicalRect,
    clip: UiNativePhysicalPixelRect,
    background: UiMountedAppearanceColor,
    radii: [i64; 4],
    opacity: u16,
}

pub(crate) struct UiNativeScrollChromePipeline;

impl UiNativeScrollChromePipeline {
    pub(crate) fn prepare(
        mechanic: &UiMountedScrollChromeMechanic,
        scale: UiNativeAppearanceScale,
    ) -> Result<UiNativeScrollChromePrimitive, UiNativeGeometryDenial> {
        Ok(UiNativeScrollChromePrimitive {
            identity: mechanic.identity(),
            semantic_surface: mechanic.semantic_surface(),
            paint_ordinal: mechanic.identity().part().paint_ordinal(),
            rect: UiNativePhysicalRect::from_allocation(mechanic.rect(), scale)?,
            clip: UiNativePhysicalRect::from_clip(mechanic.clip(), scale)?.pixel_bounds(),
            background: mechanic.background(),
            radii: physical_radii(mechanic.radii().corners(), scale)?,
            opacity: mechanic.opacity().units(),
        })
    }
}

impl UiNativeScrollChromePrimitive {
    pub(crate) const fn identity(&self) -> UiMountedScrollChromeIdentity {
        self.identity
    }

    pub(crate) const fn semantic_surface(
        &self,
    ) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.semantic_surface
    }

    /// Zero for the track, one for the thumb: the order the two parts of one
    /// axis are painted in.
    pub(crate) const fn paint_ordinal(&self) -> u32 {
        self.paint_ordinal
    }

    pub(crate) const fn rect(&self) -> UiNativePhysicalRect {
        self.rect
    }

    pub(crate) const fn clip(&self) -> UiNativePhysicalPixelRect {
        self.clip
    }

    pub(crate) const fn background(&self) -> UiMountedAppearanceColor {
        self.background
    }

    pub(crate) const fn radii(&self) -> [i64; 4] {
        self.radii
    }

    pub(crate) const fn opacity(&self) -> u16 {
        self.opacity
    }

    pub(crate) fn damage_rect(&self) -> UiNativeAppearanceDamageRect {
        UiNativeAppearanceDamageRect::from_pixel_rect(self.rect.pixel_bounds())
    }

    /// Whether this rectangle covers a pixel once the region's clip is applied.
    pub(crate) fn paints_in_order(&self, pixel_x: i64, pixel_y: i64) -> bool {
        self.clip.contains(pixel_x, pixel_y) && self.rect.pixel_bounds().contains(pixel_x, pixel_y)
    }
}
