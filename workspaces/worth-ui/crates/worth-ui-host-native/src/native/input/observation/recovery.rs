use super::{UiNativeInputObservationState, UiNativeInputObservationStop};
use worth_ui_host_contract::UiHostObservationRetentionDenial;

impl UiNativeInputObservationStop {
    pub(super) fn permits_retention_recovery(self) -> bool {
        matches!(
            self,
            Self::Retention(UiHostObservationRetentionDenial::Capacity(_))
        )
    }
}

/// Issued only after every retained predecessor report has been drained.
/// The client must cancel its input interactions before acknowledging it.
#[must_use]
pub struct UiNativeInputRecoveryGrant {
    host_session: u64,
    next_sequence: u64,
}

#[must_use]
pub struct UiNativeInputRecoveryAcknowledgement(UiNativeInputRecoveryGrant);

impl UiNativeInputRecoveryGrant {
    pub const fn host_session(&self) -> u64 {
        self.host_session
    }

    pub fn acknowledge_cancellation(self) -> UiNativeInputRecoveryAcknowledgement {
        UiNativeInputRecoveryAcknowledgement(self)
    }
}

impl UiNativeInputObservationState {
    pub(crate) fn begin_retention_recovery(&self) -> Option<UiNativeInputRecoveryGrant> {
        if !self
            .terminal_stop
            .is_some_and(UiNativeInputObservationStop::permits_retention_recovery)
            || self.has_retained_observations()
        {
            return None;
        }
        Some(UiNativeInputRecoveryGrant {
            host_session: self.active_host_session?,
            next_sequence: self.next_sequence?,
        })
    }

    pub(crate) fn complete_retention_recovery(
        &mut self,
        acknowledgement: UiNativeInputRecoveryAcknowledgement,
    ) -> bool {
        let Some(current) = self.begin_retention_recovery() else {
            return false;
        };
        let acknowledged = acknowledgement.0;
        if current.host_session != acknowledged.host_session
            || current.next_sequence != acknowledged.next_sequence
        {
            return false;
        }
        if self.pointer.end_capture().is_err() {
            self.terminal_stop = Some(UiNativeInputObservationStop::PointerCaptureEpochExhausted);
            self.record_stop(UiNativeInputObservationStop::PointerCaptureEpochExhausted);
            return false;
        }
        self.ime_composition_active = false;
        // Keep sequence and completed presentation authority. Rejected input
        // never acquired a sequence; the retained prefix was drained in order.
        // The stop history remains evidence of the cancelled input interval.
        self.terminal_stop = None;
        // A resize completed while input was stopped may still owe its profile
        // observation. Never publish a profile whose frame has not completed.
        self.emit_completed_profile_transition();
        self.terminal_stop.is_none()
    }
}
