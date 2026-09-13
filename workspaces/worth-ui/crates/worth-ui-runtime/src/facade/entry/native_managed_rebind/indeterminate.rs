use super::{
    WorthUiNativeManagedRebindDenial, WorthUiNativeManagedRebindProgress,
    WorthUiNativePendingManagedRebind,
};
use crate::facade::entry::native_application_shell::{
    WorthUiNativePhysicalPresentationRecovery, WorthUiNativePresentationRecoveryDenial,
};
use crate::runtime::rebind::UiDetachedRebindRecovery;

impl super::WorthUiNativeApplicationShell {
    pub(super) fn progress_indeterminate_rebind(
        &mut self,
        recovery: UiDetachedRebindRecovery,
        frame: crate::mounting::UiMountedIndeterminateFrame,
        progress: &crate::native_platform::UiNativeApplicationPhysicalProgress,
    ) -> Result<WorthUiNativeManagedRebindProgress, WorthUiNativeManagedRebindDenial> {
        self.validate_rebind_recovery(&recovery)?;
        self.managed_rebind_completion_tick = self.managed_rebind_completion_tick.saturating_add(1);
        match self.progress_indeterminate_presentation_recovery(
            frame,
            progress,
            u64::MAX,
            self.managed_rebind_completion_tick,
        ) {
            WorthUiNativePhysicalPresentationRecovery::Awaiting(frame) => {
                self.pending_managed_rebind =
                    Some(WorthUiNativePendingManagedRebind::Indeterminate { recovery, frame });
                Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress)
            }
            WorthUiNativePhysicalPresentationRecovery::Blocked { frame, denial } => {
                self.pending_managed_rebind =
                    Some(WorthUiNativePendingManagedRebind::Indeterminate { recovery, frame });
                Ok(WorthUiNativeManagedRebindProgress::RecoveryBlocked(denial))
            }
            WorthUiNativePhysicalPresentationRecovery::Recovered(outcome) => {
                Ok(self.finish_rebind_recovery(recovery, outcome))
            }
        }
    }

    pub(super) fn progress_rebind_recovery_reconstruction(
        &mut self,
        recovery: UiDetachedRebindRecovery,
        in_flight: crate::mounting::UiMountedPresentationInFlight,
        progress: &crate::native_platform::UiNativeApplicationPhysicalProgress,
    ) -> Result<WorthUiNativeManagedRebindProgress, WorthUiNativeManagedRebindDenial> {
        self.validate_rebind_recovery(&recovery)?;
        if !super::predecessor_reconstruction::reconstruction_matches_progress(&in_flight, progress)
        {
            self.pending_managed_rebind =
                Some(WorthUiNativePendingManagedRebind::RecoveryReconstruction {
                    recovery,
                    in_flight,
                });
            return Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress);
        }
        self.managed_rebind_completion_tick = self.managed_rebind_completion_tick.saturating_add(1);
        let outcome = self
            .session
            .complete_mounted_presentation(in_flight, self.managed_rebind_completion_tick);
        Ok(self.finish_rebind_recovery(recovery, outcome))
    }

    pub(super) fn progress_deferred_rebind_recovery(
        &mut self,
        recovery: UiDetachedRebindRecovery,
    ) -> Result<WorthUiNativeManagedRebindProgress, WorthUiNativeManagedRebindDenial> {
        self.validate_rebind_recovery(&recovery)?;
        self.managed_rebind_completion_tick = self.managed_rebind_completion_tick.saturating_add(1);
        match self.reconstruct_current_presentation(u64::MAX, self.managed_rebind_completion_tick) {
            Ok(outcome) => Ok(self.finish_rebind_recovery(recovery, outcome)),
            Err(()) => {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::RecoveryReconstructionDeferred(recovery),
                );
                Ok(WorthUiNativeManagedRebindProgress::RecoveryBlocked(
                    WorthUiNativePresentationRecoveryDenial::CurrentPresentationUnavailable,
                ))
            }
        }
    }

    fn finish_rebind_recovery(
        &mut self,
        recovery: UiDetachedRebindRecovery,
        outcome: crate::mounting::UiMountedFrameOutcome,
    ) -> WorthUiNativeManagedRebindProgress {
        match outcome {
            crate::mounting::UiMountedFrameOutcome::Reconciled(mounted) => {
                self.settle_native_mounted_reconciliation(&mounted);
                WorthUiNativeManagedRebindProgress::RebindRecovered(recovery.settle(mounted))
            }
            crate::mounting::UiMountedFrameOutcome::InFlight(in_flight) => {
                self.pending_managed_rebind =
                    Some(WorthUiNativePendingManagedRebind::RecoveryReconstruction {
                        recovery,
                        in_flight,
                    });
                WorthUiNativeManagedRebindProgress::AwaitingProgress
            }
            crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(frame) => {
                self.pending_managed_rebind =
                    Some(WorthUiNativePendingManagedRebind::Indeterminate { recovery, frame });
                WorthUiNativeManagedRebindProgress::AwaitingProgress
            }
            _ => {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::RecoveryReconstructionDeferred(recovery),
                );
                WorthUiNativeManagedRebindProgress::RecoveryBlocked(
                    WorthUiNativePresentationRecoveryDenial::FramePresentationUnavailable,
                )
            }
        }
    }

    fn validate_rebind_recovery(
        &self,
        recovery: &UiDetachedRebindRecovery,
    ) -> Result<(), WorthUiNativeManagedRebindDenial> {
        if recovery.session_identity() != self.session.session_identity() {
            return Err(WorthUiNativeManagedRebindDenial::SessionMismatch);
        }
        Ok(())
    }
}
