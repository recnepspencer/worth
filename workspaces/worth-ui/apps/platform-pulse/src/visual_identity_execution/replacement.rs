use std::time::Instant;
use worth_ui::facade::app::WorthUiNativeApplicationShell;

use super::{
    capture_restart::PlatformPulseAwaitingCaptureBudget, replacement_frame_deadline,
    PlatformPulseVisualExecutionDenial, PlatformPulseVisualIdentityJourney,
    PlatformPulseVisualIdentityState,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) enum PlatformPulseReplacementPosture {
    #[default]
    Current,
    Pending,
}

impl PlatformPulseReplacementPosture {
    pub(super) fn note(&mut self) {
        *self = Self::Pending;
    }
    pub(super) const fn is_pending(self) -> bool {
        matches!(self, Self::Pending)
    }
}

impl PlatformPulseVisualIdentityJourney {
    pub(crate) fn prepare_source_rebind(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        tick: u64,
        now: Instant,
    ) -> Result<bool, PlatformPulseVisualExecutionDenial> {
        if self.queued_rebind.is_some() {
            return Ok(false);
        }
        match &self.state {
            PlatformPulseVisualIdentityState::ComparisonReady(retained) => {
                if super::frame_affinity::snapshot_matches_current_mounted_frame(
                    &retained.snapshot,
                    shell,
                )? {
                    return Ok(true);
                }
                self.refresh_after_presentation_replacement(shell, tick, now)?;
                Ok(false)
            }
            PlatformPulseVisualIdentityState::Retired => Ok(true),
            _ => Ok(false),
        }
    }

    pub(crate) fn refresh_after_presentation_replacement(
        &mut self,
        _shell: &mut WorthUiNativeApplicationShell,
        _tick: u64,
        now: Instant,
    ) -> Result<(), PlatformPulseVisualExecutionDenial> {
        let deadline = replacement_frame_deadline(now)?;
        let state = self.begin_transition()?;
        let next = match state {
            PlatformPulseVisualIdentityState::Settling { .. } => {
                PlatformPulseVisualIdentityState::AwaitingCapture {
                    budget: PlatformPulseAwaitingCaptureBudget::fresh(deadline),
                }
            }
            PlatformPulseVisualIdentityState::Retired => {
                PlatformPulseVisualIdentityState::AwaitingRebase {
                    budget: PlatformPulseAwaitingCaptureBudget::fresh(deadline),
                }
            }
            PlatformPulseVisualIdentityState::ComparisonReady(predecessor) => {
                PlatformPulseVisualIdentityState::AwaitingRefresh {
                    predecessor,
                    budget: PlatformPulseAwaitingCaptureBudget::fresh(deadline),
                }
            }
            PlatformPulseVisualIdentityState::AwaitingComparison {
                predecessor,
                rebind,
                budget,
            } => {
                drop(rebind);
                PlatformPulseVisualIdentityState::AwaitingRefresh {
                    predecessor,
                    budget,
                }
            }
            PlatformPulseVisualIdentityState::OverlayVisible(mut overlay) => {
                overlay.replacement.note();
                PlatformPulseVisualIdentityState::OverlayVisible(overlay)
            }
            PlatformPulseVisualIdentityState::Comparing(mut comparison) => {
                comparison.note_presentation_replacement();
                PlatformPulseVisualIdentityState::Comparing(comparison)
            }
            PlatformPulseVisualIdentityState::Capturing(mut capture) => {
                capture.replacement.note();
                PlatformPulseVisualIdentityState::Capturing(capture)
            }
            PlatformPulseVisualIdentityState::Rebasing(mut capture) => {
                capture.replacement.note();
                PlatformPulseVisualIdentityState::Rebasing(capture)
            }
            PlatformPulseVisualIdentityState::Refreshing(mut refresh) => {
                refresh.capture.replacement.note();
                PlatformPulseVisualIdentityState::Refreshing(refresh)
            }
            // Awaiting work retains its original budget; another replacement
            // neither extends the host deadline nor waits for a Portal to close.
            state => state,
        };
        self.state = next;
        self.schedule_current_wake();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::PlatformPulseReplacementPosture;

    #[test]
    fn in_flight_replacement_is_retained_for_a_fresh_successor_capture() {
        let mut posture = PlatformPulseReplacementPosture::default();
        posture.note();
        posture.note();
        assert!(posture.is_pending());
    }
}
