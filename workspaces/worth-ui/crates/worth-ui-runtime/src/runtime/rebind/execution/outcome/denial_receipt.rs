use super::{
    UiRebindDenialCause, UiRebindDenialReceipt, UiRebindOutcome, UiRebindStoppedPhase,
    UiRebindValidNextAction,
};
use crate::runtime::rebind::execution::state::UiRebindReservation;

impl<'session> UiRebindDenialReceipt<'session> {
    pub(crate) fn prepared_basis_changed(retry: super::super::UiPreparedRebind<'session>) -> Self {
        Self {
            predecessor_remains_current: true,
            stopped_phase: UiRebindStoppedPhase::EffectAdmission,
            cause: UiRebindDenialCause::MountedPresentation(
                crate::mounting::UiMountedPresentationAdmissionDenial::PreparedFrameBasisChanged,
            ),
            host_rejections: Box::new([]),
            valid_next_action: UiRebindValidNextAction::RetryPrepared,
            retry: Some(Box::new(retry)),
        }
    }

    pub(crate) fn capacity(
        denial: super::super::UiRebindReservationDenial,
        retry: super::super::UiPreparedRebind<'session>,
    ) -> Self {
        Self {
            predecessor_remains_current: true,
            stopped_phase: UiRebindStoppedPhase::EffectAdmission,
            cause: UiRebindDenialCause::RuntimeCapacity(denial),
            host_rejections: Box::new([]),
            valid_next_action: UiRebindValidNextAction::RetryPrepared,
            retry: Some(Box::new(retry)),
        }
    }

    pub(super) fn retry(
        plan: crate::runtime::rebind::UiRebindPlan,
        registration: UiRebindReservation,
        kind: super::super::preparation::UiPreparedRebindKind<'session>,
        stopped_phase: UiRebindStoppedPhase,
        cause: UiRebindDenialCause,
    ) -> Self {
        Self {
            predecessor_remains_current: true,
            stopped_phase,
            cause,
            host_rejections: Box::new([]),
            valid_next_action: UiRebindValidNextAction::RetryPrepared,
            retry: Some(Box::new(super::super::UiPreparedRebind {
                plan,
                reservation: registration,
                kind,
            })),
        }
    }

    pub(super) fn retry_host(
        plan: crate::runtime::rebind::UiRebindPlan,
        registration: UiRebindReservation,
        kind: super::super::preparation::UiPreparedRebindKind<'session>,
        rejections: Box<[crate::mounting::UiMountedSurfacePresentationRejection]>,
    ) -> Self {
        Self {
            predecessor_remains_current: true,
            stopped_phase: UiRebindStoppedPhase::HostPresentation,
            cause: super::UiRebindDenialCause::HostRejectedBeforeEffects,
            host_rejections: rejections,
            valid_next_action: UiRebindValidNextAction::RetryPrepared,
            retry: Some(Box::new(super::super::UiPreparedRebind {
                plan,
                reservation: registration,
                kind,
            })),
        }
    }

    pub const fn predecessor_remains_current(&self) -> bool {
        self.predecessor_remains_current
    }

    pub const fn stopped_phase(&self) -> UiRebindStoppedPhase {
        self.stopped_phase
    }

    pub const fn cause(&self) -> UiRebindDenialCause {
        self.cause
    }

    pub fn host_rejections(&self) -> &[crate::mounting::UiMountedSurfacePresentationRejection] {
        &self.host_rejections
    }

    pub const fn valid_next_action(&self) -> UiRebindValidNextAction {
        self.valid_next_action
    }

    pub fn retry_at(mut self, now_tick: u64) -> UiRebindOutcome<'session> {
        match self.retry.take() {
            Some(retry) => retry.execute(now_tick),
            None => UiRebindOutcome::RejectedBeforeEffects(self),
        }
    }

    pub(crate) fn detach_retry_for_native(self) -> Result<super::UiDetachedRebindRetry, Self> {
        super::UiDetachedRebindRetry::from_denial(self)
    }
}
