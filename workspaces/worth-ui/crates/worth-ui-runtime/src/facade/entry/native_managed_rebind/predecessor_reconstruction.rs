use super::{
    finish_normalized_managed_rebind, WorthUiNativeManagedRebindDenial,
    WorthUiNativeManagedRebindProgress, WorthUiNativeManagedRebindStop,
    WorthUiNativePendingManagedRebind,
};

pub(super) fn reconstruction_matches_progress(
    in_flight: &crate::mounting::UiMountedPresentationInFlight,
    progress: &crate::native_platform::UiNativeApplicationPhysicalProgress,
) -> bool {
    progress.class() == worth_ui_host_native::UiNativePhysicalProgressClass::Presentation
        && in_flight.awaits_progress_class(
            worth_ui_host_contract::UiHostPresentationProgressClass::PhysicalSurface,
        )
        && progress.presentation().is_some_and(|presentation| {
            presentation.attempt() == in_flight.attempt()
                && in_flight
                    .pending_bindings()
                    .any(|binding| binding == presentation.binding())
        })
}

pub(super) fn reconstruction_settled(outcome: &crate::mounting::UiMountedFrameOutcome) -> bool {
    matches!(
        outcome,
        crate::mounting::UiMountedFrameOutcome::Published(_)
            | crate::mounting::UiMountedFrameOutcome::Unchanged(_)
            | crate::mounting::UiMountedFrameOutcome::Reconciled(_)
    )
}

impl super::WorthUiNativeApplicationShell {
    pub(in crate::facade::entry::native_managed_rebind) fn begin_predecessor_reconstruction(
        &mut self,
        retry: crate::runtime::rebind::UiDetachedRebindRetry,
    ) -> Result<WorthUiNativeManagedRebindProgress, WorthUiNativeManagedRebindDenial> {
        if retry.session_identity() != self.session.session_identity() {
            return Err(WorthUiNativeManagedRebindDenial::SessionMismatch);
        }
        let recovery = self
            .reconstruct_current_presentation(u64::MAX, self.managed_rebind_completion_tick)
            .map_err(|()| WorthUiNativeManagedRebindDenial::PredecessorReconstruction)?;
        match recovery {
            crate::mounting::UiMountedFrameOutcome::InFlight(in_flight) => {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::PredecessorReconstruction {
                        retry,
                        in_flight,
                    },
                );
                Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress)
            }
            outcome if reconstruction_settled(&outcome) => {
                let outcome = retry
                    .rebase_content_and_retry(
                        &mut self.session,
                        self.managed_rebind_completion_tick,
                    )
                    .map_err(WorthUiNativeManagedRebindDenial::Preparation)?;
                Ok(finish_normalized_managed_rebind(
                    &mut self.pending_managed_rebind,
                    outcome,
                ))
            }
            _ => Ok(WorthUiNativeManagedRebindProgress::Stopped(
                WorthUiNativeManagedRebindStop::PredecessorReconstructionFailed,
            )),
        }
    }
}
