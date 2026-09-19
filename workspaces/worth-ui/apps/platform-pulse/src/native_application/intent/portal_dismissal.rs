use worth_ui::facade::app::{
    WorthUiNativeApplicationShell, WorthUiNativeManagedPortalDismissalOutcome,
};

use super::super::{PlatformPulseApplicationRuntime, PlatformPulsePendingManagedRebind};

impl PlatformPulseApplicationRuntime {
    pub(in crate::native_application) fn dismiss_open_portal(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        dismissal: worth_ui::facade::interaction::UiDismissInteraction,
    ) -> bool {
        self.presentation_tick = self.presentation_tick.saturating_add(1);
        let outcome = shell.begin_managed_portal_dismissal(dismissal, self.presentation_tick);
        self.handle_portal_dismissal_outcome(shell, outcome)
    }

    pub(in crate::native_application) fn continue_retained_portal_dismissal(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
    ) -> bool {
        self.presentation_tick = self.presentation_tick.saturating_add(1);
        let outcome =
            shell.continue_retained_portal_dismissal_after_managed_intent(self.presentation_tick);
        self.handle_portal_dismissal_outcome(shell, outcome)
    }

    fn handle_portal_dismissal_outcome(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        outcome: WorthUiNativeManagedPortalDismissalOutcome,
    ) -> bool {
        match outcome {
            WorthUiNativeManagedPortalDismissalOutcome::Ignored => true,
            WorthUiNativeManagedPortalDismissalOutcome::Retained => true,
            WorthUiNativeManagedPortalDismissalOutcome::Stopped(
                worth_ui::facade::app::WorthUiNativePortalDismissalStop::StalePresentation,
            ) => true,
            WorthUiNativeManagedPortalDismissalOutcome::Published(receipt) => {
                self.settle_portal_dismissal(shell, receipt)
            }
            WorthUiNativeManagedPortalDismissalOutcome::Pending => {
                self.pending_managed_rebind =
                    Some(PlatformPulsePendingManagedRebind::PortalDismissal);
                true
            }
            WorthUiNativeManagedPortalDismissalOutcome::Stopped(stop) => {
                self.fail_intent_settlement(format!(
                    "native portal dismissal did not publish: {stop:?}"
                ));
                false
            }
        }
    }

    pub(in crate::native_application) fn settle_portal_dismissal(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        receipt: worth_ui::facade::app::UiPortalDismissalPublicationReceipt,
    ) -> bool {
        if let Err(error) = self
            .publisher
            .project_observation(|stream| stream.project_portal_dismissed(receipt.mounted()))
        {
            self.fail(
                super::super::PlatformPulseTerminalError::ObservationPublication,
                Err(error),
            );
            return false;
        }
        if let Err(error) = self
            .publisher
            .semantic_focus_published(receipt.focus_publication(), receipt.mounted())
        {
            self.fail(
                super::super::PlatformPulseTerminalError::ObservationPublication,
                Err(error),
            );
            return false;
        }
        match self.visual_identity.refresh_after_presentation_replacement(
            shell,
            self.presentation_tick,
            std::time::Instant::now(),
        ) {
            Ok(()) => true,
            Err(denial) => {
                self.fail_visual_identity(denial);
                false
            }
        }
    }
}
