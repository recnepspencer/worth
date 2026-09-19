use super::{
    PlatformPulseApplicationRuntime, PlatformPulseIntentPosturePublicationDisposition,
    PlatformPulsePreparedIntentPosture, WorthUiNativeApplicationShell,
};

pub(in crate::native_application) enum PlatformPulsePendingNativePublication {
    Intent(PlatformPulsePreparedIntentPosture),
    Dismiss(worth_ui::facade::interaction::UiDismissInteraction),
}

impl PlatformPulseApplicationRuntime {
    pub(in crate::native_application) fn advance_pending_native_publications(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
    ) {
        while self.terminal_error.is_none()
            && !self.visual_identity.retains_rebind_receipt()
            && self.pending_managed_rebind.is_none()
            && self.pending_frame_presentation.is_none()
        {
            let Some(prepared) = self.pending_native_publications.pop_front() else {
                return;
            };
            match prepared {
                PlatformPulsePendingNativePublication::Intent(posture) => {
                    if !matches!(
                        self.publish_native_intent_posture(shell, posture),
                        PlatformPulseIntentPosturePublicationDisposition::Published
                    ) {
                        return;
                    }
                }
                PlatformPulsePendingNativePublication::Dismiss(dismissal) => {
                    if !self.dismiss_open_portal(shell, dismissal) {
                        return;
                    }
                }
            }
        }
    }
}
