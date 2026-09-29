//! Move-only atlas transaction records and inert cross-domain upload views.
use super::candidate_store::CandidateAtlasStore;
use super::ownership::AtlasCore;
use super::raster_upload::{upload_shape_is_valid, UiNativeTextAtlasUpload};
use super::recovery::UiNativeTextAtlasGeneration;
use super::transaction_plan_snapshot::UiNativeTextAtlasTransactionPlanSnapshot;
use super::UiNativeTextAtlasDemand;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use worth_ui_host_contract::{
    UiGlyphRasterDemandIdentity, UiGlyphRasterKey, UiQualifiedTextLayoutIdentity,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativeTextAtlasPinRequest {
    layout: UiQualifiedTextLayoutIdentity,
    key: UiGlyphRasterKey,
}

impl UiNativeTextAtlasPinRequest {
    #[doc(hidden)]
    pub const fn from_text_mechanics(
        layout: UiQualifiedTextLayoutIdentity,
        key: UiGlyphRasterKey,
    ) -> Self {
        Self { layout, key }
    }

    pub(crate) const fn layout(self) -> UiQualifiedTextLayoutIdentity {
        self.layout
    }

    pub(crate) const fn key(self) -> UiGlyphRasterKey {
        self.key
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiNativeTextAtlasPinTransition {
    additions: Box<[UiNativeTextAtlasPinRequest]>,
    releases: Box<[UiNativeTextAtlasPinRequest]>,
}

impl UiNativeTextAtlasPinTransition {
    #[doc(hidden)]
    pub fn from_text_mechanics(
        additions: impl IntoIterator<Item = UiNativeTextAtlasPinRequest>,
        releases: impl IntoIterator<Item = UiNativeTextAtlasPinRequest>,
    ) -> Self {
        Self {
            additions: additions.into_iter().collect(),
            releases: releases.into_iter().collect(),
        }
    }

    pub(crate) fn additions(&self) -> &[UiNativeTextAtlasPinRequest] {
        &self.additions
    }

    pub(crate) fn releases(&self) -> &[UiNativeTextAtlasPinRequest] {
        &self.releases
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeTextAtlasExternalOutcome {
    Submitted,
    Rejected,
    EffectsIndeterminate,
}

pub(crate) struct UiNativeTextAtlasTransactionPlan {
    pub(crate) core: Rc<RefCell<AtlasCore>>,
    pub(crate) reservation: u64,
    pub(crate) demand_identity: UiGlyphRasterDemandIdentity,
    pub(crate) peak_entries: u32,
    pub(crate) peak_texel_bytes: u64,
    pub(crate) predecessor_generation: UiNativeTextAtlasGeneration,
    pub(crate) candidate_generation: UiNativeTextAtlasGeneration,
    pub(crate) misses: Box<[UiNativeTextAtlasDemand]>,
    pub(crate) hits: Box<[UiGlyphRasterKey]>,
    pub(crate) evictions: Box<[UiGlyphRasterKey]>,
    pub(crate) candidate_alpha: CandidateAtlasStore,
    pub(crate) candidate_color: CandidateAtlasStore,
    pub(crate) pin_additions: Box<[UiNativeTextAtlasPinRequest]>,
    pub(crate) pin_releases: Box<[super::ownership::PinIdentity]>,
    pub(crate) pin_change_keys: Box<[UiGlyphRasterKey]>,
    pub(crate) next_entry: u64,
    pub(crate) staged_bytes: u64,
    pub(crate) physical_staged_bytes: u64,
    pub(crate) key_lookups: u32,
    pub(crate) page_probes: u32,
    pub(crate) placement_probes: u32,
    pub(crate) eviction_candidates: u32,
    pub(crate) committed: bool,
}

impl UiNativeTextAtlasTransactionPlan {
    pub(crate) const fn transaction_identity(&self) -> u64 {
        self.reservation
    }

    pub(crate) fn miss_demands(&self) -> &[UiNativeTextAtlasDemand] {
        &self.misses
    }

    pub(crate) fn placement_for(&self, key: UiGlyphRasterKey) -> Option<(u32, [u32; 2])> {
        let core = self.core.borrow();
        self.candidate_alpha
            .entry(&core.alpha, key)
            .or_else(|| self.candidate_color.entry(&core.color, key))
            .map(|entry| (entry.page, [entry.rect.x, entry.rect.y]))
    }

    pub(crate) fn candidate_page_counts(&self) -> (usize, usize) {
        (
            self.candidate_alpha.page_count(),
            self.candidate_color.page_count(),
        )
    }

    #[cfg(test)]
    pub(crate) fn hit_keys(&self) -> &[UiGlyphRasterKey] {
        &self.hits
    }

    #[cfg(test)]
    pub(crate) fn evicted_keys(&self) -> &[UiGlyphRasterKey] {
        &self.evictions
    }

    #[cfg(test)]
    pub(crate) fn candidate_overlay_entries(&self) -> usize {
        self.candidate_alpha.added_len() + self.candidate_color.added_len()
    }

    #[cfg(test)]
    pub(crate) const fn predecessor_generation(&self) -> UiNativeTextAtlasGeneration {
        self.predecessor_generation
    }

    #[cfg(test)]
    pub(crate) const fn candidate_generation(&self) -> UiNativeTextAtlasGeneration {
        self.candidate_generation
    }

    #[cfg(test)]
    pub(crate) const fn staged_bytes(&self) -> u64 {
        self.staged_bytes
    }

    pub(crate) const fn physical_staged_bytes(&self) -> u64 {
        self.physical_staged_bytes
    }

    pub(crate) fn snapshot(&self) -> UiNativeTextAtlasTransactionPlanSnapshot {
        UiNativeTextAtlasTransactionPlanSnapshot {
            demand_identity: self.demand_identity,
            peak_entries: self.peak_entries,
            peak_texel_bytes: self.peak_texel_bytes,
            predecessor_generation: self.predecessor_generation,
            candidate_generation: self.candidate_generation,
            misses: u32::try_from(self.misses.len()).unwrap_or(u32::MAX),
            hits: u32::try_from(self.hits.len()).unwrap_or(u32::MAX),
            evictions: u32::try_from(self.evictions.len()).unwrap_or(u32::MAX),
            staged_bytes: self.staged_bytes,
            physical_staged_bytes: self.physical_staged_bytes,
            key_lookups: self.key_lookups,
            page_probes: self.page_probes,
            placement_probes: self.placement_probes,
            eviction_candidates: self.eviction_candidates,
        }
    }

    pub(crate) fn admits_uploads(&self, uploads: &[UiNativeTextAtlasUpload]) -> bool {
        if uploads.len() != self.misses.len() {
            return false;
        }
        let miss_keys = self
            .misses
            .iter()
            .map(|demand| demand.key())
            .collect::<HashSet<_>>();
        worth_ui_host_contract::record_presentation_map_inserts(self.misses.len());
        if miss_keys.len() != self.misses.len() {
            return false;
        }
        worth_ui_host_contract::record_presentation_map_inserts(uploads.len());
        let mut upload_keys = HashSet::with_capacity(uploads.len());
        uploads.iter().all(|upload| {
            upload_keys.insert(upload.key())
                && miss_keys.contains(&upload.key())
                && self.admits_upload(upload)
                && upload_shape_is_valid(upload)
        }) && upload_keys == miss_keys
    }

    fn admits_upload(&self, upload: &UiNativeTextAtlasUpload) -> bool {
        self.misses.iter().any(|demand| {
            demand.key() == upload.key()
                && upload.width() <= demand.width()
                && upload.height() <= demand.height()
                && u64::try_from(upload.bytes().len()).unwrap_or(u64::MAX) <= demand.staged_bytes()
                && upload.key().source() == demand.key().source()
        })
    }
}

impl Drop for UiNativeTextAtlasTransactionPlan {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        if let Ok(mut core) = self.core.try_borrow_mut() {
            if core.reservation == Some(self.reservation) {
                core.reservation = None;
            }
        }
    }
}
