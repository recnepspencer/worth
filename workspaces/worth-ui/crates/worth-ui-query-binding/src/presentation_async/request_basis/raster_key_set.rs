use worth_ui_host_contract::{UiGlyphRasterKey, UiGlyphRasterSource};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct WorthUiPresentationRasterKeySetBasis {
    keys: Box<[UiGlyphRasterKey]>,
}

impl WorthUiPresentationRasterKeySetBasis {
    pub(crate) fn from_runtime(mut keys: Vec<UiGlyphRasterKey>) -> Self {
        keys.sort_by_cached_key(key_sort_key);
        keys.dedup();
        Self {
            keys: keys.into_boxed_slice(),
        }
    }

    pub fn keys(&self) -> &[UiGlyphRasterKey] {
        &self.keys
    }

    pub fn contains(&self, key: UiGlyphRasterKey) -> bool {
        self.keys.contains(&key)
    }
}

type RasterFaceFacts = (u64, [u8; 32], u64, [u8; 32], u32, [u8; 32]);
type RasterGlyphFacts = (u32, u16, u32, u64, u32, u16, u16);
pub(super) type RasterKeySortKey = (RasterFaceFacts, RasterGlyphFacts, Vec<([u8; 4], u32)>);

/// Keys order by their fields in identity encoding order, then variation
/// axes; the key is typed so sorting formats nothing.
pub(super) fn key_sort_key(key: &UiGlyphRasterKey) -> RasterKeySortKey {
    let face = key.face();
    let origin = key.fractional_origin();
    (
        (
            key.font_collection_generation().get(),
            key.font_collection_lineage().digest(),
            key.profile_generation().get(),
            face.font_bytes_digest(),
            face.face_index(),
            face.selection_digest(),
        ),
        (
            key.glyph_id(),
            key.palette().index(),
            key.size().millipoints(),
            raster_source_ordinal(key.source()),
            key.dpi_milli(),
            origin.x_over_64() as u16,
            origin.y_over_64() as u16,
        ),
        key.variations()
            .records()
            .map(|axis| (axis.axis(), axis.value_milli() as u32))
            .collect(),
    )
}

pub(super) const fn raster_source_ordinal(source: UiGlyphRasterSource) -> u64 {
    match source {
        UiGlyphRasterSource::ColorOutline => 0,
        UiGlyphRasterSource::ColorBitmap => 1,
        UiGlyphRasterSource::AlphaOutline => 2,
        UiGlyphRasterSource::LastResort => 3,
    }
}

#[cfg(test)]
#[path = "sort_key_tests.rs"]
mod tests;
