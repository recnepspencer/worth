use super::super::PlatformPulseApplicationRuntime;

impl PlatformPulseApplicationRuntime {
    pub(super) fn drain_native_observations(
        &mut self,
        shell: &mut worth_ui::facade::app::WorthUiNativeApplicationShell,
    ) {
        while self.terminal.is_running()
            && self.pending_managed_rebind.is_none()
            && self.pending_frame_presentation.is_none()
            && shell.native_frame_boundary_available()
        {
            let Some(progress) = self.pending_native_observations.pop_front() else {
                break;
            };
            if let Some(denial) = progress
                .focus_publications()
                .find_map(|result| result.as_ref().err())
            {
                self.fail(
                    super::super::PlatformPulseTerminalError::FocusPlacement(*denial),
                    Ok(()),
                );
                break;
            }
            if let Err(denial) = self.native_input.observe_native(&progress, &self.publisher) {
                self.fail(
                    super::super::PlatformPulseTerminalError::ObservationPublication,
                    Err(denial),
                );
                break;
            }
            self.admit_worth_native_intent_input(shell, progress);
        }
    }
}
