//! Effect-free native atlas demand admission and reservation planning.

use std::collections::HashSet;
use std::rc::Rc;

use worth_ui_host_contract::{
    UiGlyphRasterDemandBatchView, UiGlyphRasterDemandIdentity, UiGlyphRasterKey,
    UiGlyphRasterSource,
};

use super::candidate_store::CandidateAtlasStore;
use super::capacity::{MAX_ATLAS_TEXEL_BYTES, MAX_ENTRIES, MAX_STAGED_BYTES};
use super::demand_admission::{translate_demands, validate_inputs};
use super::entry::UiAtlasEntry;
use super::key::UiAtlasEntryIdentity;
use super::ownership::{AtlasCore, UiNativeTextAtlas};
use super::pinning::protected_keys;
use super::placement::UiAtlasRect;
use super::recovery::{UiNativeTextAtlasDenial, UiNativeTextAtlasGeneration};
use super::transaction::{UiNativeTextAtlasPinTransition, UiNativeTextAtlasTransactionPlan};
use super::UiNativeTextAtlasDemand;

mod eviction;

struct Candidate {
    alpha: CandidateAtlasStore,
    color: CandidateAtlasStore,
    misses: Vec<UiNativeTextAtlasDemand>,
    hits: Vec<UiGlyphRasterKey>,
    evictions: Vec<UiGlyphRasterKey>,
    pin_additions: Vec<super::transaction::UiNativeTextAtlasPinRequest>,
    pin_releases: Vec<super::ownership::PinIdentity>,
    pin_change_keys: HashSet<UiGlyphRasterKey>,
    next_entry: u64,
    peak_entries: usize,
    peak_texel_bytes: u64,
    demand_identity: UiGlyphRasterDemandIdentity,
    staged_bytes: u64,
    physical_staged_bytes: u64,
    key_lookups: u32,
    page_probes: u32,
    placement_probes: u32,
    eviction_candidates: u32,
}

impl UiNativeTextAtlas {
    pub(crate) fn plan_many(
        &self,
        demand_views: &[UiGlyphRasterDemandBatchView<'_>],
        transition: &UiNativeTextAtlasPinTransition,
    ) -> Result<UiNativeTextAtlasTransactionPlan, UiNativeTextAtlasDenial> {
        #[cfg(test)]
        self.plan_calls.set(self.plan_calls.get().saturating_add(1));
        let (demands, _) = translate_demands(demand_views)?;
        self.plan_demands(&demands, transition)
    }

    pub(crate) fn plan_demands(
        &self,
        demands: &[UiNativeTextAtlasDemand],
        transition: &UiNativeTextAtlasPinTransition,
    ) -> Result<UiNativeTextAtlasTransactionPlan, UiNativeTextAtlasDenial> {
        let mut core = self.core.borrow_mut();
        if core.quarantined {
            return Err(UiNativeTextAtlasDenial::ReconstructionRequired);
        }
        let (normalized, demand_identity) = validate_inputs(&core, demands, transition)?;
        let mut candidate = build_candidate(&core, &normalized, transition)?;
        candidate.demand_identity = demand_identity;
        reserve_plan(&mut core, Rc::clone(&self.core), candidate)
    }

    #[cfg(test)]
    pub(crate) fn reset_plan_calls(&self) {
        self.plan_calls.set(0);
    }

    #[cfg(test)]
    pub(crate) fn plan_calls(&self) -> usize {
        self.plan_calls.get()
    }
}

fn build_candidate(
    core: &AtlasCore,
    demands: &[UiNativeTextAtlasDemand],
    transition: &UiNativeTextAtlasPinTransition,
) -> Result<Candidate, UiNativeTextAtlasDenial> {
    worth_ui_host_contract::record_presentation_glyphs(
        worth_ui_host_contract::UiPresentationWorkStage::AtlasPlan,
        demands.len(),
    );
    let protected = protected_keys(core, transition, demands);
    let mut candidate = candidate_from_predecessor(core);
    for demand in demands.iter().copied() {
        admit_demand(
            &mut candidate,
            core,
            demand,
            &protected,
            core.completed_use_epoch,
        )?;
    }
    finalize_candidate_capacity(core, transition, &mut candidate)?;
    Ok(candidate)
}

fn candidate_from_predecessor(core: &AtlasCore) -> Candidate {
    Candidate {
        alpha: CandidateAtlasStore::from_predecessor(&core.alpha),
        color: CandidateAtlasStore::from_predecessor(&core.color),
        misses: Vec::new(),
        hits: Vec::new(),
        evictions: Vec::new(),
        pin_additions: Vec::new(),
        pin_releases: Vec::new(),
        pin_change_keys: HashSet::new(),
        next_entry: core.next_entry,
        peak_entries: 0,
        peak_texel_bytes: 0,
        demand_identity: UiGlyphRasterDemandIdentity::from_text_mechanics([0; 32]),
        staged_bytes: 0,
        physical_staged_bytes: 0,
        key_lookups: 0,
        page_probes: 0,
        placement_probes: 0,
        eviction_candidates: 0,
    }
}

fn finalize_candidate_capacity(
    core: &AtlasCore,
    transition: &UiNativeTextAtlasPinTransition,
    candidate: &mut Candidate,
) -> Result<(), UiNativeTextAtlasDenial> {
    candidate.staged_bytes = candidate
        .misses
        .iter()
        .try_fold(0_u64, |total, demand| {
            total.checked_add(demand.staged_bytes())
        })
        .ok_or(UiNativeTextAtlasDenial::StagingCapacityExceeded)?;
    candidate.physical_staged_bytes = candidate
        .misses
        .iter()
        .try_fold(0_u64, |total, demand| {
            total.checked_add(demand.physical_staged_bytes())
        })
        .ok_or(UiNativeTextAtlasDenial::StagingCapacityExceeded)?;
    if candidate.staged_bytes > MAX_STAGED_BYTES
        || candidate.physical_staged_bytes > MAX_STAGED_BYTES
    {
        return Err(UiNativeTextAtlasDenial::StagingCapacityExceeded);
    }
    let (pin_additions, pin_releases) = pin_changes_for_overlay(core, transition, candidate)?;
    candidate.pin_additions = pin_additions;
    candidate.pin_releases = pin_releases;
    candidate.pin_change_keys = transition
        .additions()
        .iter()
        .chain(transition.releases())
        .map(|pin| pin.key())
        .collect();
    candidate.peak_texel_bytes = candidate
        .alpha
        .texel_bytes(&core.alpha)
        .checked_add(candidate.color.texel_bytes(&core.color))
        .ok_or(UiNativeTextAtlasDenial::TexelCapacityExceeded)?;
    if candidate.peak_texel_bytes > MAX_ATLAS_TEXEL_BYTES {
        return Err(UiNativeTextAtlasDenial::TexelCapacityExceeded);
    }
    candidate.peak_entries = (core.alpha.entries.len() + core.color.entries.len())
        .max(candidate.alpha.len(&core.alpha) + candidate.color.len(&core.color));
    if candidate.peak_entries > MAX_ENTRIES {
        return Err(UiNativeTextAtlasDenial::EntryCapacityExceeded);
    }
    Ok(())
}

fn admit_demand(
    candidate: &mut Candidate,
    core: &AtlasCore,
    demand: UiNativeTextAtlasDemand,
    protected: &HashSet<UiGlyphRasterKey>,
    completed_use_epoch: u64,
) -> Result<(), UiNativeTextAtlasDenial> {
    let key = demand.key();
    candidate.key_lookups = candidate.key_lookups.saturating_add(1);
    if candidate.alpha.contains(&core.alpha, key) || candidate.color.contains(&core.color, key) {
        candidate.hits.push(key);
        return Ok(());
    }
    if candidate.alpha.len(&core.alpha) + candidate.color.len(&core.color) >= MAX_ENTRIES {
        let evicted = eviction::evict_candidate_one(candidate, core, protected)
            .ok_or(UiNativeTextAtlasDenial::PinnedCapacityExceeded)?;
        candidate.evictions.push(evicted);
    }
    let (page, rect) = allocate_slot(candidate, core, demand, protected)?;
    insert_entry(candidate, demand, page, rect, completed_use_epoch)?;
    candidate.misses.push(demand);
    Ok(())
}

fn allocate_slot(
    candidate: &mut Candidate,
    core: &AtlasCore,
    demand: UiNativeTextAtlasDemand,
    protected: &HashSet<UiGlyphRasterKey>,
) -> Result<(u32, UiAtlasRect), UiNativeTextAtlasDenial> {
    match demand.key().source() {
        UiGlyphRasterSource::ColorOutline | UiGlyphRasterSource::ColorBitmap => {
            eviction::allocate_candidate_with_eviction(
                &mut candidate.color,
                &core.color,
                demand,
                protected,
                &mut candidate.evictions,
                &mut candidate.page_probes,
                &mut candidate.placement_probes,
                &mut candidate.eviction_candidates,
            )
        }
        UiGlyphRasterSource::AlphaOutline | UiGlyphRasterSource::LastResort => {
            eviction::allocate_candidate_with_eviction(
                &mut candidate.alpha,
                &core.alpha,
                demand,
                protected,
                &mut candidate.evictions,
                &mut candidate.page_probes,
                &mut candidate.placement_probes,
                &mut candidate.eviction_candidates,
            )
        }
    }
}

fn insert_entry(
    candidate: &mut Candidate,
    demand: UiNativeTextAtlasDemand,
    page: u32,
    rect: UiAtlasRect,
    completed_use_epoch: u64,
) -> Result<(), UiNativeTextAtlasDenial> {
    let identity = UiAtlasEntryIdentity::from_native_host(candidate.next_entry)
        .ok_or(UiNativeTextAtlasDenial::EntryCapacityExceeded)?;
    candidate.next_entry = candidate
        .next_entry
        .checked_add(1)
        .ok_or(UiNativeTextAtlasDenial::EntryCapacityExceeded)?;
    let entry = UiAtlasEntry {
        identity,
        key: demand.key(),
        page,
        rect,
        content_extent: [demand.width(), demand.height()],
        bearing: worth_ui_host_contract::UiGlyphRasterBearing::from_sixty_fourths(0, 0),
        staged_bytes: demand.staged_bytes(),
        digest: [0; 32],
        pin_count: 0,
        completed_use_epoch,
    };
    match demand.key().source() {
        UiGlyphRasterSource::ColorOutline | UiGlyphRasterSource::ColorBitmap => {
            candidate.color.insert(entry)
        }
        UiGlyphRasterSource::AlphaOutline | UiGlyphRasterSource::LastResort => {
            candidate.alpha.insert(entry)
        }
    }
    Ok(())
}

fn pin_changes_for_overlay(
    core: &AtlasCore,
    transition: &UiNativeTextAtlasPinTransition,
    candidate: &Candidate,
) -> Result<
    (
        Vec<super::transaction::UiNativeTextAtlasPinRequest>,
        Vec<super::ownership::PinIdentity>,
    ),
    UiNativeTextAtlasDenial,
> {
    let releases = transition
        .releases()
        .iter()
        .map(|release| super::ownership::PinIdentity::new(release.layout(), release.key()))
        .collect();
    let mut additions = Vec::with_capacity(transition.additions().len());
    for add in transition.additions() {
        if !(candidate.alpha.contains(&core.alpha, add.key())
            || candidate.color.contains(&core.color, add.key()))
        {
            return Err(UiNativeTextAtlasDenial::StalePin);
        }
        additions.push(*add);
    }
    Ok((additions, releases))
}

fn reserve_plan(
    core: &mut AtlasCore,
    core_handle: Rc<std::cell::RefCell<AtlasCore>>,
    candidate: Candidate,
) -> Result<UiNativeTextAtlasTransactionPlan, UiNativeTextAtlasDenial> {
    let predecessor_generation = core.generation;
    let candidate_generation = UiNativeTextAtlasGeneration::new(
        predecessor_generation
            .get()
            .checked_add(1)
            .ok_or(UiNativeTextAtlasDenial::GenerationExhausted)?,
    )
    .ok_or(UiNativeTextAtlasDenial::GenerationExhausted)?;
    let reservation = core.next_reservation;
    let next_reservation = core
        .next_reservation
        .checked_add(1)
        .ok_or(UiNativeTextAtlasDenial::ReservationConflict)?;
    core.next_reservation = next_reservation;
    core.reservation = Some(reservation);
    Ok(UiNativeTextAtlasTransactionPlan {
        core: core_handle,
        reservation,
        demand_identity: candidate.demand_identity,
        peak_entries: u32::try_from(candidate.peak_entries).unwrap_or(u32::MAX),
        peak_texel_bytes: candidate.peak_texel_bytes,
        predecessor_generation,
        candidate_generation,
        misses: candidate.misses.into_boxed_slice(),
        hits: candidate.hits.into_boxed_slice(),
        evictions: candidate.evictions.into_boxed_slice(),
        candidate_alpha: candidate.alpha,
        candidate_color: candidate.color,
        pin_additions: candidate.pin_additions.into_boxed_slice(),
        pin_releases: candidate.pin_releases.into_boxed_slice(),
        pin_change_keys: candidate
            .pin_change_keys
            .into_iter()
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        next_entry: candidate.next_entry,
        staged_bytes: candidate.staged_bytes,
        physical_staged_bytes: candidate.physical_staged_bytes,
        key_lookups: candidate.key_lookups,
        page_probes: candidate.page_probes,
        placement_probes: candidate.placement_probes,
        eviction_candidates: candidate.eviction_candidates,
        committed: false,
    })
}

fn bounded_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
