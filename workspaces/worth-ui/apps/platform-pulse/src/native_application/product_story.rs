use worth_ui::facade::app::{
    WorthUiNativeApplicationShell, WorthUiNativeIntentPosture, WorthUiNativeIntentPostureKind,
};
use worth_ui_native_platform::{
    UiNativeApplicationProgramDenial, UiNativeComponentSemanticTextChange,
};
use worth_ui_platform_pulse::product_world::PlatformPulseProductComponent as Component;

mod period_selection;

#[derive(Default)]
pub(super) struct PlatformPulseProductStory {
    approval_revision: u64,
    denial_revision: u64,
    period: period_selection::PeriodSelection,
}
impl PlatformPulseProductStory {
    pub(in crate::native_application) fn prepare_intent_feedback(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        posture: &WorthUiNativeIntentPosture,
    ) -> Result<(), UiNativeApplicationProgramDenial> {
        if shell.mounted_component_instance(
            &Component::ReviewPrimaryTarget.authored_semantic_identity(),
        ) != Some(posture.mounted_instance())
        {
            return Ok(());
        }
        let text = match posture.kind() {
            WorthUiNativeIntentPostureKind::Admitted => "Approving…",
            WorthUiNativeIntentPostureKind::Completed => "Deployment approved",
            WorthUiNativeIntentPostureKind::ConfirmationRequired => "Confirm deployment",
            WorthUiNativeIntentPostureKind::Denied => "Approval unavailable",
            WorthUiNativeIntentPostureKind::StaleConfirmation => "Please confirm again",
            WorthUiNativeIntentPostureKind::Cancelled => "Approval cancelled",
        };
        publish(
            shell,
            Component::ReviewPrimaryLabel,
            text,
            &mut self.approval_revision,
        )
    }
    pub(in crate::native_application) fn prepare_completed_feedback(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        target: worth_ui::facade::app::UiMountedInstanceIdentity,
    ) -> Result<(), UiNativeApplicationProgramDenial> {
        self.period.publish_completed(shell, target)?;
        if shell.mounted_component_instance(
            &Component::ReviewPrimaryTarget.authored_semantic_identity(),
        ) == Some(target)
        {
            publish(
                shell,
                Component::ReviewPrimaryLabel,
                "Deployment approved",
                &mut self.approval_revision,
            )?;
        }
        Ok(())
    }
    pub(super) fn publish_query_denial(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        denial:worth_ui_platform_pulse::observation_contract::PlatformPulseQueryActionPreconditionDenial,
    ) -> Result<(), UiNativeApplicationProgramDenial> {
        let story =
            worth_ui_platform_pulse::product_world::PlatformPulseQueryDenialStory::new(denial);
        publish(
            shell,
            Component::ReviewBody,
            story.explanation(),
            &mut self.denial_revision,
        )
    }
}
fn publish(
    shell: &mut WorthUiNativeApplicationShell,
    component: Component,
    text: &str,
    revision: &mut u64,
) -> Result<(), UiNativeApplicationProgramDenial> {
    let successor = revision
        .checked_add(1)
        .ok_or(UiNativeApplicationProgramDenial::ChangeCapacityExceeded)?;
    shell.apply_component_semantic_text(&[UiNativeComponentSemanticTextChange::successor(
        component.authored_semantic_identity(),
        successor,
        text,
    )?])?;
    *revision = successor;
    Ok(())
}
