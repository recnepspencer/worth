//! Inert qualified glyph-raster key components shared across text and native.
//!
//! These values identify raster equivalence. They do not grant cache, atlas,
//! pin, upload, or raster-production authority.

use std::hash::{Hash, Hasher};

use super::{
    UiFontCollectionGeneration, UiFontCollectionLineageIdentity, UiQualifiedFontFaceIdentity,
    UiQualifiedTextVariationRecord, UiTextProfileGeneration,
};

mod evidence;

pub use evidence::UiGlyphRasterKeyEvidence;

const MAX_VARIATION_AXES: usize = 8;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UiGlyphRasterSource {
    ColorOutline,
    ColorBitmap,
    AlphaOutline,
    LastResort,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiGlyphRasterPalette {
    index: u16,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiGlyphRasterSize {
    millipoints: u32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiGlyphRasterFractionalOrigin {
    x_over_64: i16,
    y_over_64: i16,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiGlyphVariationCoordinates {
    axes: [Option<UiQualifiedTextVariationRecord>; MAX_VARIATION_AXES],
    len: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiGlyphRasterKey {
    /// Derived from every other field, so it is the only part hashed, and
    /// declared first so unequal keys usually differ at the first compare.
    fingerprint: u64,
    font_collection: UiFontCollectionGeneration,
    font_collection_lineage: UiFontCollectionLineageIdentity,
    profile: UiTextProfileGeneration,
    face: UiQualifiedFontFaceIdentity,
    glyph_id: u32,
    variations: UiGlyphVariationCoordinates,
    palette: UiGlyphRasterPalette,
    size: UiGlyphRasterSize,
    source: UiGlyphRasterSource,
    dpi_milli: u32,
    origin: UiGlyphRasterFractionalOrigin,
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct UiGlyphRasterKeyInput {
    pub font_collection: UiFontCollectionGeneration,
    pub font_collection_lineage: UiFontCollectionLineageIdentity,
    pub profile: UiTextProfileGeneration,
    pub face: UiQualifiedFontFaceIdentity,
    pub glyph_id: u32,
    pub variations: UiGlyphVariationCoordinates,
    pub palette: UiGlyphRasterPalette,
    pub size: UiGlyphRasterSize,
    pub source: UiGlyphRasterSource,
    pub dpi_milli: u32,
    pub origin: UiGlyphRasterFractionalOrigin,
}

impl UiGlyphRasterPalette {
    pub const fn new(index: u16) -> Self {
        Self { index }
    }

    pub const fn index(self) -> u16 {
        self.index
    }
}

impl UiGlyphRasterSize {
    pub const fn from_millipoints(millipoints: u32) -> Option<Self> {
        if millipoints == 0 {
            None
        } else {
            Some(Self { millipoints })
        }
    }

    pub const fn millipoints(self) -> u32 {
        self.millipoints
    }
}

impl UiGlyphRasterFractionalOrigin {
    pub const fn from_sixty_fourths(x_over_64: i16, y_over_64: i16) -> Self {
        Self {
            x_over_64,
            y_over_64,
        }
    }

    pub const fn x_over_64(self) -> i16 {
        self.x_over_64
    }

    pub const fn y_over_64(self) -> i16 {
        self.y_over_64
    }
}

impl UiGlyphVariationCoordinates {
    pub const fn empty() -> Self {
        Self {
            axes: [None; MAX_VARIATION_AXES],
            len: 0,
        }
    }

    pub fn from_records(records: &[UiQualifiedTextVariationRecord]) -> Option<Self> {
        if records.len() > MAX_VARIATION_AXES {
            return None;
        }
        let mut axes = [None; MAX_VARIATION_AXES];
        let mut index = 0;
        while index < records.len() {
            axes[index] = Some(records[index]);
            index += 1;
        }
        Some(Self {
            axes,
            len: index as u8,
        })
    }

    pub fn records(self) -> impl Iterator<Item = UiQualifiedTextVariationRecord> {
        self.axes.into_iter().flatten()
    }

    pub const fn len(self) -> usize {
        self.len as usize
    }

    pub const fn is_empty(self) -> bool {
        self.len == 0
    }
}

impl UiGlyphRasterKey {
    #[doc(hidden)]
    pub const fn from_text_mechanics(input: UiGlyphRasterKeyInput) -> Option<Self> {
        if input.dpi_milli == 0 {
            None
        } else {
            let mut key = Self {
                fingerprint: 0,
                font_collection: input.font_collection,
                font_collection_lineage: input.font_collection_lineage,
                profile: input.profile,
                face: input.face,
                glyph_id: input.glyph_id,
                variations: input.variations,
                palette: input.palette,
                size: input.size,
                source: input.source,
                dpi_milli: input.dpi_milli,
                origin: input.origin,
            };
            key.fingerprint = UiGlyphRasterKeyEvidence::encode(&key).fingerprint();
            Some(key)
        }
    }

    pub const fn font_collection_generation(self) -> UiFontCollectionGeneration {
        self.font_collection
    }

    pub const fn font_collection_lineage(self) -> UiFontCollectionLineageIdentity {
        self.font_collection_lineage
    }

    pub const fn profile_generation(self) -> UiTextProfileGeneration {
        self.profile
    }

    pub const fn face(self) -> UiQualifiedFontFaceIdentity {
        self.face
    }

    pub const fn glyph_id(self) -> u32 {
        self.glyph_id
    }

    pub const fn variations(self) -> UiGlyphVariationCoordinates {
        self.variations
    }

    pub const fn palette(self) -> UiGlyphRasterPalette {
        self.palette
    }

    pub const fn size(self) -> UiGlyphRasterSize {
        self.size
    }

    pub const fn source(self) -> UiGlyphRasterSource {
        self.source
    }

    pub const fn dpi_milli(self) -> u32 {
        self.dpi_milli
    }

    pub const fn fractional_origin(self) -> UiGlyphRasterFractionalOrigin {
        self.origin
    }

    /// Stable boundary representation used to join the same qualified key
    /// across Runtime and the native atlas owner. Its byte order is a
    /// deterministic tie-break, never a second identity.
    pub const fn canonical_evidence_bytes(self) -> UiGlyphRasterKeyEvidence {
        UiGlyphRasterKeyEvidence::encode(&self)
    }
}

impl Hash for UiGlyphRasterKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.fingerprint);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_key_with_lineage(lineage: [u8; 32]) -> UiGlyphRasterKey {
        let variation =
            crate::UiQualifiedTextVariationRecord::from_text_mechanics(*b"wght", 400_000);
        UiGlyphRasterKey::from_text_mechanics(UiGlyphRasterKeyInput {
            font_collection: UiFontCollectionGeneration::new(1).unwrap(),
            font_collection_lineage: UiFontCollectionLineageIdentity::from_text_mechanics(lineage),
            profile: UiTextProfileGeneration::new(1).unwrap(),
            face: crate::UiQualifiedFontFaceIdentity::from_text_mechanics([3; 32], 0),
            glyph_id: 17,
            variations: UiGlyphVariationCoordinates::from_records(&[variation]).unwrap(),
            palette: UiGlyphRasterPalette::new(2),
            size: UiGlyphRasterSize::from_millipoints(14_000).unwrap(),
            source: UiGlyphRasterSource::AlphaOutline,
            dpi_milli: 1_500,
            origin: UiGlyphRasterFractionalOrigin::from_sixty_fourths(32, -16),
        })
        .unwrap()
    }

    fn sample_key() -> UiGlyphRasterKey {
        sample_key_with_lineage([8; 32])
    }

    #[test]
    fn raster_key_carries_every_profile_field() {
        let key = sample_key();
        assert_eq!(key.font_collection_generation().get(), 1);
        assert_eq!(key.profile_generation().get(), 1);
        assert_eq!(key.face().face_index(), 0);
        assert_eq!(key.glyph_id(), 17);
        assert_eq!(key.variations().len(), 1);
        assert_eq!(key.palette().index(), 2);
        assert_eq!(key.size().millipoints(), 14_000);
        assert_eq!(key.source(), UiGlyphRasterSource::AlphaOutline);
        assert_eq!(key.dpi_milli(), 1_500);
        assert_eq!(key.fractional_origin().x_over_64(), 32);
        assert_eq!(key.fractional_origin().y_over_64(), -16);
    }

    #[test]
    fn raster_key_rejects_zero_dpi_and_oversized_variation_sets() {
        let mut input = UiGlyphRasterKeyInput {
            font_collection: UiFontCollectionGeneration::new(1).unwrap(),
            font_collection_lineage: UiFontCollectionLineageIdentity::from_text_mechanics([8; 32]),
            profile: UiTextProfileGeneration::new(1).unwrap(),
            face: crate::UiQualifiedFontFaceIdentity::from_text_mechanics([3; 32], 0),
            glyph_id: 1,
            variations: UiGlyphVariationCoordinates::empty(),
            palette: UiGlyphRasterPalette::new(0),
            size: UiGlyphRasterSize::from_millipoints(12_000).unwrap(),
            source: UiGlyphRasterSource::LastResort,
            dpi_milli: 0,
            origin: UiGlyphRasterFractionalOrigin::from_sixty_fourths(0, 0),
        };
        assert!(UiGlyphRasterKey::from_text_mechanics(input).is_none());
        input.dpi_milli = 96;
        let extra = [crate::UiQualifiedTextVariationRecord::from_text_mechanics(*b"wght", 1); 9];
        assert!(UiGlyphVariationCoordinates::from_records(&extra).is_none());
        assert!(UiGlyphRasterKey::from_text_mechanics(input).is_some());
    }

    #[test]
    fn same_generation_different_collection_lineage_is_not_reusable() {
        let first = sample_key_with_lineage([8; 32]);
        let second = sample_key_with_lineage([9; 32]);
        assert_eq!(
            first.font_collection_generation(),
            second.font_collection_generation()
        );
        assert_ne!(
            first.font_collection_lineage(),
            second.font_collection_lineage()
        );
        assert_ne!(first, second);
    }
}
