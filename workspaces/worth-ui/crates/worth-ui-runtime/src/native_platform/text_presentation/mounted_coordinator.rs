//! Ordinary runtime coordinator for mounted native text pin transactions.

use std::collections::HashMap;

use crate::mounting::presentation::coordinator::{
    UiMountedTextPinCandidate, UiMountedTextPinState,
};
use crate::mounting::{
    UiMountedTextForegroundPresentationBasis, UiMountedTextForegroundReuseReceipt,
};
use worth_ui_host_contract::{
    UiMountedPaintCommandIdentity, UiMountedPresentationWorkView, UiSurfaceBindingGeneration,
};

use super::{
    prepare_from_foreground_reuse, presentation_damage_digest, UiMountedEventTimeDpiAuthority,
    UiNativeTextAtlasTransaction, UiNativeTextPresentationPreparation,
    UiNativeTextPresentationPrepared,
};

#[derive(Default)]
pub(crate) struct UiNativeMountedTextCoordinator {
    pins: UiMountedTextPinState,
    retained_mechanics: std::collections::HashMap<
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
    outcome: worth_ui_host_contract::UiHostSurfacePresentationOutcome,
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
        let retained: Option<Vec<UiMountedTextForegroundReuseReceipt>> = semantic_work
            .mechanics
            .iter()
            .map(|(command, _)| self.foreground_receipts.get(command).cloned())
            .collect::<Option<Vec<UiMountedTextForegroundReuseReceipt>>>();
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
                    && retained.iter().all(|receipt| {
                        receipt.pins_are_continuous(UiMountedTextPinState::binding_pins(&candidate))
                    });
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
            worth_ui_host_contract::UiHostSurfacePresentationOutcome,
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
        let work_observation = request_bases.first().map(|basis| {
            let key = [
                basis.mounted_frame().diagnostic_value(),
                basis.binding().diagnostic_value(),
            ];
            let layout_work = if self.admit_layout_work(key) {
                prepared.performed_layout_work()
            } else {
                [0; 17]
            };
            let (active_mechanics, removed_mechanics) = self.advance_mechanic_evidence(basis);
            super::UiNativeTextPresentationWorkObservation::after_mounted_work(
                basis,
                prepared,
                raster_work,
                layout_work,
                active_mechanics,
                removed_mechanics,
            )
        });
        if reconstruction_required {
            self.raster_cache = reconstructed_cache;
            self.raster_cache_reconstruction_required = false;
        }
        self.peak_raster_cache_entries =
            self.peak_raster_cache_entries.max(self.raster_cache.len());
        if let Some(observation) = work_observation {
            self.record_work_observation(observation);
        }
        let pending_candidate = match outcome {
            worth_ui_host_contract::UiHostSurfacePresentationOutcome::Presented(_) => {
                self.pins.commit_presented(candidate);
                None
            }
            worth_ui_host_contract::UiHostSurfacePresentationOutcome::InFlight(_) => {
                Some(candidate)
            }
            worth_ui_host_contract::UiHostSurfacePresentationOutcome::RejectedBeforeEffects(
                worth_ui_host_contract::UiHostSurfacePresentationDenial::TextAtlasPresentationDeferred,
            ) => {
                self.pins.commit_presented(candidate);
                None
            }
            worth_ui_host_contract::UiHostSurfacePresentationOutcome::RejectedBeforeEffects(_)
            | worth_ui_host_contract::UiHostSurfacePresentationOutcome::PresentationIndeterminate => {
                None
            }
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
        let receipts = semantic_work
            .mechanics
            .iter()
            .zip(prepared.demand_batches())
            .map(|((command, mechanic), demand)| {
                UiMountedTextForegroundReuseReceipt::from_prepared(
                    *command,
                    mechanic,
                    demand,
                    UiMountedTextPinState::binding_pins(candidate),
                    basis,
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
                .collect::<Vec<_>>();
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

    pub(crate) fn take_work_observations(
        &mut self,
    ) -> (Box<[super::UiNativeTextPresentationWorkObservation]>, bool) {
        (
            std::mem::take(&mut self.work_observations).into_boxed_slice(),
            !std::mem::take(&mut self.work_observation_overflowed),
        )
    }

    fn record_work_observation(
        &mut self,
        observation: super::UiNativeTextPresentationWorkObservation,
    ) {
        if self.work_observations.len() == TEXT_WORK_OBSERVATION_CAPACITY {
            self.work_observation_overflowed = true;
            return;
        }
        self.work_observations.push(observation);
    }

    fn admit_layout_work(&mut self, key: [u64; 2]) -> bool {
        if self.reported_layout_work.contains(&key) {
            return false;
        }
        if self.reported_layout_work.len() == TEXT_WORK_OBSERVATION_CAPACITY {
            self.reported_layout_work.pop_front();
        }
        self.reported_layout_work.push_back(key);
        true
    }

    fn advance_mechanic_evidence(
        &mut self,
        basis: &worth_ui_query_binding::WorthUiPresentationRequestBasis,
    ) -> (
        Box<[super::UiNativeTextPresentationMechanicObservation]>,
        Box<[super::UiNativeTextPresentationMechanicObservation]>,
    ) {
        let mut removed = if basis.complete() {
            std::mem::take(&mut self.retained_mechanics)
                .into_values()
                .collect::<Vec<_>>()
        } else {
            basis
                .removed_mechanics()
                .iter()
                .filter_map(|identity| self.retained_mechanics.remove(identity))
                .collect::<Vec<_>>()
        };
        let active = basis
            .mechanics()
            .iter()
            .map(super::UiNativeTextPresentationMechanicObservation::from_basis)
            .collect::<Vec<_>>();
        for mechanic in &active {
            self.retained_mechanics
                .insert(mechanic.mechanic(), *mechanic);
        }
        removed.sort_by_key(|mechanic| {
            let identity = mechanic.mechanic();
            let (slot, row) = identity
                .semantic_text_identity_parts()
                .expect("retained text mechanic preserves semantic-text identity");
            (identity.mounted_instance().diagnostic_value(), slot, row)
        });
        (active.into_boxed_slice(), removed.into_boxed_slice())
    }
}

impl UiNativeMountedSurfaceTextObservation {
    pub(crate) fn into_parts(
        self,
    ) -> (
        worth_ui_host_contract::UiHostSurfacePresentationOutcome,
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

#[cfg(test)]
#[path = "mounted_coordinator_tests.rs"]
mod tests;
