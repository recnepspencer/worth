//! Transaction-local atlas overlay without cloning retained entry maps.

use std::collections::{HashMap, HashSet};

use worth_ui_host_contract::{UiGlyphRasterKey, UiGlyphRasterKeyEvidence};

use super::entry::UiAtlasEntry;
use super::key::canonical_raster_key_bytes;
use super::ownership::AtlasStore;
use super::placement::{UiAtlasPage, UiAtlasRect};

#[derive(Default)]
pub(crate) struct CandidateAtlasStore {
    pages: Vec<UiAtlasPage>,
    added: HashMap<UiGlyphRasterKey, UiAtlasEntry>,
    removed: HashSet<UiGlyphRasterKey>,
}

pub(crate) struct CandidateAtlasAllocation {
    pub(crate) placement: Option<(u32, UiAtlasRect)>,
    pub(crate) page_probes: usize,
    pub(crate) placement_probes: usize,
}

impl CandidateAtlasStore {
    pub(crate) fn from_predecessor(predecessor: &AtlasStore) -> Self {
        Self {
            pages: predecessor.pages.clone(),
            added: HashMap::new(),
            removed: HashSet::new(),
        }
    }

    pub(crate) fn contains(&self, predecessor: &AtlasStore, key: UiGlyphRasterKey) -> bool {
        self.added.contains_key(&key)
            || (!self.removed.contains(&key) && predecessor.entries.contains_key(&key))
    }

    pub(crate) fn entry<'entry>(
        &'entry self,
        predecessor: &'entry AtlasStore,
        key: UiGlyphRasterKey,
    ) -> Option<&'entry UiAtlasEntry> {
        self.added.get(&key).or_else(|| {
            (!self.removed.contains(&key))
                .then(|| predecessor.entries.get(&key))
                .flatten()
        })
    }

    pub(crate) fn added_entry_mut(&mut self, key: UiGlyphRasterKey) -> Option<&mut UiAtlasEntry> {
        self.added.get_mut(&key)
    }

    pub(crate) fn len(&self, predecessor: &AtlasStore) -> usize {
        predecessor
            .entries
            .len()
            .saturating_sub(self.removed.len())
            .saturating_add(self.added.len())
    }

    pub(crate) fn page_count(&self) -> usize {
        self.pages.len()
    }

    #[cfg(test)]
    pub(crate) fn added_len(&self) -> usize {
        self.added.len()
    }

    pub(crate) fn texel_bytes(&self, predecessor: &AtlasStore) -> u64 {
        u64::try_from(self.pages.len())
            .unwrap_or(u64::MAX)
            .saturating_mul(u64::from(predecessor.page_width))
            .saturating_mul(u64::from(predecessor.page_height))
            .saturating_mul(u64::from(predecessor.channels))
    }

    pub(crate) fn allocate_observed(
        &mut self,
        predecessor: &AtlasStore,
        width: u32,
        height: u32,
    ) -> CandidateAtlasAllocation {
        let mut page_probes = 0;
        let mut placement_probes = 0;
        for (page, slot) in self.pages.iter_mut().enumerate() {
            page_probes += 1;
            let (placement, probes) = slot.allocate_observed(width, height);
            placement_probes += probes;
            if let Some(rect) = placement {
                return CandidateAtlasAllocation {
                    placement: u32::try_from(page).ok().map(|page| (page, rect)),
                    page_probes,
                    placement_probes,
                };
            }
        }
        if u32::try_from(self.pages.len())
            .ok()
            .is_none_or(|pages| pages >= predecessor.page_limit)
        {
            return CandidateAtlasAllocation {
                placement: None,
                page_probes,
                placement_probes,
            };
        }
        self.pages.push(UiAtlasPage::new(
            predecessor.page_width,
            predecessor.page_height,
        ));
        page_probes += 1;
        let page = u32::try_from(self.pages.len() - 1).ok();
        let (placement, probes) = self
            .pages
            .last_mut()
            .expect("new atlas page remains retained")
            .allocate_observed(width, height);
        placement_probes += probes;
        CandidateAtlasAllocation {
            placement: page.zip(placement),
            page_probes,
            placement_probes,
        }
    }

    pub(crate) fn insert(&mut self, entry: UiAtlasEntry) {
        self.added.insert(entry.key, entry);
    }

    pub(crate) fn remove_existing(
        &mut self,
        predecessor: &AtlasStore,
        key: UiGlyphRasterKey,
    ) -> Option<UiAtlasEntry> {
        if self.removed.contains(&key) {
            return None;
        }
        let entry = predecessor.entries.get(&key)?.clone();
        self.pages
            .get_mut(usize::try_from(entry.page).ok()?)?
            .release(entry.rect);
        self.removed.insert(key);
        Some(entry)
    }

    pub(crate) fn oldest_unprotected_observed(
        &self,
        predecessor: &AtlasStore,
        protected: &HashSet<UiGlyphRasterKey>,
    ) -> (
        Option<(u64, UiGlyphRasterKeyEvidence, UiGlyphRasterKey)>,
        usize,
    ) {
        let mut candidates = 0;
        let oldest = predecessor
            .entries
            .values()
            .filter(|entry| !self.removed.contains(&entry.key))
            .filter(|entry| !protected.contains(&entry.key))
            .inspect(|_| candidates += 1)
            .map(|entry| {
                (
                    entry.completed_use_epoch,
                    canonical_raster_key_bytes(entry.key),
                    entry.key,
                )
            })
            .min_by(|left, right| (left.0, &left.1).cmp(&(right.0, &right.1)));
        (oldest, candidates)
    }

    pub(crate) fn apply(self, store: &mut AtlasStore) {
        store.pages = self.pages;
        for key in self.removed {
            store.entries.remove(&key);
        }
        store.entries.extend(self.added);
    }
}
