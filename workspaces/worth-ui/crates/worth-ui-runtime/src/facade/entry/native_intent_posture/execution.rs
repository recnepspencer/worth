use super::{
    stopped, NativeIntentPostureTransfer, WorthUiNativeIntentPosturePublicationCompletion,
    WorthUiNativeIntentPosturePublicationOutcome, WorthUiNativeIntentPosturePublicationRecovery,
    WorthUiNativeIntentPosturePublicationRetry,
};
use crate::facade::entry::WorthUiActiveApplicationSession;

mod pending;
mod recovery;
mod retry;
pub(in crate::facade::entry) use pending::DetachedNativeIntentPosturePending;

pub(super) struct PreparedNativeIntentPostureRebind<'session> {
    pub(super) session: &'session mut WorthUiActiveApplicationSession,
    pub(super) plan: crate::runtime::rebind::UiRebindPlan,
    pub(super) reservation: crate::runtime::rebind::UiRebindReservation,
    pub(super) frame: crate::mounting::UiPreparedMountedFrame,
    pub(super) transfer: NativeIntentPostureTransfer,
    pub(super) now_tick: u64,
}

struct NativeIntentPostureAdmitted<'session> {
    session: &'session mut WorthUiActiveApplicationSession,
    plan: crate::runtime::rebind::UiRebindPlan,
    reservation: crate::runtime::rebind::UiRebindReservation,
    transfer: NativeIntentPostureTransfer,
}

pub(super) struct NativeIntentPostureInFlight<'session> {
    admitted: NativeIntentPostureAdmitted<'session>,
    mounted: crate::mounting::UiMountedPresentationInFlight,
}

pub(in crate::facade::entry) struct DetachedNativeIntentPostureInFlight {
    session_identity: crate::facade::WorthUiActiveApplicationSessionIdentity,
    plan: crate::runtime::rebind::UiRebindPlan,
    reservation: crate::runtime::rebind::UiRebindReservation,
    transfer: NativeIntentPostureTransfer,
    mounted: crate::mounting::UiMountedPresentationInFlight,
}

pub(super) struct NativeIntentPostureIndeterminate<'session> {
    admitted: NativeIntentPostureAdmitted<'session>,
    frame: crate::mounting::UiMountedIndeterminateFrame,
}

impl<'session> PreparedNativeIntentPostureRebind<'session> {
    pub(super) fn execute(mut self) -> WorthUiNativeIntentPosturePublicationOutcome<'session> {
        if let Err(denial) = self
            .transfer
            .observation
            .validate(self.session, &self.frame)
        {
            return stopped(
                crate::runtime::intent_execution::UiIntentConsequenceStopReason::Preparation(
                    Box::new(denial),
                ),
            );
        }
        if let Err(denial) = self.reservation.begin_effecting() {
            return stopped(
                crate::runtime::intent_execution::UiIntentConsequenceStopReason::RebindAdmission(
                    denial,
                ),
            );
        }
        let deadline = presentation_deadline(&self.plan);
        let Self {
            session,
            plan,
            reservation,
            frame,
            transfer,
            now_tick,
        } = self;
        let outcome = session
            .present_prepared_observed_frame(frame, &transfer.observation, None, deadline, now_tick)
            .expect("exclusive admission preserves the validated observation/frame bond");
        finish(
            NativeIntentPostureAdmitted {
                session,
                plan,
                reservation,
                transfer,
            },
            outcome,
        )
    }
}

impl WorthUiNativeIntentPosturePublicationCompletion<'_> {
    pub fn attempt(&self) -> worth_ui_host_contract::UiMountedPresentationAttemptIdentity {
        self.state().mounted.attempt()
    }

    pub fn deadline(&self) -> worth_ui_host_contract::UiPresentationDeadline {
        self.state().mounted.deadline()
    }
}

impl<'session> WorthUiNativeIntentPosturePublicationCompletion<'session> {
    pub fn complete(
        mut self,
        now_tick: u64,
    ) -> WorthUiNativeIntentPosturePublicationOutcome<'session> {
        let state = self.take_state();
        let outcome =
            match state.admitted.session.complete_prepared_observed_frame(
                state.mounted,
                &state.admitted.transfer.observation,
                now_tick,
            ) {
                Ok(outcome) => outcome,
                Err(denial) => return stopped(
                    crate::runtime::intent_execution::UiIntentConsequenceStopReason::Preparation(
                        Box::new(denial),
                    ),
                ),
            };
        finish(state.admitted, outcome)
    }

    pub fn dispose(mut self) -> WorthUiNativeIntentPosturePublicationOutcome<'session> {
        let state = self.take_state();
        let outcome = state
            .admitted
            .session
            .cancel_mounted_presentation(state.mounted);
        finish(state.admitted, outcome)
    }

    pub(in crate::facade::entry) fn detach_for_native(
        mut self,
    ) -> DetachedNativeIntentPostureInFlight {
        let state = self.take_state();
        let NativeIntentPostureInFlight { admitted, mounted } = *state;
        let NativeIntentPostureAdmitted {
            session,
            plan,
            reservation,
            transfer,
        } = admitted;
        DetachedNativeIntentPostureInFlight {
            session_identity: session.session_identity(),
            plan,
            reservation,
            transfer,
            mounted,
        }
    }

    fn state(&self) -> &NativeIntentPostureInFlight<'session> {
        self.state
            .as_deref()
            .expect("live posture completion owns its state")
    }

    fn take_state(&mut self) -> Box<NativeIntentPostureInFlight<'session>> {
        self.state
            .take()
            .expect("live posture completion owns its state")
    }
}

pub(super) struct NativeIntentPostureRejected<'session> {
    admitted: NativeIntentPostureAdmitted<'session>,
    frame: crate::mounting::UiPreparedMountedFrame,
    rejections: Box<[crate::mounting::UiMountedSurfacePresentationRejection]>,
}

impl DetachedNativeIntentPostureInFlight {
    pub(in crate::facade::entry) const fn session_identity(
        &self,
    ) -> crate::facade::WorthUiActiveApplicationSessionIdentity {
        self.session_identity
    }

    pub(in crate::facade::entry) fn matches_native_progress(
        &self,
        progress: &crate::native_platform::UiNativeApplicationPhysicalProgress,
    ) -> bool {
        let class = match progress.class() {
            worth_ui_host_native::UiNativePhysicalProgressClass::Presentation => {
                worth_ui_host_contract::UiHostPresentationProgressClass::PhysicalSurface
            }
            worth_ui_host_native::UiNativePhysicalProgressClass::TextAtlas => {
                worth_ui_host_contract::UiHostPresentationProgressClass::TextAtlas
            }
            worth_ui_host_native::UiNativePhysicalProgressClass::PresentationRecovery => {
                return false;
            }
        };
        self.mounted.awaits_progress_class(class)
            && progress.presentation().is_none_or(|presentation| {
                presentation.attempt() == self.mounted.attempt()
                    && self
                        .mounted
                        .pending_bindings()
                        .any(|binding| binding == presentation.binding())
            })
    }

    pub(in crate::facade::entry) fn complete<'session>(
        self,
        session: &'session mut WorthUiActiveApplicationSession,
        now_tick: u64,
    ) -> WorthUiNativeIntentPosturePublicationOutcome<'session> {
        let outcome =
            match session.complete_prepared_observed_frame(
                self.mounted,
                &self.transfer.observation,
                now_tick,
            ) {
                Ok(outcome) => outcome,
                Err(denial) => return stopped(
                    crate::runtime::intent_execution::UiIntentConsequenceStopReason::Preparation(
                        Box::new(denial),
                    ),
                ),
            };
        finish(
            NativeIntentPostureAdmitted {
                session,
                plan: self.plan,
                reservation: self.reservation,
                transfer: self.transfer,
            },
            outcome,
        )
    }

    pub(in crate::facade::entry) fn cancel<'session>(
        self,
        session: &'session mut WorthUiActiveApplicationSession,
    ) -> WorthUiNativeIntentPosturePublicationOutcome<'session> {
        let outcome = session.cancel_mounted_presentation(self.mounted);
        finish(
            NativeIntentPostureAdmitted {
                session,
                plan: self.plan,
                reservation: self.reservation,
                transfer: self.transfer,
            },
            outcome,
        )
    }
}

impl Drop for WorthUiNativeIntentPosturePublicationCompletion<'_> {
    fn drop(&mut self) {
        let Some(state) = self.state.take() else {
            return;
        };
        let outcome = state
            .admitted
            .session
            .cancel_mounted_presentation(state.mounted);
        drop(finish(state.admitted, outcome));
    }
}

fn finish<'session>(
    mut admitted: NativeIntentPostureAdmitted<'session>,
    outcome: crate::mounting::UiMountedFrameOutcome,
) -> WorthUiNativeIntentPosturePublicationOutcome<'session> {
    match outcome {
        crate::mounting::UiMountedFrameOutcome::Published(mounted) => publish(admitted, mounted),
        crate::mounting::UiMountedFrameOutcome::InFlight(mounted) => {
            WorthUiNativeIntentPosturePublicationOutcome::InFlight(
                WorthUiNativeIntentPosturePublicationCompletion {
                    state: Some(Box::new(NativeIntentPostureInFlight { admitted, mounted })),
                },
            )
        }
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejected) => {
            admitted
                .reservation
                .return_to_pending()
                .expect("host rejection returns posture reservation to pending");
            let (frame, rejections) = rejected.into_parts();
            WorthUiNativeIntentPosturePublicationOutcome::RejectedBeforeEffects(
                WorthUiNativeIntentPosturePublicationRetry {
                    state: Some(Box::new(NativeIntentPostureRejected {
                        admitted,
                        frame,
                        rejections,
                    })),
                },
            )
        }
        crate::mounting::UiMountedFrameOutcome::RetentionDenied(rejected) => stopped(
            crate::runtime::intent_execution::UiIntentConsequenceStopReason::MountedRetention(
                rejected.denial(),
            ),
        ),
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(rejected) => stopped(
            crate::runtime::intent_execution::UiIntentConsequenceStopReason::MountedPresentation(
                rejected.denial(),
            ),
        ),
        crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(frame) => {
            admitted
                .reservation
                .retain_recovery()
                .expect("pre-effect admission reserved recovery capacity");
            WorthUiNativeIntentPosturePublicationOutcome::Indeterminate(
                WorthUiNativeIntentPosturePublicationRecovery {
                    state: Some(Box::new(NativeIntentPostureIndeterminate {
                        admitted,
                        frame,
                    })),
                },
            )
        }
        crate::mounting::UiMountedFrameOutcome::Superseded(_) => {
            unreachable!("ordinary native intent cannot overlap a superseding frame")
        }
        crate::mounting::UiMountedFrameOutcome::CompletionDenied(_) => {
            panic!("exact posture completion authority became unknown")
        }
        crate::mounting::UiMountedFrameOutcome::Unchanged(_)
        | crate::mounting::UiMountedFrameOutcome::Reconciled(_) => {
            unreachable!("fresh posture content always presents a fresh frame")
        }
    }
}

fn publish<'session>(
    mut admitted: NativeIntentPostureAdmitted<'session>,
    mounted: crate::mounting::UiMountedFramePublicationReceipt,
) -> WorthUiNativeIntentPosturePublicationOutcome<'session> {
    assert!(matches!(
        admitted.plan.take_semantic_proof(),
        crate::runtime::rebind::UiRebindSemanticProof::NonSource
    ));
    admitted
        .session
        .commit_consequence_observation(admitted.transfer.observation);
    admitted
        .session
        .intent_postures
        .commit(admitted.transfer.posture);
    let generation = admitted.plan.basis().candidate_generation().clone();
    match crate::runtime::rebind::UiRebindReceipt::content(
        admitted.plan,
        admitted.reservation,
        generation,
        mounted,
    ) {
        Ok(receipt) => WorthUiNativeIntentPosturePublicationOutcome::Published(receipt),
        Err(defect) => WorthUiNativeIntentPosturePublicationOutcome::InternalDefect(defect),
    }
}

fn presentation_deadline(
    plan: &crate::runtime::rebind::UiRebindPlan,
) -> worth_ui_host_contract::UiPresentationDeadline {
    let tick = match plan.execution_policy().deadline() {
        crate::runtime::rebind::UiRebindDeadlinePolicy::NoDeadline => u64::MAX,
        crate::runtime::rebind::UiRebindDeadlinePolicy::At(deadline) => deadline.tick(),
    };
    worth_ui_host_contract::UiPresentationDeadline::at_tick(tick)
}
