use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use worth_ui_host_contract::{
    UiGlyphRasterBearing, UiGlyphRasterContentDigest, UiGlyphRasterDemandRecord,
    UiGlyphRasterExtent, UiGlyphRasterKey,
};

use super::batch::{UiGlyphRasterRecord, UiGlyphRasterRecordInput};
use super::{UiAlphaRasterKind, UiColorRasterKind, UiGlyphRasterAdmissionDenial};

const QUALIFIED_CACHE_ENTRY_LIMIT: usize = 4_096;
const QUALIFIED_CACHE_BYTE_LIMIT: usize = 64 * 1024 * 1024;

#[derive(Clone)]
struct UiCachedGlyphRaster {
    bearing: UiGlyphRasterBearing,
    extent: UiGlyphRasterExtent,
    stride: u32,
    pixels: Arc<[u8]>,
    digest: UiGlyphRasterContentDigest,
}

/// Text-owned bounded cache of renderer output. It carries no atlas placement,
/// pin, upload, or presentation authority.
#[derive(Default)]
pub struct UiGlyphRasterCache {
    entries: HashMap<UiGlyphRasterKey, UiCachedGlyphRaster>,
    insertion_order: VecDeque<UiGlyphRasterKey>,
    retained_bytes: usize,
}

impl UiGlyphRasterCache {
    pub fn clear(&mut self) -> usize {
        let removed = self.entries.len();
        self.entries.clear();
        self.insertion_order.clear();
        self.retained_bytes = 0;
        removed
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Report whether the exact text-owned raster key is retained.  Pin and
    /// atlas ownership stay outside this cache; this check only lets a
    /// presentation successor prove that its reuse receipt cannot select a
    /// raster miss.
    pub fn contains_key(&self, key: UiGlyphRasterKey) -> bool {
        self.entries.contains_key(&key)
    }

    pub(crate) fn alpha_record(
        &self,
        demand: UiGlyphRasterDemandRecord,
    ) -> Option<Result<UiGlyphRasterRecord<UiAlphaRasterKind>, UiGlyphRasterAdmissionDenial>> {
        self.record(demand)
            .map(UiGlyphRasterRecord::<UiAlphaRasterKind>::from_text_mechanics)
    }

    pub(crate) fn color_record(
        &self,
        demand: UiGlyphRasterDemandRecord,
    ) -> Option<Result<UiGlyphRasterRecord<UiColorRasterKind>, UiGlyphRasterAdmissionDenial>> {
        self.record(demand)
            .map(UiGlyphRasterRecord::<UiColorRasterKind>::from_text_mechanics)
    }

    fn record(&self, demand: UiGlyphRasterDemandRecord) -> Option<UiGlyphRasterRecordInput> {
        let cached = self.entries.get(&demand.key())?;
        Some(UiGlyphRasterRecordInput {
            key: demand.key(),
            attribution: demand.attribution(),
            bearing: cached.bearing,
            extent: cached.extent,
            stride: cached.stride,
            pixels: Arc::clone(&cached.pixels),
            digest: cached.digest,
        })
    }

    pub(crate) fn insert<Kind: super::UiGlyphRasterFormat>(
        &mut self,
        record: &UiGlyphRasterRecord<Kind>,
    ) {
        let key = record.key();
        if self.entries.contains_key(&key) {
            return;
        }
        let pixels = record.pixels_arc();
        let bytes = pixels.len();
        if bytes > QUALIFIED_CACHE_BYTE_LIMIT {
            return;
        }
        while self.entries.len() == QUALIFIED_CACHE_ENTRY_LIMIT
            || self.retained_bytes.saturating_add(bytes) > QUALIFIED_CACHE_BYTE_LIMIT
        {
            let Some(evicted) = self.insertion_order.pop_front() else {
                break;
            };
            if let Some(entry) = self.entries.remove(&evicted) {
                self.retained_bytes = self.retained_bytes.saturating_sub(entry.pixels.len());
            }
        }
        self.retained_bytes = self.retained_bytes.saturating_add(bytes);
        self.insertion_order.push_back(key);
        self.entries.insert(
            key,
            UiCachedGlyphRaster {
                bearing: record.bearing(),
                extent: record.extent(),
                stride: record.stride(),
                pixels,
                digest: record.digest(),
            },
        );
    }
}
