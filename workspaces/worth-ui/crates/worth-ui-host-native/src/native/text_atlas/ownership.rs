//! Host-owned bounded alpha/color atlas state.

#[cfg(test)]
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::Digest;

use worth_ui_host_contract::UiCountedSha256;

use worth_ui_host_contract::{
    UiGlyphRasterKey, UiGlyphRasterKeyEvidence, UiQualifiedTextLayoutIdentity,
};

use super::entry::UiAtlasEntry;
use super::key::canonical_raster_key_bytes;
use super::pinning::UiNativeTextAtlasPin;
use super::placement::UiAtlasPage;
#[cfg(test)]
use super::placement::UiAtlasRect;
use super::recovery::{UiNativeTextAtlasGeneration, UiNativeTextAtlasLineageIdentity};
use super::{alpha::AlphaAtlasStore, color::ColorAtlasStore};

static NEXT_ATLAS_LINEAGE: AtomicU64 = AtomicU64::new(1);

fn next_atlas_lineage() -> UiNativeTextAtlasLineageIdentity {
    let value = NEXT_ATLAS_LINEAGE.fetch_add(1, Ordering::Relaxed);
    UiNativeTextAtlasLineageIdentity::from_native_host(value)
        .unwrap_or_else(|| panic!("atlas lineage identity exhausted"))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AtlasStore {
    pub(crate) page_width: u32,
    pub(crate) page_height: u32,
    pub(crate) page_limit: u32,
    pub(crate) channels: u32,
    pub(crate) pages: Vec<UiAtlasPage>,
    pub(crate) entries: HashMap<UiGlyphRasterKey, UiAtlasEntry>,
}

impl AtlasStore {
    pub(crate) fn new(page_width: u32, page_height: u32, page_limit: u32, channels: u32) -> Self {
        Self {
            page_width,
            page_height,
            page_limit,
            channels,
            pages: Vec::new(),
            entries: HashMap::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn allocate(&mut self, width: u32, height: u32) -> Option<(u32, UiAtlasRect)> {
        for (page, slot) in self.pages.iter_mut().enumerate() {
            if let Some(rect) = slot.allocate(width, height) {
                return Some((u32::try_from(page).ok()?, rect));
            }
        }
        if u32::try_from(self.pages.len()).ok()? >= self.page_limit {
            return None;
        }
        self.pages
            .push(UiAtlasPage::new(self.page_width, self.page_height));
        let page = u32::try_from(self.pages.len() - 1).ok()?;
        self.pages
            .last_mut()?
            .allocate(width, height)
            .map(|rect| (page, rect))
    }

    #[cfg(test)]
    pub(crate) fn remove(&mut self, key: UiGlyphRasterKey) -> Option<UiAtlasEntry> {
        let entry = self.entries.remove(&key)?;
        let page = self.pages.get_mut(usize::try_from(entry.page).ok()?)?;
        page.release(entry.rect);
        Some(entry)
    }

    #[cfg(test)]
    pub(crate) fn insert(&mut self, entry: UiAtlasEntry) {
        self.entries.insert(entry.key, entry);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct PinIdentity {
    layout: [u8; 32],
    key: UiGlyphRasterKeyEvidence,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UiNativeTextPinObservation {
    layout: [u8; 32],
    raster_key: [u8; 32],
    entry: super::key::UiAtlasEntryIdentity,
    generation: UiNativeTextAtlasGeneration,
}

impl PinIdentity {
    pub(crate) fn new(layout: UiQualifiedTextLayoutIdentity, key: UiGlyphRasterKey) -> Self {
        Self {
            layout: layout.digest(),
            key: canonical_raster_key_bytes(key),
        }
    }

    fn raster_key_digest(self) -> [u8; 32] {
        UiCountedSha256::digest(self.key).into()
    }
}

impl UiNativeTextPinObservation {
    #[doc(hidden)]
    pub const fn layout_digest(self) -> [u8; 32] {
        self.layout
    }

    #[doc(hidden)]
    pub const fn raster_key_digest(self) -> [u8; 32] {
        self.raster_key
    }

    #[doc(hidden)]
    pub const fn entry_identity(self) -> super::key::UiAtlasEntryIdentity {
        self.entry
    }

    #[doc(hidden)]
    pub const fn generation(self) -> UiNativeTextAtlasGeneration {
        self.generation
    }

    #[doc(hidden)]
    pub fn matches(self, request: worth_ui_host_contract::UiGlyphRasterPinRequest) -> bool {
        let identity = PinIdentity::new(request.layout_identity(), request.key());
        self.layout == identity.layout && self.raster_key == identity.raster_key_digest()
    }
}

pub(crate) struct AtlasCore {
    pub(crate) alpha: AlphaAtlasStore,
    pub(crate) color: ColorAtlasStore,
    pub(crate) generation: UiNativeTextAtlasGeneration,
    pub(crate) reservation: Option<u64>,
    pub(crate) next_reservation: u64,
    pub(crate) next_entry: u64,
    pub(crate) completed_use_epoch: u64,
    pub(crate) committed_transactions: u64,
    pub(crate) pins: BTreeMap<PinIdentity, UiNativeTextAtlasPin>,
    pub(crate) quarantined: bool,
    pub(crate) lineage: UiNativeTextAtlasLineageIdentity,
}

impl AtlasCore {
    pub(crate) fn new(lineage: UiNativeTextAtlasLineageIdentity) -> Self {
        Self {
            alpha: AlphaAtlasStore::new(),
            color: ColorAtlasStore::new(),
            generation: UiNativeTextAtlasGeneration::new(1).expect("nonzero atlas generation"),
            reservation: None,
            next_reservation: 1,
            next_entry: 1,
            completed_use_epoch: 0,
            committed_transactions: 0,
            pins: BTreeMap::new(),
            quarantined: false,
            lineage,
        }
    }

    pub(crate) fn reset_for_reconstruction(&mut self, generation: UiNativeTextAtlasGeneration) {
        self.alpha = AlphaAtlasStore::new();
        self.color = ColorAtlasStore::new();
        self.generation = generation;
        self.reservation = None;
        self.pins.clear();
        self.quarantined = false;
    }
}

pub(crate) struct UiNativeTextAtlas {
    pub(crate) core: Rc<RefCell<AtlasCore>>,
    #[cfg(test)]
    pub(crate) plan_calls: Cell<usize>,
}

#[derive(Clone, Copy)]
pub(crate) struct UiNativeTextAtlasEntryView {
    pub(crate) kind: super::upload::UiNativeGpuAtlasKind,
    pub(crate) page: u32,
    pub(crate) origin: [u32; 2],
    pub(crate) extent: [u32; 2],
    pub(crate) page_extent: [u32; 2],
    pub(crate) bearing: worth_ui_host_contract::UiGlyphRasterBearing,
}

impl Default for UiNativeTextAtlas {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNativeTextAtlas {
    pub(crate) fn new() -> Self {
        let lineage = next_atlas_lineage();
        Self {
            core: Rc::new(RefCell::new(AtlasCore::new(lineage))),
            #[cfg(test)]
            plan_calls: Cell::new(0),
        }
    }

    /// Whether the atlas holds `request`'s pin, by its identity alone.
    pub(crate) fn is_pinned(
        &self,
        request: worth_ui_host_contract::UiGlyphRasterPinRequest,
    ) -> bool {
        self.core
            .borrow()
            .pins
            .contains_key(&PinIdentity::new(request.layout_identity(), request.key()))
    }

    pub(crate) fn pin_count(&self) -> usize {
        self.core.borrow().pins.len()
    }

    pub(crate) fn pin_observations(&self) -> Box<[UiNativeTextPinObservation]> {
        self.core
            .borrow()
            .pins
            .values()
            .map(|pin| {
                let identity = pin.identity();
                UiNativeTextPinObservation {
                    layout: pin.layout().digest(),
                    raster_key: identity.raster_key_digest(),
                    entry: pin.entry(),
                    generation: pin.generation(),
                }
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    pub(crate) fn committed_transactions(&self) -> u64 {
        self.core.borrow().committed_transactions
    }

    pub(crate) fn semantic_model_digest(&self) -> [u8; 32] {
        let core = self.core.borrow();
        let mut rows = Vec::with_capacity(core.alpha.entries.len() + core.color.entries.len());
        rows.extend(
            core.alpha
                .entries
                .values()
                .map(|entry| model_entry_bytes(0, entry)),
        );
        rows.extend(
            core.color
                .entries
                .values()
                .map(|entry| model_entry_bytes(1, entry)),
        );
        rows.sort_unstable();
        let mut digest = UiCountedSha256::new();
        digest.update(core.alpha.page_width.to_le_bytes());
        digest.update(core.alpha.page_height.to_le_bytes());
        digest.update((core.alpha.pages.len() as u64).to_le_bytes());
        digest.update(core.color.page_width.to_le_bytes());
        digest.update(core.color.page_height.to_le_bytes());
        digest.update((core.color.pages.len() as u64).to_le_bytes());
        digest.update((rows.len() as u64).to_le_bytes());
        for row in rows {
            digest.update((row.len() as u64).to_le_bytes());
            digest.update(row);
        }
        digest.finalize().into()
    }

    pub(crate) fn entry_view(&self, key: UiGlyphRasterKey) -> Option<UiNativeTextAtlasEntryView> {
        let core = self.core.borrow();
        let (entry, kind, page_extent) = match key.source() {
            worth_ui_host_contract::UiGlyphRasterSource::AlphaOutline
            | worth_ui_host_contract::UiGlyphRasterSource::LastResort => (
                core.alpha.entries.get(&key)?,
                super::upload::UiNativeGpuAtlasKind::Alpha,
                [core.alpha.page_width, core.alpha.page_height],
            ),
            worth_ui_host_contract::UiGlyphRasterSource::ColorOutline
            | worth_ui_host_contract::UiGlyphRasterSource::ColorBitmap => (
                core.color.entries.get(&key)?,
                super::upload::UiNativeGpuAtlasKind::Color,
                [core.color.page_width, core.color.page_height],
            ),
        };
        Some(UiNativeTextAtlasEntryView {
            kind,
            page: entry.page,
            origin: [entry.rect.x, entry.rect.y],
            extent: entry.content_extent,
            page_extent,
            bearing: entry.bearing,
        })
    }
}

fn model_entry_bytes(kind: u8, entry: &UiAtlasEntry) -> Vec<u8> {
    let mut row = Vec::with_capacity(160);
    row.push(kind);
    row.extend_from_slice(&canonical_raster_key_bytes(entry.key));
    row.extend_from_slice(&entry.page.to_le_bytes());
    row.extend_from_slice(&entry.rect.x.to_le_bytes());
    row.extend_from_slice(&entry.rect.y.to_le_bytes());
    row.extend_from_slice(&entry.rect.width.to_le_bytes());
    row.extend_from_slice(&entry.rect.height.to_le_bytes());
    row.extend_from_slice(&entry.content_extent[0].to_le_bytes());
    row.extend_from_slice(&entry.content_extent[1].to_le_bytes());
    row.extend_from_slice(&entry.bearing.x_over_64().to_le_bytes());
    row.extend_from_slice(&entry.bearing.y_over_64().to_le_bytes());
    row.extend_from_slice(&entry.digest);
    row.extend_from_slice(&entry.pin_count.to_le_bytes());
    row
}
