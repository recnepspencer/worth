use worth_ui::facade::app::{
    WorthUiNativeApplicationShell, WorthUiNativeIntentPosture, WorthUiNativeIntentPostureKind,
};
use worth_ui_native_platform::UiNativeApplicationProgramDenial;
use worth_ui_platform_pulse::product_world::PlatformPulseProductComponent as Component;

impl super::PlatformPulseProductStory {
    /// Admit product copy before the posture's mounted transaction. Publication
    /// consumes these exact revisions alongside the owner-issued observation.
    pub(in crate::native_application) fn prepare_intent_feedback(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        posture: &WorthUiNativeIntentPosture,
    ) -> Result<(), UiNativeApplicationProgramDenial> {
        let target = [
            (Component::ActionTarget, Component::ActionLabel),
            (Component::ConfirmationTarget, Component::ConfirmationLabel),
            (Component::PortalTarget, Component::PortalLabel),
            (Component::PortalCancelTarget, Component::PortalCancelLabel),
            (
                Component::PortalPrimaryTarget,
                Component::PortalPrimaryLabel,
            ),
        ]
        .into_iter()
        .find(|(target, _)| {
            shell.mounted_component_instance(&target.authored_semantic_identity())
                == Some(posture.mounted_instance())
        });
        let Some((target, label)) = target else {
            return Ok(());
        };
        let retained = match target {
            Component::ActionTarget => &mut self.action,
            Component::ConfirmationTarget => &mut self.confirmation,
            Component::PortalTarget => &mut self.portal,
            Component::PortalCancelTarget => &mut self.portal_cancel,
            Component::PortalPrimaryTarget => &mut self.portal_primary,
            _ => unreachable!("feedback targets are enumerated above"),
        };
        let text = match posture.kind() {
            WorthUiNativeIntentPostureKind::Admitted => "Working…",
            WorthUiNativeIntentPostureKind::ConfirmationRequired => "Confirm action",
            WorthUiNativeIntentPostureKind::Completed => "Completed",
            WorthUiNativeIntentPostureKind::Denied => "Unavailable",
            WorthUiNativeIntentPostureKind::StaleConfirmation => "Confirm again",
            WorthUiNativeIntentPostureKind::Cancelled => "Cancelled",
        };
        super::publish_changed(shell, retained, text.to_owned(), label)
    }
}
