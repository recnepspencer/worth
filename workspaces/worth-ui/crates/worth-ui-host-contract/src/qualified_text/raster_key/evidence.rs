//! Canonical evidence bytes of a qualified glyph-raster key.
//!
//! The encoding joins one key across Runtime and the native atlas owner. It is
//! held inline, so encoding a key never allocates, and it seeds the key's
//! fingerprint, so hashing a key never re-reads its digests.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use super::{UiGlyphRasterKey, UiGlyphRasterSource, MAX_VARIATION_AXES};

const FIXED_EVIDENCE_BYTES: usize = 136;
const VARIATION_EVIDENCE_BYTES: usize = 8;
const MAX_EVIDENCE_BYTES: usize =
    FIXED_EVIDENCE_BYTES + MAX_VARIATION_AXES * VARIATION_EVIDENCE_BYTES;

/// A key's canonical evidence: every profile field in a stable byte order.
/// It is evidence, not a second identity; it orders and compares as its
/// bytes.
#[derive(Clone, Copy)]
pub struct UiGlyphRasterKeyEvidence {
    bytes: [u8; MAX_EVIDENCE_BYTES],
    len: u8,
}

impl UiGlyphRasterKeyEvidence {
    pub(super) const fn encode(key: &UiGlyphRasterKey) -> Self {
        let mut evidence = Self {
            bytes: [0; MAX_EVIDENCE_BYTES],
            len: 0,
        };
        evidence.put(&key.font_collection.get().to_le_bytes());
        evidence.put(&key.font_collection_lineage.digest());
        evidence.put(&key.profile.get().to_le_bytes());
        evidence.put(&key.face.font_bytes_digest());
        evidence.put(&key.face.face_index().to_le_bytes());
        evidence.put(&key.face.selection_digest());
        evidence.put(&key.glyph_id.to_le_bytes());
        evidence.put(&[key.variations.len() as u8]);
        let mut axis = 0;
        while axis < key.variations.len() {
            if let Some(variation) = key.variations.axes[axis] {
                evidence.put(&variation.axis());
                evidence.put(&variation.value_milli().to_le_bytes());
            }
            axis += 1;
        }
        evidence.put(&key.palette.index().to_le_bytes());
        evidence.put(&key.size.millipoints().to_le_bytes());
        evidence.put(&[match key.source {
            UiGlyphRasterSource::ColorOutline => 0,
            UiGlyphRasterSource::ColorBitmap => 1,
            UiGlyphRasterSource::AlphaOutline => 2,
            UiGlyphRasterSource::LastResort => 3,
        }]);
        evidence.put(&key.dpi_milli.to_le_bytes());
        evidence.put(&key.origin.x_over_64().to_le_bytes());
        evidence.put(&key.origin.y_over_64().to_le_bytes());
        evidence
    }

    const fn put(&mut self, part: &[u8]) {
        let mut index = 0;
        while index < part.len() {
            self.bytes[self.len as usize] = part[index];
            self.len += 1;
            index += 1;
        }
    }

    /// A 64-bit digest of the evidence for hashing only. Equal keys encode
    /// equal evidence, so they always share it.
    pub(super) const fn fingerprint(&self) -> u64 {
        const MULTIPLIER: u64 = 0x9e37_79b9_7f4a_7c15;
        let mut hash = self.len as u64;
        let mut index = 0;
        while index < self.len as usize {
            let mut word = 0_u64;
            let mut byte = 0;
            while byte < 8 && index + byte < self.len as usize {
                word |= (self.bytes[index + byte] as u64) << (byte * 8);
                byte += 1;
            }
            hash = (hash ^ word).wrapping_mul(MULTIPLIER).rotate_left(29);
            index += 8;
        }
        hash ^= hash >> 32;
        hash.wrapping_mul(MULTIPLIER)
    }
}

impl Deref for UiGlyphRasterKeyEvidence {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.bytes[..usize::from(self.len)]
    }
}

impl AsRef<[u8]> for UiGlyphRasterKeyEvidence {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl PartialEq for UiGlyphRasterKeyEvidence {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl Eq for UiGlyphRasterKeyEvidence {}

impl Hash for UiGlyphRasterKeyEvidence {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

/// Shows only the encoded bytes, never the unused capacity.
impl fmt::Debug for UiGlyphRasterKeyEvidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("UiGlyphRasterKeyEvidence")
            .field(&&**self)
            .finish()
    }
}

impl PartialOrd for UiGlyphRasterKeyEvidence {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UiGlyphRasterKeyEvidence {
    fn cmp(&self, other: &Self) -> Ordering {
        (**self).cmp(&**other)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    use crate::{
        UiFontCollectionGeneration, UiFontCollectionLineageIdentity, UiGlyphRasterFractionalOrigin,
        UiGlyphRasterKey, UiGlyphRasterKeyInput, UiGlyphRasterPalette, UiGlyphRasterSize,
        UiGlyphRasterSource, UiGlyphVariationCoordinates, UiQualifiedFontFaceIdentity,
        UiQualifiedTextVariationRecord, UiTextProfileGeneration,
    };

    fn key(glyph_id: u32, axes: usize) -> UiGlyphRasterKey {
        let records = [UiQualifiedTextVariationRecord::from_text_mechanics(*b"wght", -7_000); 8];
        UiGlyphRasterKey::from_text_mechanics(UiGlyphRasterKeyInput {
            font_collection: UiFontCollectionGeneration::new(3).unwrap(),
            font_collection_lineage: UiFontCollectionLineageIdentity::from_text_mechanics([6; 32]),
            profile: UiTextProfileGeneration::new(5).unwrap(),
            face: UiQualifiedFontFaceIdentity::from_text_mechanics([2; 32], 1),
            glyph_id,
            variations: UiGlyphVariationCoordinates::from_records(&records[..axes]).unwrap(),
            palette: UiGlyphRasterPalette::new(4),
            size: UiGlyphRasterSize::from_millipoints(12_500).unwrap(),
            source: UiGlyphRasterSource::ColorBitmap,
            dpi_milli: 1_250,
            origin: UiGlyphRasterFractionalOrigin::from_sixty_fourths(-3, 17),
        })
        .unwrap()
    }

    /// The field-by-field encoding the inline evidence must reproduce.
    fn heap_encoding(key: UiGlyphRasterKey) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&key.font_collection_generation().get().to_le_bytes());
        bytes.extend_from_slice(&key.font_collection_lineage().digest());
        bytes.extend_from_slice(&key.profile_generation().get().to_le_bytes());
        bytes.extend_from_slice(&key.face().font_bytes_digest());
        bytes.extend_from_slice(&key.face().face_index().to_le_bytes());
        bytes.extend_from_slice(&key.face().selection_digest());
        bytes.extend_from_slice(&key.glyph_id().to_le_bytes());
        bytes.push(key.variations().len() as u8);
        for variation in key.variations().records() {
            bytes.extend_from_slice(&variation.axis());
            bytes.extend_from_slice(&variation.value_milli().to_le_bytes());
        }
        bytes.extend_from_slice(&key.palette().index().to_le_bytes());
        bytes.extend_from_slice(&key.size().millipoints().to_le_bytes());
        bytes.push(1); // ColorBitmap
        bytes.extend_from_slice(&key.dpi_milli().to_le_bytes());
        bytes.extend_from_slice(&key.fractional_origin().x_over_64().to_le_bytes());
        bytes.extend_from_slice(&key.fractional_origin().y_over_64().to_le_bytes());
        bytes
    }

    fn hash_of(key: UiGlyphRasterKey) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn inline_evidence_matches_the_field_encoding_for_every_axis_count() {
        for axes in 0..=8 {
            let evidence = key(9, axes).canonical_evidence_bytes();
            assert_eq!(&*evidence, heap_encoding(key(9, axes)).as_slice());
            assert_eq!(evidence.len(), 136 + 8 * axes);
        }
        let (low, high) = (key(0x100, 0), key(0x001, 0));
        let bytes = (heap_encoding(low), heap_encoding(high));
        assert_eq!(
            low.canonical_evidence_bytes()
                .cmp(&high.canonical_evidence_bytes()),
            bytes.0.cmp(&bytes.1),
            "evidence orders exactly as its bytes"
        );
    }

    #[test]
    fn equal_keys_hash_equal_and_single_field_changes_move_the_fingerprint() {
        assert_eq!(key(9, 2), key(9, 2));
        assert_eq!(hash_of(key(9, 2)), hash_of(key(9, 2)));
        let base = key(9, 2).fingerprint;
        for other in [key(10, 2), key(9, 3), key(9, 0)] {
            assert_ne!(other.fingerprint, base);
            assert_ne!(other, key(9, 2));
        }
    }
}
