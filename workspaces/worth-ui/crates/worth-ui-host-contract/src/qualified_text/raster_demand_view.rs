//! Borrowed glyph-raster demand transport.
//!
//! Views name exact demand identity and the complete raster key. Layout and
//! original-range attribution stay off the key so safe reuse cannot erase
//! per-layout truth.

use super::{
    UiGlyphRasterExtent, UiGlyphRasterKey, UiGlyphRasterSource, UiQualifiedTextLayoutIdentity,
    UiTextOriginalRange, UiTextScaleGeneration,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiGlyphRasterDemandIdentity([u8; 32]);

/// Text-owner selection scope; damage-filtered records never prove full coverage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiGlyphRasterDemandScope {
    CompleteLayout,
    DamageFiltered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiGlyphRasterLane {
    Ordinary,
    Reconstruction,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiGlyphRasterAttribution {
    layout: UiQualifiedTextLayoutIdentity,
    original_range: UiTextOriginalRange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiGlyphRasterDemandRecord {
    key: UiGlyphRasterKey,
    attribution: UiGlyphRasterAttribution,
    extent: UiGlyphRasterExtent,
    staged_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiGlyphRasterDemandBatchView<'demand> {
    identity: UiGlyphRasterDemandIdentity,
    layout: UiQualifiedTextLayoutIdentity,
    dpi_milli: u32,
    text_scale: UiTextScaleGeneration,
    lane: UiGlyphRasterLane,
    scope: UiGlyphRasterDemandScope,
    records: &'demand [UiGlyphRasterDemandRecord],
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct UiGlyphRasterDemandBatchViewInput<'demand> {
    pub identity: UiGlyphRasterDemandIdentity,
    pub layout: UiQualifiedTextLayoutIdentity,
    pub dpi_milli: u32,
    pub text_scale: UiTextScaleGeneration,
    pub lane: UiGlyphRasterLane,
    pub scope: UiGlyphRasterDemandScope,
    pub records: &'demand [UiGlyphRasterDemandRecord],
}

impl UiGlyphRasterDemandIdentity {
    #[doc(hidden)]
    pub const fn from_text_mechanics(digest: [u8; 32]) -> Self {
        Self(digest)
    }

    pub const fn digest(self) -> [u8; 32] {
        self.0
    }
}

impl UiGlyphRasterAttribution {
    #[doc(hidden)]
    pub const fn from_text_mechanics(
        layout: UiQualifiedTextLayoutIdentity,
        original_range: UiTextOriginalRange,
    ) -> Self {
        Self {
            layout,
            original_range,
        }
    }

    pub const fn layout(self) -> UiQualifiedTextLayoutIdentity {
        self.layout
    }

    pub const fn original_range(self) -> UiTextOriginalRange {
        self.original_range
    }
}

impl UiGlyphRasterDemandRecord {
    #[doc(hidden)]
    pub fn from_text_mechanics(
        key: UiGlyphRasterKey,
        attribution: UiGlyphRasterAttribution,
        extent: UiGlyphRasterExtent,
    ) -> Option<Self> {
        let channels = match key.source() {
            UiGlyphRasterSource::ColorOutline | UiGlyphRasterSource::ColorBitmap => 4,
            UiGlyphRasterSource::AlphaOutline | UiGlyphRasterSource::LastResort => 1,
        };
        let staged_bytes = (extent.width() as u64)
            .checked_mul(extent.height() as u64)
            .and_then(|pixels| pixels.checked_mul(channels))?;
        Some(Self {
            key,
            attribution,
            extent,
            staged_bytes,
        })
    }

    pub const fn key(self) -> UiGlyphRasterKey {
        self.key
    }

    pub const fn attribution(self) -> UiGlyphRasterAttribution {
        self.attribution
    }

    pub const fn extent(self) -> UiGlyphRasterExtent {
        self.extent
    }

    pub const fn staged_bytes(self) -> u64 {
        self.staged_bytes
    }
}

impl<'demand> UiGlyphRasterDemandBatchView<'demand> {
    #[doc(hidden)]
    pub const fn from_text_mechanics(
        input: UiGlyphRasterDemandBatchViewInput<'demand>,
    ) -> Option<Self> {
        if input.dpi_milli == 0 {
            return None;
        }
        Some(Self {
            identity: input.identity,
            layout: input.layout,
            dpi_milli: input.dpi_milli,
            text_scale: input.text_scale,
            lane: input.lane,
            scope: input.scope,
            records: input.records,
        })
    }

    pub const fn identity(self) -> UiGlyphRasterDemandIdentity {
        self.identity
    }

    pub const fn layout_identity(self) -> UiQualifiedTextLayoutIdentity {
        self.layout
    }

    pub const fn dpi_milli(self) -> u32 {
        self.dpi_milli
    }

    pub const fn text_scale_generation(self) -> UiTextScaleGeneration {
        self.text_scale
    }

    pub const fn lane(self) -> UiGlyphRasterLane {
        self.lane
    }

    pub const fn scope(self) -> UiGlyphRasterDemandScope {
        self.scope
    }

    pub const fn records(self) -> &'demand [UiGlyphRasterDemandRecord] {
        self.records
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qualified_text::raster_key::{
        UiGlyphRasterFractionalOrigin, UiGlyphRasterKey, UiGlyphRasterKeyInput,
        UiGlyphRasterPalette, UiGlyphRasterSize, UiGlyphRasterSource, UiGlyphVariationCoordinates,
    };
    use crate::{
        UiFontCollectionGeneration, UiFontCollectionLineageIdentity, UiQualifiedFontFaceIdentity,
        UiTextProfileGeneration,
    };

    fn key() -> UiGlyphRasterKey {
        UiGlyphRasterKey::from_text_mechanics(UiGlyphRasterKeyInput {
            font_collection: UiFontCollectionGeneration::new(1).unwrap(),
            font_collection_lineage: UiFontCollectionLineageIdentity::from_text_mechanics([7; 32]),
            profile: UiTextProfileGeneration::new(1).unwrap(),
            face: UiQualifiedFontFaceIdentity::from_text_mechanics([4; 32], 0),
            glyph_id: 9,
            variations: UiGlyphVariationCoordinates::empty(),
            palette: UiGlyphRasterPalette::new(0),
            size: UiGlyphRasterSize::from_millipoints(12_000).unwrap(),
            source: UiGlyphRasterSource::ColorOutline,
            dpi_milli: 1_000,
            origin: UiGlyphRasterFractionalOrigin::from_sixty_fourths(0, 0),
        })
        .unwrap()
    }

    #[test]
    fn demand_view_keeps_key_and_layout_attribution_separate() {
        let layout = UiQualifiedTextLayoutIdentity::from_text_mechanics([1; 32]);
        let foreign = UiQualifiedTextLayoutIdentity::from_text_mechanics([2; 32]);
        let record = UiGlyphRasterDemandRecord::from_text_mechanics(
            key(),
            UiGlyphRasterAttribution::from_text_mechanics(
                layout,
                UiTextOriginalRange::new(0, 4).unwrap(),
            ),
            UiGlyphRasterExtent::new(9, 11).unwrap(),
        )
        .unwrap();
        let records = [record];
        let view =
            UiGlyphRasterDemandBatchView::from_text_mechanics(UiGlyphRasterDemandBatchViewInput {
                identity: UiGlyphRasterDemandIdentity::from_text_mechanics([5; 32]),
                layout,
                dpi_milli: 1_000,
                text_scale: UiTextScaleGeneration::new(3).unwrap(),
                lane: UiGlyphRasterLane::Ordinary,
                scope: UiGlyphRasterDemandScope::DamageFiltered,
                records: &records,
            })
            .unwrap();
        assert_eq!(view.records()[0].key().glyph_id(), 9);
        assert_eq!(view.records()[0].attribution().layout(), layout);
        assert_eq!(view.records()[0].extent().width(), 9);
        assert_eq!(view.records()[0].staged_bytes(), 9 * 11 * 4);
        assert_ne!(view.records()[0].attribution().layout(), foreign);
        assert_eq!(view.layout_identity(), layout);
        assert_eq!(view.identity().digest(), [5; 32]);
    }

    #[test]
    fn demand_view_borrows_records_without_owning_them() {
        let layout = UiQualifiedTextLayoutIdentity::from_text_mechanics([1; 32]);
        let record = UiGlyphRasterDemandRecord::from_text_mechanics(
            key(),
            UiGlyphRasterAttribution::from_text_mechanics(
                layout,
                UiTextOriginalRange::new(1, 2).unwrap(),
            ),
            UiGlyphRasterExtent::new(2, 3).unwrap(),
        )
        .unwrap();
        let records = [record];
        let view =
            UiGlyphRasterDemandBatchView::from_text_mechanics(UiGlyphRasterDemandBatchViewInput {
                identity: UiGlyphRasterDemandIdentity::from_text_mechanics([6; 32]),
                layout,
                dpi_milli: 96,
                text_scale: UiTextScaleGeneration::new(1).unwrap(),
                lane: UiGlyphRasterLane::Reconstruction,
                scope: UiGlyphRasterDemandScope::DamageFiltered,
                records: &records,
            })
            .unwrap();
        assert_eq!(view.records().as_ptr(), records.as_ptr());
        assert_eq!(view.lane(), UiGlyphRasterLane::Reconstruction);
    }
}
