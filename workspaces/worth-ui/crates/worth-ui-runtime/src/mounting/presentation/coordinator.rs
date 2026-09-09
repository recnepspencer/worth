use std::collections::{BTreeMap, BTreeSet};
use std::{cell::RefCell, rc::Rc};
use worth_ui_host_contract::{
    UiMountedPresentationAttemptIdentity, UiPresentationDeadline, UiSurfaceBindingGeneration,
};

use super::consumption_view::UiMountedHostPresentationAuthority;
use super::outcome::{
    UiMountedIndeterminateFrame, UiMountedPresentationOutcome, UiMountedPresentationReceipt,
    UiMountedSurfacePresentationReceipt, UiMountedSurfacePresentationRejection,
    UiPresentationIndeterminateReport,
};
use super::preflight::validate_before_effects;
use super::terminal::{
    aggregate_affected, frame_rejections, rejected_outcome, UiIndeterminatePresentationEvidence,
};
use super::{UiMountedPresentationAttempt, UiMountedPresentationInFlight};
use crate::facade::UiHostEffectPort;

mod admission;
mod cancellation;
mod cancellation_settlement;
mod candidate_preparation;
mod duplicate_observation;
mod host_truth;
mod motion_evidence;
mod motion_sample;
mod pending_completion;
mod physical_uncertainty;
mod presentation_attempt;
mod presentation_outcome;
mod presented;
mod presented_semantic_settlement;
mod raster_cache_reconstruction;
mod semantic_text_raster;
mod settlement;
mod superseding_admission;
mod surface_binding;
mod surface_uncertainty;
mod text_pins;
mod work_preparation;

pub(crate) use candidate_preparation::UiAcceptedAppearanceMotion;
pub(super) use candidate_preparation::UiPreparedFrameCandidates;
pub(crate) use motion_sample::UiMotionSamplePresentationOutcome;
use presentation_attempt::{
    present_one_surface, UiMountedPresentationProgress, UiMountedPresentationStart,
};
pub(crate) use text_pins::{UiMountedTextPinCandidate, UiMountedTextPinState};

const DEFAULT_IN_FLIGHT_LIMIT: usize = 2;

pub struct UiMountedPresentationCoordinator {
    shutting_down: bool,
    in_flight_limit: usize,
    active: Rc<RefCell<BTreeSet<UiMountedPresentationAttemptIdentity>>>,
    in_flight: BTreeMap<
        UiMountedPresentationAttemptIdentity,
        super::state::UiMountedPresentationInFlightState,
    >,
    appearance_attempts: BTreeMap<
        UiMountedPresentationAttemptIdentity,
        crate::runtime::appearance::UiAppearanceInspectionAttemptBatch,
    >,
    unresolved_semantic_receipts: BTreeMap<
        UiMountedPresentationAttemptIdentity,
        Vec<worth_ui_query_binding::WorthUiPresentationRecoveryRequiredReceipt>,
    >,
    unresolved_semantic_recoveries: BTreeMap<
        UiMountedPresentationAttemptIdentity,
        Vec<worth_ui_query_binding::WorthUiPresentationRecoveryReceipt>,
    >,
    presentation_states: super::work_producer::UiMountedPresentationCandidates,
    motion_sample_in_flight: Option<motion_sample::UiPendingMotionSamplePresentation>,
    reconstruction_bindings: BTreeSet<UiSurfaceBindingGeneration>,
    text: crate::native_platform::text_presentation::UiNativeMountedTextCoordinator,
    presentation_async:
        Option<crate::native_platform::text_presentation::UiPresentationAsyncRuntime>,
    host_truth: crate::mounting::UiMountedHostTruthCoordinator,
    pub(super) focus_placement: super::focus_placement::UiMountedFocusPlacementState,
}

struct UiMountedPresentationSettlement<'host> {
    frame: super::super::UiPreparedMountedFrame,
    retention: super::super::retention::UiMountedRetentionReservation,
    attempt: UiMountedPresentationAttemptIdentity,
    deadline: UiPresentationDeadline,
    pending: Vec<super::state::UiPendingMountedSurface>,
    rejected: Vec<UiMountedSurfacePresentationRejection>,
    completed: Vec<UiMountedSurfacePresentationReceipt>,
    superseded_costs: Vec<worth_ui_host_contract::UiHostPresentationCostReport>,
    semantic_requests: Vec<worth_ui_query_binding::WorthUiPresentationRequestBasis>,
    superseded: bool,
    reconstructed_bindings: Vec<UiSurfaceBindingGeneration>,
    candidates: super::work_producer::UiMountedPresentationCandidates,
    host: UiHostEffectPort<'host>,
}

impl Default for UiMountedPresentationCoordinator {
    fn default() -> Self {
        Self {
            shutting_down: false,
            in_flight_limit: DEFAULT_IN_FLIGHT_LIMIT,
            active: Rc::new(RefCell::new(BTreeSet::new())),
            in_flight: BTreeMap::new(),
            appearance_attempts: BTreeMap::new(),
            unresolved_semantic_receipts: BTreeMap::new(),
            unresolved_semantic_recoveries: BTreeMap::new(),
            presentation_states: BTreeMap::new(),
            motion_sample_in_flight: None,
            reconstruction_bindings: BTreeSet::new(),
            text: Default::default(),
            presentation_async: None,
            host_truth: Default::default(),
            focus_placement: Default::default(),
        }
    }
}

impl UiMountedPresentationCoordinator {
    pub(crate) fn new(
        presentation_async: Option<
            crate::native_platform::text_presentation::UiPresentationAsyncRuntime,
        >,
    ) -> Self {
        Self {
            presentation_async,
            ..Self::default()
        }
    }

    pub(crate) fn present(
        &mut self,
        attempt: UiMountedPresentationAttempt,
        host: UiHostEffectPort<'_>,
        authority: UiMountedHostPresentationAuthority<'_>,
        now: u64,
    ) -> UiMountedPresentationOutcome {
        let (frame, retention, attempt, deadline, candidates) = attempt.into_parts();
        if deadline.expired_at(now) {
            self.active.borrow_mut().remove(&attempt);
            let rejections = frame_rejections(
                &frame,
                worth_ui_host_contract::UiHostSurfacePresentationDenial::DeadlineExpired,
            );
            return rejected_outcome(attempt, frame, retention, rejections);
        }
        self.present_all(
            UiMountedPresentationStart {
                frame,
                retention,
                attempt,
                deadline,
                host,
                authority,
            },
            candidates,
        )
    }

    pub(crate) fn retain_appearance_attempt(
        &mut self,
        attempt: UiMountedPresentationAttemptIdentity,
        batch: crate::runtime::appearance::UiAppearanceInspectionAttemptBatch,
    ) {
        let replaced = self.appearance_attempts.insert(attempt, batch);
        assert!(
            replaced.is_none(),
            "runtime-minted presentation attempts have one appearance batch"
        );
    }

    pub(crate) fn take_appearance_attempt(
        &mut self,
        attempt: UiMountedPresentationAttemptIdentity,
    ) -> Option<crate::runtime::appearance::UiAppearanceInspectionAttemptBatch> {
        self.appearance_attempts.remove(&attempt)
    }

    fn present_all(
        &mut self,
        start: UiMountedPresentationStart<'_, '_>,
        candidates: UiPreparedFrameCandidates,
    ) -> UiMountedPresentationOutcome {
        if let Err(rejections) =
            validate_before_effects(&start.frame, start.host.adapter(), start.authority)
        {
            self.active.borrow_mut().remove(&start.attempt);
            return rejected_outcome(start.attempt, start.frame, start.retention, rejections);
        }
        let prepared = match work_preparation::issue(
            candidates,
            &start.frame,
            &self.presentation_states,
            &self.reconstruction_bindings,
            &start.authority,
        ) {
            Ok(prepared) => prepared,
            Err(denial) => {
                self.active.borrow_mut().remove(&start.attempt);
                let rejections = frame_rejections(&start.frame, denial);
                return rejected_outcome(start.attempt, start.frame, start.retention, rejections);
            }
        };
        let mut progress = UiMountedPresentationProgress::default();
        let reconstructed_bindings = start
            .frame
            .surfaces()
            .iter()
            .zip(&prepared.surfaces)
            .filter_map(|(surface, prepared)| {
                matches!(
                    prepared.work.view(),
                    worth_ui_host_contract::UiMountedPresentationWorkView::Reconstruction(_)
                )
                .then_some(surface.requirement().binding())
            })
            .collect();
        for (surface, prepared_surface) in start.frame.surfaces().iter().zip(&prepared.surfaces) {
            if let Err(evidence) = present_one_surface(
                &start,
                surface,
                &prepared_surface.work,
                &prepared_surface.expected_effects,
                &mut progress,
                &mut self.text,
                self.presentation_async.as_mut(),
            ) {
                return self.indeterminate(start.frame, start.retention, start.attempt, evidence);
            }
        }
        self.finish_or_wait(UiMountedPresentationSettlement {
            frame: start.frame,
            retention: start.retention,
            attempt: start.attempt,
            deadline: start.deadline,
            pending: progress.pending,
            rejected: progress.rejected,
            completed: progress.completed,
            superseded_costs: progress.superseded_costs,
            semantic_requests: progress.semantic_requests,
            superseded: progress.superseded,
            reconstructed_bindings,
            candidates: prepared.candidates,
            host: start.host,
        })
    }

    fn finish_or_wait(
        &mut self,
        settlement: UiMountedPresentationSettlement<'_>,
    ) -> UiMountedPresentationOutcome {
        if !settlement.pending.is_empty() {
            return self.retain_in_flight(settlement);
        }
        if settlement.superseded {
            return self.finish_superseded(settlement);
        }
        if settlement.completed.is_empty() {
            return self.finish_rejected(settlement);
        }
        if !settlement.rejected.is_empty() {
            return self.finish_partially_presented(settlement);
        }
        self.finish_presented(settlement)
    }

    fn retain_in_flight(
        &mut self,
        settlement: UiMountedPresentationSettlement<'_>,
    ) -> UiMountedPresentationOutcome {
        let cost = match UiMountedPresentationReceipt::compose_cost_with_additional(
            settlement.frame.cost_report(),
            &settlement.completed,
            &settlement.superseded_costs,
        ) {
            Ok(cost) => cost,
            Err(_) => {
                let affected = aggregate_affected(
                    &settlement.completed,
                    &settlement.pending,
                    &settlement.rejected,
                );
                let stopped = cancellation::cancel_all(settlement.pending, settlement.host);
                let cancelled = cancellation_settlement::settle(
                    stopped,
                    self.presentation_async.as_mut(),
                    worth_ui_host_contract::UiHostSurfacePresentationDenial::CancelledBeforeEffects,
                );
                let (_, semantic_receipts, recovery_required, physical_recovery_bindings) =
                    cancelled.into_parts();
                return self.indeterminate(
                    settlement.frame,
                    settlement.retention,
                    settlement.attempt,
                    UiIndeterminatePresentationEvidence::new(affected, settlement.completed)
                        .with_semantic_receipts(semantic_receipts)
                        .with_recovery_required(recovery_required)
                        .with_physical_recovery_bindings(physical_recovery_bindings),
                );
            }
        };
        let state = super::state::UiMountedPresentationInFlightState {
            frame: settlement.frame,
            retention: settlement.retention,
            attempt: settlement.attempt,
            deadline: settlement.deadline,
            pending: settlement.pending,
            rejected: settlement.rejected,
            completed: settlement.completed,
            superseded_costs: settlement.superseded_costs,
            semantic_requests: settlement.semantic_requests,
            superseded: settlement.superseded,
            reconstructed_bindings: settlement.reconstructed_bindings,
            candidates: settlement.candidates,
        };
        let handle = UiMountedPresentationInFlight::from_state(&state, cost);
        self.in_flight.insert(state.attempt, state);
        UiMountedPresentationOutcome::InFlight(handle)
    }

    fn finish_rejected(
        &mut self,
        settlement: UiMountedPresentationSettlement<'_>,
    ) -> UiMountedPresentationOutcome {
        self.active.borrow_mut().remove(&settlement.attempt);
        for rejection in &settlement.rejected {
            if rejection.denial()
                == worth_ui_host_contract::UiHostSurfacePresentationDenial::ReconstructionRequired
            {
                if let Some(requirement) = settlement
                    .frame
                    .surfaces()
                    .iter()
                    .find(|surface| surface.requirement().binding() == rejection.binding())
                    .map(|surface| surface.requirement())
                {
                    self.host_truth.block_presentation(requirement);
                }
                self.presentation_states.remove(&rejection.binding());
            }
        }
        rejected_outcome(
            settlement.attempt,
            settlement.frame,
            settlement.retention,
            settlement.rejected,
        )
    }

    fn finish_partially_presented(
        &mut self,
        settlement: UiMountedPresentationSettlement<'_>,
    ) -> UiMountedPresentationOutcome {
        let affected = aggregate_affected(&settlement.completed, &[], &settlement.rejected);
        self.indeterminate(
            settlement.frame,
            settlement.retention,
            settlement.attempt,
            UiIndeterminatePresentationEvidence::new(affected, settlement.completed),
        )
    }

    fn indeterminate(
        &mut self,
        frame: super::super::UiPreparedMountedFrame,
        retention: super::super::retention::UiMountedRetentionReservation,
        attempt: UiMountedPresentationAttemptIdentity,
        evidence: UiIndeterminatePresentationEvidence,
    ) -> UiMountedPresentationOutcome {
        let (affected, cost, semantic_receipts, recovery_required, physical_recovery_bindings) =
            evidence.into_terminal_parts(frame.cost_report());
        self.retain_semantic_uncertainty(attempt, semantic_receipts);
        if !recovery_required.is_empty() {
            self.unresolved_semantic_receipts
                .entry(attempt)
                .or_default()
                .extend(recovery_required);
        }
        self.active.borrow_mut().remove(&attempt);
        for binding in &affected {
            self.presentation_states.remove(binding);
            let requirement = frame
                .surfaces()
                .iter()
                .find(|surface| surface.requirement().binding() == *binding)
                .expect("affected binding belongs to the retained prepared frame")
                .requirement();
            self.host_truth.block_presentation(requirement);
        }
        drop(retention);
        let report =
            UiPresentationIndeterminateReport::new(attempt, affected, physical_recovery_bindings);
        UiMountedPresentationOutcome::PresentationIndeterminate(UiMountedIndeterminateFrame::new(
            frame, report, cost,
        ))
    }
}
