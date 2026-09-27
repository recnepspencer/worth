//! The presentation a host rejected before effects while it waits for
//! readiness it schedules itself: a timed wake after an external timeout, or
//! visibility after occlusion.

use super::super::WorthUiNativeApplicationShell;

/// A frame the host rejected before effects while it waits for readiness it
/// schedules itself. The rejection ends no presentation debt, so that
/// readiness owes one successor presentation until the host, answering any
/// owner's surface presentation, drops its wake.
pub(in super::super) struct UiNativePresentationRetry {
    owed_at_answer: Option<u64>,
}

impl UiNativePresentationRetry {
    pub(in super::super) const fn new() -> Self {
        Self {
            owed_at_answer: None,
        }
    }

    /// Lands `outcome`, taking `answers`, the host's count of retry-wake
    /// clearing answers, from before its attempt. The host judges an attempt
    /// by all its answers at once, so a count from before it is the one a debt
    /// holds at; a frame rejected only by timeout or occlusion adds none, so
    /// the count after agrees. Only a frame the runtime refused before
    /// reaching the host leaves the debt unchanged.
    pub(in super::super) fn land_outcome(
        &mut self,
        outcome: &crate::mounting::UiMountedFrameOutcome,
        answers: u64,
    ) {
        use crate::mounting::UiMountedFrameOutcome as Outcome;
        match outcome {
            Outcome::AdmissionDenied(_) | Outcome::RetentionDenied(_) => {}
            Outcome::RejectedBeforeEffects(_) => {
                self.owed_at_answer =
                    WorthUiNativeApplicationShell::frame_presentation_awaits_host_readiness(
                        outcome,
                    )
                    .then_some(answers);
            }
            Outcome::Published(_)
            | Outcome::Unchanged(_)
            | Outcome::Reconciled(_)
            | Outcome::InFlight(_)
            | Outcome::Superseded(_)
            | Outcome::PresentationIndeterminate(_)
            | Outcome::CompletionDenied(_) => self.owed_at_answer = None,
        }
    }
}

impl WorthUiNativeApplicationShell {
    /// Whether the host rejected the latest frame before effects and the
    /// readiness it schedules still owes that frame's successor.
    pub fn native_presentation_retry_pending(&self) -> bool {
        self.presentation_retry.owed_at_answer == Some(self.host_retry_wake_clearing_answers())
    }

    pub(in super::super) fn host_retry_wake_clearing_answers(&self) -> u64 {
        self.session
            .host_session
            .effect_port()
            .authority()
            .retry_wake_clearing_answers()
    }

    /// Presents again a frame the host rejected before effects, landing the
    /// outcome as every shell presentation does.
    pub(crate) fn retry_rejected_frame_presentation(
        &mut self,
        rejected: crate::mounting::UiMountedRejectedFrame,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now_tick: u64,
    ) -> crate::mounting::UiMountedFrameOutcome {
        let answers = self.host_retry_wake_clearing_answers();
        let outcome = self.session.present_prepared_mounted_frame_internal(
            rejected.into_frame(),
            deadline,
            now_tick,
        );
        self.land_frame_outcome(&outcome, answers);
        outcome
    }

    /// Whether the host rejected `outcome` before effects only while it waits
    /// for readiness it schedules itself: a timed wake after an external
    /// timeout, or visibility after occlusion. Presenting again at that
    /// readiness, with the rejected frame's purpose, is the lawful retry.
    pub fn frame_presentation_awaits_host_readiness(
        outcome: &crate::mounting::UiMountedFrameOutcome,
    ) -> bool {
        use worth_ui_host_contract::UiHostSurfacePresentationDenial as Denial;
        matches!(
            outcome,
            crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejected)
                if super::rejected_only_by(rejected, |denial| {
                    matches!(denial, Denial::ExternalTimeout | Denial::SurfaceOccluded)
                })
        )
    }
}

#[cfg(test)]
#[path = "host_retry_tests.rs"]
mod tests;
