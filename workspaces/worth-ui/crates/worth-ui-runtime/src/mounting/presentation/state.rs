use worth_ui_host_contract::{
    UiHostPresentationCompletionToken, UiMountedPresentationAttemptIdentity,
    UiPresentationDeadline, UiSurfaceBindingGeneration,
};

use super::super::UiPreparedMountedFrame;
use super::outcome::{UiMountedSurfacePresentationReceipt, UiMountedSurfacePresentationRejection};

pub struct UiMountedPresentationAdmission {
    pub(super) frame: UiPreparedMountedFrame,
    pub(super) retention: super::super::retention::UiMountedRetentionReservation,
    pub(super) attempt: UiMountedPresentationAttemptIdentity,
    pub(super) deadline: UiPresentationDeadline,
    candidates: super::coordinator::UiPreparedFrameCandidates,
    lease: UiPresentationAdmissionLease,
}

struct UiPresentationAdmissionLease {
    active: Rc<RefCell<BTreeSet<UiMountedPresentationAttemptIdentity>>>,
    attempt: UiMountedPresentationAttemptIdentity,
    release_on_drop: bool,
}

pub struct UiMountedPresentationAttempt {
    admission: UiMountedPresentationAdmission,
}

pub struct UiMountedPresentationAdmissionRejection {
    denial: UiMountedPresentationAdmissionDenial,
    attempt: Option<UiMountedPresentationAttemptIdentity>,
    frame: Box<UiPreparedMountedFrame>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiMountedPresentationInFlight {
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    attempt: UiMountedPresentationAttemptIdentity,
    deadline: UiPresentationDeadline,
    pending_bindings: Box<[UiSurfaceBindingGeneration]>,
    pending_progress_classes: Box<[worth_ui_host_contract::UiHostPresentationProgressClass]>,
    semantic_requests: Box<[worth_ui_query_binding::WorthUiPresentationRequestBasis]>,
    cost: super::super::UiMountCostReport,
    retention: super::super::retention::UiMountedRetentionReservationIdentity,
}

#[derive(Clone, Copy)]
pub(crate) struct UiMountedSupersedingPresentationBasis {
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    attempt: UiMountedPresentationAttemptIdentity,
    retention: super::super::retention::UiMountedRetentionReservationIdentity,
}

pub(super) struct UiMountedPresentationInFlightState {
    pub(super) frame: UiPreparedMountedFrame,
    pub(super) retention: super::super::retention::UiMountedRetentionReservation,
    pub(super) attempt: UiMountedPresentationAttemptIdentity,
    pub(super) deadline: UiPresentationDeadline,
    pub(super) pending: Vec<UiPendingMountedSurface>,
    pub(super) rejected: Vec<UiMountedSurfacePresentationRejection>,
    pub(super) completed: Vec<UiMountedSurfacePresentationReceipt>,
    pub(super) superseded_costs: Vec<worth_ui_host_contract::UiHostPresentationCostReport>,
    pub(super) semantic_requests: Vec<worth_ui_query_binding::WorthUiPresentationRequestBasis>,
    pub(super) superseded: bool,
    pub(super) reconstructed_bindings: Vec<UiSurfaceBindingGeneration>,
    pub(super) candidates: super::work_producer::UiMountedPresentationCandidates,
}

pub(super) struct UiPendingMountedSurface {
    pub(super) binding: UiSurfaceBindingGeneration,
    pub(super) token: UiHostPresentationCompletionToken,
    pub(super) expected_effects: Box<[worth_ui_host_contract::UiMountedEffectFamily]>,
    pub(super) text_candidate: Option<super::coordinator::UiMountedTextPinCandidate>,
    pub(super) semantic_receipts: Box<[worth_ui_query_binding::WorthUiPresentationRecoveryReceipt]>,
    pub(super) text_reuse:
        Option<crate::native_platform::text_presentation::UiMountedTextForegroundReuseUpdate>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedPresentationAdmissionDenial {
    CoordinatorShuttingDown,
    CapacityExceeded,
    DeadlineExpired,
    PreparedFrameBasisChanged,
    CapabilityGenerationChanged(UiSurfaceBindingGeneration),
    CapabilityProfileChanged(UiSurfaceBindingGeneration),
    BindingRequiresReconciliation(UiSurfaceBindingGeneration),
    BaselineReceiptUnavailable(UiSurfaceBindingGeneration),
    ReconciliationBasisMismatch,
    IdentityExhausted,
    SupersedingPredecessorUnavailable,
    CandidatePreparation(worth_ui_host_contract::UiHostSurfacePresentationDenial),
    AppearanceOutputUnavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedPresentationCompletionDenial {
    UnknownAttempt,
}

impl UiMountedPresentationAdmission {
    pub(super) fn new(
        prepared: super::super::retention::UiRetentionPreparedMountedFrame,
        attempt: UiMountedPresentationAttemptIdentity,
        deadline: UiPresentationDeadline,
        active: Rc<RefCell<BTreeSet<UiMountedPresentationAttemptIdentity>>>,
        candidates: super::coordinator::UiPreparedFrameCandidates,
    ) -> Self {
        let (frame, retention) = prepared.into_parts();
        Self {
            frame,
            candidates,
            retention,
            attempt,
            deadline,
            lease: UiPresentationAdmissionLease {
                active,
                attempt,
                release_on_drop: true,
            },
        }
    }

    pub fn attempt(&self) -> UiMountedPresentationAttemptIdentity {
        self.attempt
    }

    pub fn deadline(&self) -> UiPresentationDeadline {
        self.deadline
    }

    pub fn frame(&self) -> &UiPreparedMountedFrame {
        &self.frame
    }

    pub(crate) fn lower_appearance(
        &mut self,
        profile: Option<&worth_ui_host_contract::UiHostAppearanceProfileContract>,
    ) -> crate::runtime::appearance::UiAppearanceInspectionAttemptBatch {
        let requires_complete = self.candidates.requires_complete_appearance_projection();
        if requires_complete && self.frame.prepare_appearance_reconstruction().is_err() {
            return self.frame.deny_appearance_output();
        }
        let targets = self.frame.appearance_motion_targets(&[]);
        let motion = self.candidates.accepted_appearance_motion(&targets);
        self.frame
            .record_accepted_motion_commands_visited(motion.commands_visited());
        let result = self
            .frame
            .lower_appearance_with_motion(self.attempt, profile, motion);
        self.candidates.bind_appearance_opacity(&self.frame);
        result
    }

    pub(crate) fn lower_appearance_with_overlays(
        &mut self,
        profile: Option<&worth_ui_host_contract::UiHostAppearanceProfileContract>,
        overlays: &[crate::mounting::UiMountedAppearanceSurfaceOverlayInput],
    ) -> crate::runtime::appearance::UiAppearanceInspectionAttemptBatch {
        let requires_complete = self.candidates.requires_complete_appearance_projection();
        if requires_complete && self.frame.prepare_appearance_reconstruction().is_err() {
            return self.frame.deny_appearance_output();
        }
        let targets = self.frame.appearance_motion_targets(overlays);
        let motion = self.candidates.accepted_appearance_motion(&targets);
        self.frame
            .record_accepted_motion_commands_visited(motion.commands_visited());
        let result = self.frame.lower_appearance_with_motion_and_overlays(
            self.attempt,
            profile,
            motion,
            overlays,
        );
        self.candidates.bind_appearance_opacity(&self.frame);
        result
    }

    pub(crate) fn deny_appearance_output(
        &mut self,
    ) -> crate::runtime::appearance::UiAppearanceInspectionAttemptBatch {
        self.frame.deny_appearance_output()
    }

    pub(crate) fn appearance_output_available(&self) -> bool {
        self.frame.appearance_output_available()
    }

    pub(crate) fn reject_appearance_output(self) -> UiMountedPresentationAdmissionRejection {
        let attempt = self.attempt;
        let frame = self.frame;
        drop(self.retention);
        UiMountedPresentationAdmissionRejection::new_with_attempt(
            frame,
            UiMountedPresentationAdmissionDenial::AppearanceOutputUnavailable,
            attempt,
        )
    }

    pub fn into_attempt(self) -> UiMountedPresentationAttempt {
        UiMountedPresentationAttempt { admission: self }
    }
}

impl UiMountedPresentationAdmissionRejection {
    pub(crate) fn new(
        frame: UiPreparedMountedFrame,
        denial: UiMountedPresentationAdmissionDenial,
    ) -> Self {
        Self {
            denial,
            attempt: None,
            frame: Box::new(frame),
        }
    }

    fn new_with_attempt(
        frame: UiPreparedMountedFrame,
        denial: UiMountedPresentationAdmissionDenial,
        attempt: UiMountedPresentationAttemptIdentity,
    ) -> Self {
        Self {
            denial,
            attempt: Some(attempt),
            frame: Box::new(frame),
        }
    }

    pub fn denial(&self) -> UiMountedPresentationAdmissionDenial {
        self.denial
    }

    pub fn frame(&self) -> &UiPreparedMountedFrame {
        &self.frame
    }

    pub fn attempt(&self) -> Option<UiMountedPresentationAttemptIdentity> {
        self.attempt
    }

    pub fn into_frame(self) -> UiPreparedMountedFrame {
        *self.frame
    }
}

impl std::fmt::Debug for UiMountedPresentationAdmissionRejection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UiMountedPresentationAdmissionRejection")
            .field("denial", &self.denial)
            .finish_non_exhaustive()
    }
}

impl UiMountedPresentationAttempt {
    pub(super) fn into_parts(
        mut self,
    ) -> (
        UiPreparedMountedFrame,
        super::super::retention::UiMountedRetentionReservation,
        UiMountedPresentationAttemptIdentity,
        UiPresentationDeadline,
        super::coordinator::UiPreparedFrameCandidates,
    ) {
        self.admission.lease.release_on_drop = false;
        (
            self.admission.frame,
            self.admission.retention,
            self.admission.attempt,
            self.admission.deadline,
            self.admission.candidates,
        )
    }
}

impl Drop for UiPresentationAdmissionLease {
    fn drop(&mut self) {
        if self.release_on_drop {
            self.active.borrow_mut().remove(&self.attempt);
        }
    }
}

impl UiMountedPresentationInFlight {
    pub(super) fn from_state(
        state: &UiMountedPresentationInFlightState,
        cost: super::super::UiMountCostReport,
    ) -> Self {
        Self {
            frame: state.frame.canonical_core().frame(),
            attempt: state.attempt,
            deadline: state.deadline,
            pending_bindings: state
                .pending
                .iter()
                .map(|pending| pending.binding)
                .collect(),
            pending_progress_classes: state
                .pending
                .iter()
                .map(|pending| pending.token.progress_class())
                .collect(),
            semantic_requests: state.semantic_requests.clone().into_boxed_slice(),
            cost,
            retention: state.retention.identity(),
        }
    }

    pub fn attempt(&self) -> UiMountedPresentationAttemptIdentity {
        self.attempt
    }

    pub fn deadline(&self) -> UiPresentationDeadline {
        self.deadline
    }

    pub fn pending_bindings(
        &self,
    ) -> impl ExactSizeIterator<Item = UiSurfaceBindingGeneration> + '_ {
        self.pending_bindings.iter().copied()
    }

    pub fn cost_report(&self) -> super::super::UiMountCostReport {
        self.cost
    }

    pub fn awaits_progress_class(
        &self,
        class: worth_ui_host_contract::UiHostPresentationProgressClass,
    ) -> bool {
        self.pending_progress_classes.contains(&class)
    }

    pub fn semantic_requests(&self) -> &[worth_ui_query_binding::WorthUiPresentationRequestBasis] {
        &self.semantic_requests
    }

    pub(crate) const fn superseding_basis(&self) -> UiMountedSupersedingPresentationBasis {
        UiMountedSupersedingPresentationBasis {
            frame: self.frame,
            attempt: self.attempt,
            retention: self.retention,
        }
    }
}

impl UiMountedSupersedingPresentationBasis {
    pub(crate) const fn frame(self) -> worth_ui_host_contract::UiMountedFrameIdentity {
        self.frame
    }

    pub(crate) const fn attempt(self) -> UiMountedPresentationAttemptIdentity {
        self.attempt
    }

    pub(crate) const fn retention(
        self,
    ) -> super::super::retention::UiMountedRetentionReservationIdentity {
        self.retention
    }
}
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;
