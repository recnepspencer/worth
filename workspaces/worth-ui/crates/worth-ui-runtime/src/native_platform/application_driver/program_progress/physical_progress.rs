use super::*;

#[path = "physical_progress/pending_completion.rs"]
mod pending_completion;
#[path = "physical_progress/recovery_progress.rs"]
mod recovery_progress;
#[path = "physical_progress/settlement_progress.rs"]
mod settlement_progress;

impl UiNativeApplicationProgramProgress {
    #[cfg(test)]
    pub(in crate::native_platform::application_driver) fn settle_first_pending_presentation_for_test(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
    ) -> Result<(), ()> {
        let pending = self.pending.pop_front().ok_or(())?;
        let completed = self.complete_pending_physical_progress(shell, pending, None)?;
        self.settle_completed_physical_progress(shell, completed, None, false)
    }

    pub(in crate::native_platform::application_driver) fn physical_work_progressed(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        mut grant: worth_ui_host_native::UiNativePhysicalProgressGrant,
    ) -> Result<(), ()> {
        if self.pending_theme_frame.is_some() {
            let progress =
                crate::native_platform::UiNativeApplicationPhysicalProgress::from_host(grant);
            self.progress_theme_switch(shell, &progress)?;
            return self.advance(shell);
        }
        if shell.component_presence_awaits_portal_dismissal() {
            let progress =
                crate::native_platform::UiNativeApplicationPhysicalProgress::from_host(grant);
            let managed = shell.progress_managed_rebind(&progress).map_err(|_| ())?;
            grant = progress.into_host();
            match managed {
                crate::facade::entry::WorthUiNativeManagedRebindProgress::PortalDismissed(_)
                | crate::facade::entry::WorthUiNativeManagedRebindProgress::RecoveredToPredecessor(
                    crate::facade::entry::WorthUiNativePredecessorRecovery::PortalDismissal,
                ) => {
                    self.next_completion_tick = self.next_completion_tick.saturating_add(1);
                    shell
                        .resume_pending_component_presence(self.next_completion_tick)
                        .map_err(|_| ())?;
                    return self.advance(shell);
                }
                crate::facade::entry::WorthUiNativeManagedRebindProgress::AwaitingProgress => {}
                crate::facade::entry::WorthUiNativeManagedRebindProgress::RecoveryBlocked(_) => {
                    return Ok(())
                }
                crate::facade::entry::WorthUiNativeManagedRebindProgress::Stopped(_) => {
                    return Err(())
                }
                crate::facade::entry::WorthUiNativeManagedRebindProgress::Unrelated
                | crate::facade::entry::WorthUiNativeManagedRebindProgress::RebindRecovered(_)
                | crate::facade::entry::WorthUiNativeManagedRebindProgress::Published(_)
                | crate::facade::entry::WorthUiNativeManagedRebindProgress::IntentConsequencePublished(_)
                | crate::facade::entry::WorthUiNativeManagedRebindProgress::RecoveredToPredecessor(_) => {
                    return Err(())
                }
            }
        }
        let class = grant.class();
        if class == worth_ui_host_native::UiNativePhysicalProgressClass::PresentationRecovery {
            let presentation = grant.presentation().ok_or(())?;
            return self.progress_presentation_recovery(shell, presentation);
        }
        if class == worth_ui_host_native::UiNativePhysicalProgressClass::TextAtlas
            && self.retry_readiness() == Some(UiNativeProgramRetryReadiness::TextAtlas)
        {
            self.progress_text_atlas_retry(shell)?;
            return self.advance(shell);
        }
        let presentation = grant.presentation();
        let Some(pending) = self.take_pending_for_physical_progress(class, presentation)? else {
            return self.advance(shell);
        };
        let completed = self.complete_pending_physical_progress(shell, pending, presentation)?;
        self.settle_completed_physical_progress(
            shell,
            completed,
            presentation,
            grant.duplicate_presentation_observed(),
        )
    }
}
