use super::UiNativeHostState;

mod owner_poll;
mod ready_token;
mod settlement;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativePresentationPhysicalProgress {
    NoProgress,
    Pending,
    Completed { duplicate_observation: bool },
    Superseded,
    Rejected,
    IndeterminateRecoveryScheduled,
    RecoveryCompleted,
}

impl UiNativeHostState {
    pub(super) fn progress_pending_presentation(
        &mut self,
        identity: crate::native::physical_work_signal::UiNativePhysicalPresentationIdentity,
    ) -> UiNativePresentationPhysicalProgress {
        let Ok(ready) = self.acquire_ready_presentation(identity) else {
            return UiNativePresentationPhysicalProgress::NoProgress;
        };
        let Ok(polled) = self.poll_presentation_owner(identity, ready) else {
            return UiNativePresentationPhysicalProgress::NoProgress;
        };
        let signal_settlement = self.physical_signal.reconcile(polled.observation);
        let progress = self.settle_presentation_signal(identity, polled.ready, signal_settlement);
        if let Some(crate::native::UiNativePresentationOwners { device, .. }) =
            self.presentation_owners.as_mut()
        {
            let _ = crate::native::lifecycle::collect_settled_device_generations(
                device,
                &mut self.resources,
            );
        }
        progress
    }
}
