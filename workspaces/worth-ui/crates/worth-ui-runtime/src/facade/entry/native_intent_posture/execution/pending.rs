use super::{
    finish, presentation_deadline, stopped, DetachedNativeIntentPostureInFlight,
    NativeIntentPostureAdmitted, NativeIntentPostureRejected, NativeIntentPostureTransfer,
    WorthUiNativeIntentPosturePublicationOutcome, WorthUiNativeIntentPosturePublicationRetry,
};
use crate::facade::entry::WorthUiActiveApplicationSession;

pub(in crate::facade::entry) enum DetachedNativeIntentPosturePending {
    InFlight(DetachedNativeIntentPostureInFlight),
    TextAtlasRetry(DetachedNativeIntentPostureRetry),
    ReconstructionRetry(DetachedNativeIntentPostureRetry),
}

pub(in crate::facade::entry) struct DetachedNativeIntentPostureRetry {
    session_identity: crate::facade::WorthUiActiveApplicationSessionIdentity,
    plan: crate::runtime::rebind::UiRebindPlan,
    reservation: crate::runtime::rebind::UiRebindReservation,
    transfer: NativeIntentPostureTransfer,
    frame: crate::mounting::UiPreparedMountedFrame,
    rejections: Box<[crate::mounting::UiMountedSurfacePresentationRejection]>,
}

impl DetachedNativeIntentPosturePending {
    pub(in crate::facade::entry) const fn session_identity(
        &self,
    ) -> crate::facade::WorthUiActiveApplicationSessionIdentity {
        match self {
            Self::InFlight(pending) => pending.session_identity(),
            Self::TextAtlasRetry(pending) | Self::ReconstructionRetry(pending) => {
                pending.session_identity
            }
        }
    }

    pub(in crate::facade::entry) fn matches_native_progress(
        &self,
        progress: &crate::native_platform::UiNativeApplicationPhysicalProgress,
    ) -> bool {
        match self {
            Self::InFlight(pending) => pending.matches_native_progress(progress),
            Self::TextAtlasRetry(_) => {
                progress.class() == worth_ui_host_native::UiNativePhysicalProgressClass::TextAtlas
            }
            Self::ReconstructionRetry(_) => false,
        }
    }

    pub(in crate::facade::entry) fn complete<'session>(
        self,
        session: &'session mut WorthUiActiveApplicationSession,
        now_tick: u64,
    ) -> WorthUiNativeIntentPosturePublicationOutcome<'session> {
        match self {
            Self::InFlight(pending) => pending.complete(session, now_tick),
            Self::TextAtlasRetry(pending) | Self::ReconstructionRetry(pending) => {
                pending.retry(session, now_tick)
            }
        }
    }

    pub(in crate::facade::entry) fn cancel(self, session: &mut WorthUiActiveApplicationSession) {
        match self {
            Self::InFlight(pending) => drop(pending.cancel(session)),
            Self::TextAtlasRetry(pending) | Self::ReconstructionRetry(pending) => drop(pending),
        }
    }
}

impl WorthUiNativeIntentPosturePublicationRetry<'_> {
    pub(in crate::facade::entry) fn into_stop(
        mut self,
    ) -> super::super::WorthUiNativeIntentPosturePublicationStop {
        let state = self
            .state
            .take()
            .expect("live posture retry owns its state");
        super::super::WorthUiNativeIntentPosturePublicationStop::host_rejected(
            state
                .rejections
                .iter()
                .map(|rejection| rejection.denial())
                .collect(),
        )
    }

    pub(in crate::facade::entry) fn detach_for_native(
        mut self,
    ) -> DetachedNativeIntentPostureRetry {
        let state = self
            .state
            .take()
            .expect("live posture retry owns its state");
        let NativeIntentPostureRejected {
            admitted,
            frame,
            rejections,
        } = *state;
        let NativeIntentPostureAdmitted {
            session,
            plan,
            reservation,
            transfer,
        } = admitted;
        DetachedNativeIntentPostureRetry {
            session_identity: session.session_identity(),
            plan,
            reservation,
            transfer,
            frame,
            rejections,
        }
    }
}

impl DetachedNativeIntentPostureRetry {
    fn retry<'session>(
        mut self,
        session: &'session mut WorthUiActiveApplicationSession,
        now_tick: u64,
    ) -> WorthUiNativeIntentPosturePublicationOutcome<'session> {
        if let Err(denial) = self.transfer.observation.validate(session, &self.frame) {
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
        let outcome = session
            .present_prepared_observed_frame(
                self.frame,
                &self.transfer.observation,
                None,
                deadline,
                now_tick,
            )
            .expect("exclusive detached retry preserves the validated observation/frame bond");
        drop(self.rejections);
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
