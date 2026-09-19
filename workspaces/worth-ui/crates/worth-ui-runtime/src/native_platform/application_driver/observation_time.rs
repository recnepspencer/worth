use super::UiNativeApplicationDriver;
use worth_ui_host_native::{UiNativeEventLoopDirective, UiNativeObservationTimeProgress};

impl UiNativeApplicationDriver {
    pub(super) fn progress_observation_time(
        &mut self,
    ) -> Result<UiNativeObservationTimeProgress, ()> {
        let Some(shell) = self.shell.as_mut() else {
            return Ok(UiNativeObservationTimeProgress::new(
                None,
                UiNativeEventLoopDirective::Continue,
            ));
        };
        let deadline = shell.close_native_observation_time()?;
        if !shell.native_pointer_presentation_pending()
            || shell.native_motion_sample_presentation_pending()
        {
            return Ok(UiNativeObservationTimeProgress::new(
                deadline,
                UiNativeEventLoopDirective::Continue,
            ));
        }
        let directive = if self.application_runtime_active {
            self.progress_application_runtime_pointer()?
        } else {
            self.progress.refresh_pointer(shell)?;
            self.next_directive()
        };
        // A presentation callback can change owner truth. Seal it again before
        // scheduling; the next turn compares it with committed output independently.
        let deadline = self
            .shell
            .as_mut()
            .ok_or(())?
            .close_native_observation_time()?;
        Ok(UiNativeObservationTimeProgress::new(deadline, directive))
    }
}
