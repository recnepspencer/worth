//! The presentation a host rejected before effects while it waits for
//! readiness it schedules itself: a timed wake after an external timeout, or
//! visibility after occlusion.

use super::super::WorthUiNativeApplicationShell;

/// A frame the host rejected before effects while it waits for readiness it
/// schedules itself. The rejection ends no presentation debt, so that
/// readiness owes one successor presentation until any surface presentation,
/// from whichever owner, begins effects and the host drops its wake.
pub(in super::super) struct UiNativePresentationRetry {
    owed_at_effect_round: Option<u64>,
}

impl UiNativePresentationRetry {
    pub(in super::super) const fn new() -> Self {
        Self {
            owed_at_effect_round: None,
        }
    }

    /// Lands `outcome` at the host's current `effect_rounds`. Only a frame the
    /// runtime refused before reaching the host leaves the debt unchanged.
    pub(in super::super) fn land_outcome(
        &mut self,
        outcome: &crate::mounting::UiMountedFrameOutcome,
        effect_rounds: u64,
    ) {
        use crate::mounting::UiMountedFrameOutcome as Outcome;
        match outcome {
            Outcome::AdmissionDenied(_) | Outcome::RetentionDenied(_) => {}
            Outcome::RejectedBeforeEffects(_) => {
                self.owed_at_effect_round =
                    WorthUiNativeApplicationShell::frame_presentation_awaits_host_readiness(
                        outcome,
                    )
                    .then_some(effect_rounds);
            }
            Outcome::Published(_)
            | Outcome::Unchanged(_)
            | Outcome::Reconciled(_)
            | Outcome::InFlight(_)
            | Outcome::Superseded(_)
            | Outcome::PresentationIndeterminate(_)
            | Outcome::CompletionDenied(_) => self.owed_at_effect_round = None,
        }
    }
}

impl WorthUiNativeApplicationShell {
    /// Whether the host rejected the latest frame before effects and the
    /// readiness it schedules still owes that frame's successor.
    pub fn native_presentation_retry_pending(&self) -> bool {
        self.presentation_retry.owed_at_effect_round == Some(self.host_effect_rounds())
    }

    pub(in super::super) fn host_effect_rounds(&self) -> u64 {
        self.session
            .host_session
            .effect_port()
            .authority()
            .surface_effect_rounds()
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
