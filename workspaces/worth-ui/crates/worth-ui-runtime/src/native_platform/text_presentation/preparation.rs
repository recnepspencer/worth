//! Start the native text-presentation transaction from mounted authority.

use std::num::NonZeroU32;

use sha2::{Digest, Sha256};
use worth_ui_host_contract::{
    UiGlyphRunView, UiMountedLogicalDamage, UiMountedPaintCommandIdentity,
    UiMountedPresentationWorkView, UiMountedSurfaceBindingRequirement,
};

#[path = "preparation/complete.rs"]
mod complete;
pub(crate) use complete::prepare_complete_semantic_text;

#[path = "preparation/demand_join.rs"]
mod demand_join;
#[path = "preparation/mounted_work.rs"]
mod mounted_work;

#[cfg(test)]
use super::rasterization::UiNativeTextRasterWorkReport;
use demand_join::{prepare_demands, rebuild_glyph_runs, MountedTextDemandJoin, PreparedDemand};
pub(crate) use mounted_work::mounted_semantic_text;
use mounted_work::{logical_damage, MountedSemanticTextCommand, MountedSemanticTextWork};
use worth_ui_text::{UiGlyphRasterDemandBatch, UiGlyphRasterDemandDenial, UiGlyphRasterLane};

use crate::mounting::{
    UiMountedTextForegroundPresentationBasis, UiMountedTextForegroundReuseReceipt,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeTextPresentationReadiness {
    LayoutMismatch,
    SourceMismatch,
    DemandDenied(UiGlyphRasterDemandDenial),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativeTextDemandInspection {
    demand_batches: u32,
    demand_records: u32,
    key_checks: u32,
}

impl UiNativeTextDemandInspection {
    pub(crate) const fn demand_batches(self) -> u32 {
        self.demand_batches
    }

    pub(crate) const fn demand_records(self) -> u32 {
        self.demand_records
    }

    pub(crate) const fn key_checks(self) -> u32 {
        self.key_checks
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedEventTimeDpiAuthority(NonZeroU32);

impl UiMountedEventTimeDpiAuthority {
    pub(crate) fn from_requirement(
        requirement: UiMountedSurfaceBindingRequirement,
    ) -> Option<Self> {
        NonZeroU32::new(requirement.device_scale_milli()).map(Self)
    }

    pub(crate) const fn dpi_milli(self) -> u32 {
        self.0.get()
    }
}

pub(crate) enum UiNativeTextPresentationPreparation {
    Prepared(UiNativeTextPresentationPrepared),
    Denied(UiNativeTextPresentationDenial),
}

pub(crate) struct UiNativeTextPresentationPrepared {
    layout_count: usize,
    paint_span_count: usize,
    demand_batches: Box<[UiGlyphRasterDemandBatch]>,
    glyph_runs: Box<[UiGlyphRunView]>,
    pin_commands: Box<[UiMountedPaintCommandIdentity]>,
    pin_removals: Box<[UiMountedPaintCommandIdentity]>,
    pin_set_complete: bool,
    planning: Option<UiNativeTextDemandInspection>,
    performed_layout_work: [u64; 17],
    #[cfg(test)]
    foreground_reused: bool,
}

pub(crate) struct UiNativeTextPresentationDenial {
    readiness: UiNativeTextPresentationReadiness,
}

pub(crate) fn prepare_mounted_semantic_text<'work>(
    work: UiMountedPresentationWorkView<'work>,
    dpi: UiMountedEventTimeDpiAuthority,
    resolve: impl Fn(
        worth_ui_host_contract::UiQualifiedTextLayoutIdentity,
    ) -> Option<&'work worth_ui_text::UiQualifiedTextLayout>,
) -> Option<UiNativeTextPresentationPreparation> {
    let pin_work = mounted_semantic_text(work);
    // Every mounted successor, including unchanged paint, carries Query
    // currentness through host acceptance. Empty demand preserves retained
    // commands and pins without preparing layout or raster work. Physical
    // samples retain the mounted frame and have their own acceptance path.
    if matches!(work, UiMountedPresentationWorkView::Sample(_)) {
        return None;
    }
    let lane = lane_for(work);
    let join = MountedTextDemandJoin {
        dpi,
        lane,
        // Native retention replaces each selected command's entire glyph-run set.
        // Damage narrows replay, not the evidence retained for future replay.
        selection: worth_ui_text::UiGlyphRasterDemandSelection::CompleteLayout,
        resolve,
        _layout: std::marker::PhantomData,
    };
    Some(match prepare_demands(&pin_work.mechanics, &join) {
        Ok(demands) => inspect_demand_boundary(&pin_work, demands),
        Err(readiness) => denied_preparation(readiness),
    })
}

pub(crate) fn prepare_from_foreground_reuse<'work>(
    work: UiMountedPresentationWorkView<'work>,
    receipts: &[UiMountedTextForegroundReuseReceipt],
    basis: UiMountedTextForegroundPresentationBasis,
    resolve: impl Fn(
        worth_ui_host_contract::UiQualifiedTextLayoutIdentity,
    ) -> Option<&'work worth_ui_text::UiQualifiedTextLayout>,
) -> Option<UiNativeTextPresentationPrepared> {
    let pin_work = mounted_semantic_text(work);
    if pin_work.mechanics.len() != receipts.len() || receipts.is_empty() {
        return None;
    }
    let layouts: Vec<&'work worth_ui_text::UiQualifiedTextLayout> = pin_work
        .mechanics
        .iter()
        .zip(receipts)
        .map(|((command, mechanic), receipt)| {
            receipt
                .accepts_successor(*command, mechanic, basis)
                .then(|| resolve(mechanic.qualified_layout_identity()))?
        })
        .collect::<Option<Vec<_>>>()?;
    let demands = receipts
        .iter()
        .map(|receipt| receipt.demand().clone())
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let glyph_runs = rebuild_glyph_runs(&pin_work.mechanics, &layouts, &demands);
    Some(UiNativeTextPresentationPrepared {
        layout_count: demands.len(),
        paint_span_count: paint_span_count(&pin_work.mechanics),
        demand_batches: demands,
        glyph_runs,
        pin_commands: pin_work
            .mechanics
            .iter()
            .map(|(identity, _)| *identity)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        pin_removals: pin_work.removals.into_boxed_slice(),
        pin_set_complete: pin_work.complete,
        planning: Some(UiNativeTextDemandInspection {
            demand_batches: 0,
            demand_records: 0,
            key_checks: 0,
        }),
        performed_layout_work: [0; 17],
        #[cfg(test)]
        foreground_reused: true,
    })
}

fn canonical_damage(damage: &[UiMountedLogicalDamage]) -> Vec<UiMountedLogicalDamage> {
    let mut canonical = damage.to_vec();
    canonical.sort_unstable_by_key(|region| {
        let bounds = region.bounds();
        (
            bounds.x().to_bits(),
            bounds.y().to_bits(),
            bounds.width().to_bits(),
            bounds.height().to_bits(),
        )
    });
    canonical.dedup_by_key(|region| {
        let bounds = region.bounds();
        (
            bounds.x().to_bits(),
            bounds.y().to_bits(),
            bounds.width().to_bits(),
            bounds.height().to_bits(),
        )
    });
    canonical
}

pub(crate) fn presentation_damage_digest(work: UiMountedPresentationWorkView<'_>) -> [u8; 32] {
    let damage = canonical_damage(logical_damage(work));
    let mut digest = Sha256::new();
    digest.update((damage.len() as u64).to_le_bytes());
    for region in damage {
        let bounds = region.bounds();
        digest.update(bounds.x().to_bits().to_le_bytes());
        digest.update(bounds.y().to_bits().to_le_bytes());
        digest.update(bounds.width().to_bits().to_le_bytes());
        digest.update(bounds.height().to_bits().to_le_bytes());
    }
    digest.finalize().into()
}

fn inspect_demand_boundary(
    pin_work: &MountedSemanticTextWork<'_>,
    prepared: PreparedDemand,
) -> UiNativeTextPresentationPreparation {
    let demand_batches = u32::try_from(prepared.demands.len()).unwrap_or(u32::MAX);
    let demand_records = prepared
        .demands
        .iter()
        .map(|demand| u32::try_from(demand.records().len()).unwrap_or(u32::MAX))
        .fold(0_u32, u32::saturating_add);
    let inspection = UiNativeTextDemandInspection {
        demand_batches,
        demand_records,
        key_checks: demand_records,
    };
    UiNativeTextPresentationPreparation::Prepared(UiNativeTextPresentationPrepared {
        layout_count: prepared.demands.len(),
        paint_span_count: paint_span_count(&pin_work.mechanics),
        demand_batches: prepared.demands,
        glyph_runs: prepared.glyph_runs,
        pin_commands: pin_work
            .mechanics
            .iter()
            .map(|(identity, _)| *identity)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        pin_removals: pin_work.removals.clone().into_boxed_slice(),
        pin_set_complete: pin_work.complete,
        planning: Some(inspection),
        performed_layout_work: layout_work_counts(&pin_work.mechanics),
        #[cfg(test)]
        foreground_reused: false,
    })
}

impl UiNativeTextPresentationPrepared {
    pub(crate) fn layout_count(&self) -> usize {
        self.layout_count
    }

    pub(crate) fn paint_span_count(&self) -> usize {
        self.paint_span_count
    }

    pub(crate) fn demand_batches(&self) -> &[UiGlyphRasterDemandBatch] {
        &self.demand_batches
    }

    pub(crate) fn glyph_runs(&self) -> &[UiGlyphRunView] {
        &self.glyph_runs
    }

    pub(crate) fn pin_commands(&self) -> &[UiMountedPaintCommandIdentity] {
        &self.pin_commands
    }

    pub(crate) fn pin_removals(&self) -> &[UiMountedPaintCommandIdentity] {
        &self.pin_removals
    }

    pub(crate) const fn pin_set_complete(&self) -> bool {
        self.pin_set_complete
    }

    pub(crate) const fn planning_inspection(&self) -> Option<UiNativeTextDemandInspection> {
        self.planning
    }

    #[cfg(test)]
    pub(crate) const fn raster_work(&self) -> UiNativeTextRasterWorkReport {
        UiNativeTextRasterWorkReport::not_admitted()
    }

    pub(crate) const fn performed_layout_work(&self) -> [u64; 17] {
        self.performed_layout_work
    }

    #[cfg(test)]
    pub(crate) const fn foreground_reused(&self) -> bool {
        self.foreground_reused
    }
}

impl UiNativeTextPresentationDenial {
    pub(crate) const fn readiness(&self) -> UiNativeTextPresentationReadiness {
        self.readiness
    }
}

fn denied_preparation(
    readiness: UiNativeTextPresentationReadiness,
) -> UiNativeTextPresentationPreparation {
    UiNativeTextPresentationPreparation::Denied(UiNativeTextPresentationDenial { readiness })
}

fn paint_span_count(mechanics: &[MountedSemanticTextCommand<'_>]) -> usize {
    mechanics
        .iter()
        .map(|(_, mechanic)| mechanic.foregrounds().len())
        .sum()
}

fn layout_work_counts(mechanics: &[MountedSemanticTextCommand<'_>]) -> [u64; 17] {
    mechanics
        .iter()
        .filter_map(|(_, mechanic)| mechanic.performed_layout_cost())
        .fold([0; 17], |mut total, cost| {
            let row = [
                cost.analyzed_bytes() as u64,
                cost.graphemes() as u64,
                cost.word_boundaries() as u64,
                cost.line_opportunities() as u64,
                cost.bidi_contexts() as u64,
                cost.fallback_clusters() as u64,
                cost.coverage_index_queries() as u64,
                cost.face_shape_attempts() as u64,
                cost.probed_glyphs() as u64,
                cost.shaped_runs() as u64,
                cost.shaped_scalars() as u64,
                cost.emitted_glyphs() as u64,
                cost.fitted_units() as u64,
                cost.emitted_lines() as u64,
                cost.emitted_visual_runs() as u64,
                cost.positioned_glyphs() as u64,
                cost.emitted_carets() as u64,
            ];
            for (slot, performed) in total.iter_mut().zip(row) {
                *slot = slot.saturating_add(performed);
            }
            total
        })
}

fn lane_for(work: UiMountedPresentationWorkView<'_>) -> UiGlyphRasterLane {
    match work {
        UiMountedPresentationWorkView::Reconstruction(_) => UiGlyphRasterLane::Reconstruction,
        UiMountedPresentationWorkView::Initial(_)
        | UiMountedPresentationWorkView::Delta(_)
        | UiMountedPresentationWorkView::Sample(_)
        | UiMountedPresentationWorkView::Unchanged(_) => UiGlyphRasterLane::Ordinary,
    }
}

#[cfg(test)]
#[path = "preparation_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "preparation/retained_demand_tests.rs"]
mod retained_demand_tests;

#[cfg(test)]
#[path = "preparation/currentness_tests.rs"]
mod currentness_tests;
