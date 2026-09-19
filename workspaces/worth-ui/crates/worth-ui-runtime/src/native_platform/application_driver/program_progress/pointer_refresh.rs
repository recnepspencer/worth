use super::{FrameProgress, UiNativeApplicationProgramProgress, UiNativePresentationSource};
use crate::facade::WorthUiNativeApplicationShell;

impl UiNativeApplicationProgramProgress {
    pub(in crate::native_platform::application_driver) fn refresh_pointer(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
    ) -> Result<(), ()> {
        if self.should_close()
            || !self.pending.is_empty()
            || self.pending_retry.is_some()
            || self.physical_recovery.has_pending()
            || self.staged_superseding_successor.is_some()
            || self.staged_superseding_predecessor.is_some()
            || shell.native_motion_sample_presentation_pending()
            || !shell.native_pointer_presentation_pending()
        {
            return Ok(());
        }
        let tick = self.next_present_tick;
        self.next_present_tick = tick.checked_add(1).ok_or(())?;
        let outcome = shell.present_frame(u64::MAX, tick).map_err(|_| ())?;
        match self.retain_or_attribute(
            shell,
            outcome,
            UiNativePresentationSource::PointerRefresh,
            None,
            None,
            false,
        )? {
            FrameProgress::Failed => Err(()),
            _ => Ok(()),
        }
    }
}
