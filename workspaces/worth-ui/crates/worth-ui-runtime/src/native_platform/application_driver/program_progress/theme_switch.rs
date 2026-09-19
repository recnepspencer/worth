use super::{
    UiNativeApplicationProgramProgress, UiNativePresentationSource, WorthUiNativeApplicationShell,
};
use crate::facade::entry::{WorthUiNativeManagedRebindProgress, WorthUiNativeManagedRebindStop};
use crate::runtime::rebind::{UiRebindExecutionPolicy, UiRebindExecutionRequest};

impl UiNativeApplicationProgramProgress {
    pub(super) fn begin_theme_switch(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        definition: crate::capability::UiThemeDefinitionIdentity,
        frame: usize,
        tick: u64,
    ) -> Result<(), ()> {
        let surface = shell.native_layout_basis().map_err(|_| ())?.surface();
        let predecessor = shell
            .active_theme_binding(surface)
            .ok_or(())?
            .binding_generation();
        let capability = shell
            .admit_appearance_theme(surface, &definition)
            .map_err(|_| ())?;
        let request = shell
            .prepare_programmatic_theme_switch(surface, predecessor, capability)
            .map_err(|_| ())?;
        let progress = shell
            .begin_managed_theme_switch(
                request,
                UiRebindExecutionPolicy::ordinary(),
                UiRebindExecutionRequest::new(tick),
            )
            .map_err(|_| ())?;
        self.settle_theme_progress(shell, frame, progress)
    }

    pub(super) fn progress_theme_switch(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        progress: &crate::native_platform::UiNativeApplicationPhysicalProgress,
    ) -> Result<(), ()> {
        let frame = self.pending_theme_frame.ok_or(())?;
        let progress = shell.progress_managed_rebind(progress).map_err(|_| ())?;
        self.settle_theme_progress(shell, frame, progress)
    }

    fn settle_theme_progress(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        frame: usize,
        progress: WorthUiNativeManagedRebindProgress,
    ) -> Result<(), ()> {
        match progress {
            WorthUiNativeManagedRebindProgress::RebindRecovered(_) => {
                let definition = self
                    .program
                    .frames()
                    .get(frame)
                    .and_then(|frame| frame.theme_switch())
                    .cloned()
                    .ok_or(())?;
                self.next_present_tick = self.next_present_tick.saturating_add(1);
                self.begin_theme_switch(shell, definition, frame, self.next_present_tick)
            }
            WorthUiNativeManagedRebindProgress::Published(_)
            | WorthUiNativeManagedRebindProgress::Stopped(
                WorthUiNativeManagedRebindStop::ObservedNoChange,
            ) => {
                self.pending_theme_frame = None;
                self.settle_attribution(
                    shell,
                    UiNativePresentationSource::Program(frame),
                    #[cfg(feature = "certification-support")]
                    false,
                )?;
                Ok(())
            }
            WorthUiNativeManagedRebindProgress::AwaitingProgress
            | WorthUiNativeManagedRebindProgress::RecoveryBlocked(_) => {
                self.pending_theme_frame = Some(frame);
                Ok(())
            }
            WorthUiNativeManagedRebindProgress::Unrelated
            | WorthUiNativeManagedRebindProgress::RecoveredToPredecessor(_)
            | WorthUiNativeManagedRebindProgress::IntentConsequencePublished(_)
            | WorthUiNativeManagedRebindProgress::PortalDismissed(_) => Err(()),
            WorthUiNativeManagedRebindProgress::Stopped(_) => Err(()),
        }
    }
}
