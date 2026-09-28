//! Native validation of the complete qualified raster key.
//!
//! Native does not invent raster meaning. It only admits or denies a key that
//! already names every profile identity field.

use worth_ui_host_contract::{UiGlyphRasterKey, UiGlyphRasterKeyEvidence};

use super::recovery::UiNativeTextAtlasDenial;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiNativeValidatedRasterKey {
    key: UiGlyphRasterKey,
}

impl UiNativeValidatedRasterKey {
    pub(crate) fn from_native_host(key: UiGlyphRasterKey) -> Result<Self, UiNativeTextAtlasDenial> {
        if key.dpi_milli() == 0 || key.size().millipoints() == 0 {
            return Err(UiNativeTextAtlasDenial::MalformedDemand);
        }
        if key.glyph_id() == u32::MAX {
            return Err(UiNativeTextAtlasDenial::MalformedDemand);
        }
        Ok(Self { key })
    }

    #[cfg(test)]
    pub const fn key(self) -> UiGlyphRasterKey {
        self.key
    }
}

/// Encodes every profile field in a stable byte order for deterministic
/// eviction.  This is a comparator representation, never a second identity.
pub(crate) fn canonical_raster_key_bytes(key: UiGlyphRasterKey) -> UiGlyphRasterKeyEvidence {
    key.canonical_evidence_bytes()
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiAtlasEntryIdentity(u64);

impl UiAtlasEntryIdentity {
    pub(crate) const fn from_native_host(value: u64) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self(value))
        }
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_ui_host_contract::{
        UiFontCollectionGeneration, UiFontCollectionLineageIdentity, UiGlyphRasterFractionalOrigin,
        UiGlyphRasterKeyInput, UiGlyphRasterPalette, UiGlyphRasterSize, UiGlyphRasterSource,
        UiGlyphVariationCoordinates, UiQualifiedFontFaceIdentity, UiTextProfileGeneration,
    };

    fn key(dpi_milli: u32) -> UiGlyphRasterKey {
        UiGlyphRasterKey::from_text_mechanics(UiGlyphRasterKeyInput {
            font_collection: UiFontCollectionGeneration::new(1).unwrap(),
            font_collection_lineage: UiFontCollectionLineageIdentity::from_text_mechanics([6; 32]),
            profile: UiTextProfileGeneration::new(1).unwrap(),
            face: UiQualifiedFontFaceIdentity::from_text_mechanics([2; 32], 0),
            glyph_id: 4,
            variations: UiGlyphVariationCoordinates::empty(),
            palette: UiGlyphRasterPalette::new(0),
            size: UiGlyphRasterSize::from_millipoints(12_000).unwrap(),
            source: UiGlyphRasterSource::AlphaOutline,
            dpi_milli,
            origin: UiGlyphRasterFractionalOrigin::from_sixty_fourths(0, 0),
        })
        .unwrap()
    }

    #[test]
    fn native_key_validation_requires_the_complete_profile_fields() {
        let admitted = UiNativeValidatedRasterKey::from_native_host(key(1_000)).unwrap();
        assert_eq!(admitted.key().glyph_id(), 4);
        assert_eq!(admitted.key().font_collection_generation().get(), 1);
        assert_eq!(admitted.key().profile_generation().get(), 1);
        assert_eq!(admitted.key().palette().index(), 0);
        assert_eq!(admitted.key().size().millipoints(), 12_000);
        assert_eq!(admitted.key().source(), UiGlyphRasterSource::AlphaOutline);
        assert_eq!(admitted.key().fractional_origin().x_over_64(), 0);
    }
}
