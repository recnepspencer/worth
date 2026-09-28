//! Ordinary runtime coordinator for mounted native text pin transactions.

use std::collections::{HashMap, HashSet};

use crate::mounting::presentation::coordinator::{
    UiMountedTextPinCandidate, UiMountedTextPinState,
};
use crate::mounting::{
    UiMountedTextForegroundPresentationBasis, UiMountedTextForegroundReuseReceipt,
};
use worth_ui_host_contract::{
    UiHostPresentationProgressClass, UiHostSurfacePresentationDenial,
    UiHostSurfacePresentationOutcome, UiMountedPaintCommandIdentity, UiMountedPresentationWorkView,
    UiSurfaceBindingGeneration,
};

use super::{
    prepare_from_foreground_reuse, presentation_damage_digest, UiMountedEventTimeDpiAuthority,
    UiNativeTextAtlasTransaction, UiNativeTextPresentationPreparation,
    UiNativeTextPresentationPrepared,
};

#[derive(Default)]
pub(crate) struct UiNativeMountedTextCoordinator {
    pins: UiMountedTextPinState,
    retained_mechanics: HashMap<
        worth_ui_host_contract::UiMountedPaintCommandIdentity,
        super::UiNativeTextPresentationMechanicObservation,
    >,
    work_observations: Vec<super::UiNativeTextPresentationWorkObservation>,
    work_observation_overflowed: bool,
    reported_layout_work: std::collections::VecDeque<[u64; 2]>,
    raster_cache: worth_ui_text::UiGlyphRasterCache,
    raster_cache_reconstruction_required: bool,
    reconstructed_raster_cache_items: usize,
    peak_raster_cache_entries: usize,
    foreground_receipts:
        HashMap<UiMountedPaintCommandIdentity, UiMountedTextForegroundReuseReceipt>,
}

const TEXT_WORK_OBSERVATION_CAPACITY: usize = 64;

pub(crate) struct UiNativeMountedSurfaceTextObservation {
    outcome: UiHostSurfacePresentationOutcome,
    pending_candidate: Option<UiMountedTextPinCandidate>,
    request_bases: Box<[worth_ui_query_binding::WorthUiPresentationRequestBasis]>,
    pending_receipts: Box<[worth_ui_query_binding::WorthUiPresentationRecoveryReceipt]>,
    foreground_reuse: Option<UiMountedTextForegroundReuseUpdate>,
}

pub(crate) struct UiMountedTextForegroundReuseUpdate {
    binding: UiSurfaceBindingGeneration,
    complete: bool,
    retirements: Box<[UiMountedPaintCommandIdentity]>,
    receipts: Box<[UiMountedTextForegroundReuseReceipt]>,
}

impl UiNativeMountedTextCoordinator {
    pub(crate) fn prepare_mounted_semantic_text<'work>(
        &self,
        work: UiMountedPresentationWorkView<'work>,
        requirement: worth_ui_host_contract::UiMountedSurfaceBindingRequirement,
        dpi: UiMountedEventTimeDpiAuthority,
        host_lineage: Option<worth_ui_host_contract::UiHostPresentationLineageIdentity>,
        resolve: impl Fn(
            worth_ui_host_contract::UiQualifiedTextLayoutIdentity,
        ) -> Option<&'work worth_ui_text::UiQualifiedTextLayout>,
    ) -> Option<UiNativeTextPresentationPreparation> {
        let basis = UiMountedTextForegroundPresentationBasis::from_work(
            work,
            requirement,
            dpi.dpi_milli(),
            host_lineage,
            presentation_damage_digest(work),
        );
        let semantic_work = super::mounted_semantic_text(work);
        let retained: Option<Vec<&UiMountedTextForegroundReuseReceipt>> = semantic_work
            .mechanics
            .iter()
            .map(|(command, _)| self.foreground_receipts.get(command))
            .collect::<Option<Vec<&UiMountedTextForegroundReuseReceipt>>>();
        if let Some(retained) = retained.filter(|receipts| {
            !self.raster_cache_reconstruction_required
                && receipts
                    .iter()
                    .all(|receipt| receipt.raster_keys_cached(&self.raster_cache))
        }) {
            if let Some(prepared) = prepare_from_foreground_reuse(work, &retained, basis, &resolve)
            {
                let candidate = self.pins.candidate(requirement.binding(), &prepared);
                let pins_continue = candidate.has_no_pin_churn()
                    && UiMountedTextForegroundReuseReceipt::pins_are_continuous(
                        retained.iter().copied(),
                        UiMountedTextPinState::binding_pins(&candidate),
                    );
                if pins_continue {
                    return Some(UiNativeTextPresentationPreparation::Prepared(prepared));
                }
            }
        }
        super::prepare_mounted_semantic_text(work, dpi, resolve)
    }

    pub(crate) fn present_with_mounted_work<'layout>(
        &mut self,
        binding: UiSurfaceBindingGeneration,
        work: UiMountedPresentationWorkView<'layout>,
        requirement: worth_ui_host_contract::UiMountedSurfaceBindingRequirement,
        host_lineage: Option<worth_ui_host_contract::UiHostPresentationLineageIdentity>,
        prepared: &'layout UiNativeTextPresentationPrepared,
        resolve: impl Fn(
            worth_ui_host_contract::UiQualifiedTextLayoutIdentity,
        ) -> Option<&'layout worth_ui_text::UiQualifiedTextLayout>,
        present: impl FnOnce(
            &worth_ui_host_contract::UiMountedTextRasterWork<'_>,
        ) -> (
            UiHostSurfacePresentationOutcome,
            Box<[worth_ui_query_binding::WorthUiPresentationRequestBasis]>,
            Box<[worth_ui_query_binding::WorthUiPresentationRecoveryReceipt]>,
        ),
    ) -> Option<UiNativeMountedSurfaceTextObservation> {
        let candidate = self.pins.candidate(binding, prepared);
        let foreground_reuse = self.foreground_reuse_update(
            binding,
            work,
            requirement,
            host_lineage,
            prepared,
            &candidate,
        );
        let transition = UiMountedTextPinState::transition_view(&candidate);
        let reconstruction_required = self.raster_cache_reconstruction_required;
        let mut reconstructed_cache = worth_ui_text::UiGlyphRasterCache::default();
        let cache = if reconstruction_required {
            &mut reconstructed_cache
        } else {
            &mut self.raster_cache
        };
        let mut transaction = UiNativeTextAtlasTransaction::prepare(prepared, resolve, cache)?;
        if reconstruction_required {
            if !transaction.reconstruct_cache() {
                return None;
            }
            self.reconstructed_raster_cache_items = transaction.cache_len();
        }
        let ((outcome, request_bases, pending_receipts), raster_work) = transaction
            .with_mounted_work(
                transition,
                UiMountedTextPinState::binding_pins(&candidate),
                present,
            );
        drop(transaction);
        if let Some(basis) = request_bases.first() {
            self.observe_work(basis, prepared, raster_work);
        }
        if reconstruction_required {
            self.raster_cache = reconstructed_cache;
            self.raster_cache_reconstruction_required = false;
        }
        self.peak_raster_cache_entries =
            self.peak_raster_cache_entries.max(self.raster_cache.len());
        let pending_candidate = match &outcome {
            UiHostSurfacePresentationOutcome::Presented(_) => {
                self.pins.commit_presented(candidate);
                None
            }
            // The host commits a frame's text pins before it hands back a
            // physical surface still in flight, and a successor prepared
            // meanwhile must start from them. Only a pending atlas
            // transaction still holds its pins back.
            UiHostSurfacePresentationOutcome::InFlight(token) => match token.progress_class() {
                UiHostPresentationProgressClass::PhysicalSurface => {
                    self.pins.commit_presented(candidate);
                    None
                }
                UiHostPresentationProgressClass::TextAtlas => Some(candidate),
            },
            UiHostSurfacePresentationOutcome::RejectedBeforeEffects(
                UiHostSurfacePresentationDenial::TextAtlasPresentationDeferred,
            ) => {
                self.pins.commit_presented(candidate);
                None
            }
            UiHostSurfacePresentationOutcome::RejectedBeforeEffects(_)
            | UiHostSurfacePresentationOutcome::PresentationIndeterminate => None,
        };
        Some(UiNativeMountedSurfaceTextObservation {
            outcome,
            pending_candidate,
            request_bases,
            pending_receipts,
            foreground_reuse,
        })
    }

    fn foreground_reuse_update<'layout>(
        &self,
        binding: UiSurfaceBindingGeneration,
        work: UiMountedPresentationWorkView<'layout>,
        requirement: worth_ui_host_contract::UiMountedSurfaceBindingRequirement,
        host_lineage: Option<worth_ui_host_contract::UiHostPresentationLineageIdentity>,
        prepared: &'layout UiNativeTextPresentationPrepared,
        candidate: &UiMountedTextPinCandidate,
    ) -> Option<UiMountedTextForegroundReuseUpdate> {
        let semantic_work = super::mounted_semantic_text(work);
        if semantic_work.mechanics.len() != prepared.demand_batches().len() {
            return None;
        }
        let basis = UiMountedTextForegroundPresentationBasis::from_work(
            work,
            requirement,
            requirement.device_scale_milli(),
            host_lineage,
            presentation_damage_digest(work),
        );
        let pins = std::sync::Arc::from(UiMountedTextPinState::binding_pins(candidate));
        let receipts = semantic_work
            .mechanics
            .iter()
            .zip(prepared.demand_batches())
            .map(|((command, mechanic), demand)| {
                UiMountedTextForegroundReuseReceipt::from_prepared(
                    *command, mechanic, demand, &pins, basis,
                )
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        if receipts.is_empty() && semantic_work.removals.is_empty() && !semantic_work.complete {
            return None;
        }
        Some(UiMountedTextForegroundReuseUpdate {
            binding,
            complete: semantic_work.complete,
            retirements: semantic_work.removals.into_boxed_slice(),
            receipts,
        })
    }

    pub(crate) fn commit_foreground_reuse(&mut self, update: UiMountedTextForegroundReuseUpdate) {
        for command in update.retirements {
            self.foreground_receipts.remove(&command);
        }
        if update.complete {
            let active = update
                .receipts
                .iter()
                .map(UiMountedTextForegroundReuseReceipt::command)
                .collect::<HashSet<_>>();
            self.foreground_receipts.retain(|_, receipt| {
                receipt.basis().binding() != update.binding || active.contains(&receipt.command())
            });
        }
        for receipt in update.receipts {
            self.foreground_receipts.insert(receipt.command(), receipt);
        }
    }

    pub(crate) fn commit_surface_candidate(&mut self, candidate: UiMountedTextPinCandidate) {
        self.pins.commit_presented(candidate);
    }

    #[cfg(feature = "certification-support")]
    pub(crate) fn require_raster_cache_reconstruction(&mut self) -> usize {
        let lost = self.raster_cache.clear();
        if lost > 0 {
            self.raster_cache_reconstruction_required = true;
        }
        lost
    }

    pub(crate) fn take_reconstructed_raster_cache_items(&mut self) -> usize {
        std::mem::take(&mut self.reconstructed_raster_cache_items)
    }

    pub(crate) const fn peak_raster_cache_entries(&self) -> usize {
        self.peak_raster_cache_entries
    }

    pub(crate) fn deregistration_candidate(
        &self,
        binding: UiSurfaceBindingGeneration,
    ) -> UiMountedTextPinCandidate {
        self.pins.deregistration_candidate(binding)
    }
}

impl UiNativeMountedSurfaceTextObservation {
    pub(crate) fn into_parts(
        self,
    ) -> (
        UiHostSurfacePresentationOutcome,
        Option<UiMountedTextPinCandidate>,
        Box<[worth_ui_query_binding::WorthUiPresentationRequestBasis]>,
        Box<[worth_ui_query_binding::WorthUiPresentationRecoveryReceipt]>,
        Option<UiMountedTextForegroundReuseUpdate>,
    ) {
        (
            self.outcome,
            self.pending_candidate,
            self.request_bases,
            self.pending_receipts,
            self.foreground_reuse,
        )
    }
}

#[path = "mounted_coordinator/work_evidence.rs"]
mod work_evidence;

#[cfg(test)]
#[path = "mounted_coordinator_tests.rs"]
mod tests;
