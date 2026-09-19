use worth_ui::facade::app::{UiMountedInstanceIdentity, WorthUiNativeApplicationShell};
use worth_ui_native_platform::{
    UiNativeApplicationProgramDenial, UiNativeComponentSemanticTextChange,
};

#[derive(Default)]
pub(super) struct PeriodSelection {
    revision: u64,
}

impl PeriodSelection {
    pub(super) fn publish_completed(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        target_instance: UiMountedInstanceIdentity,
    ) -> Result<(), UiNativeApplicationProgramDenial> {
        let ranges = [
            (
                "period_day",
                "Last 24 hours",
                ["12 AM", "4 AM", "8 AM", "12 PM", "4 PM", "8 PM", "12 AM"],
            ),
            (
                "period_week",
                "Last 7 days",
                ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
            ),
            (
                "period_month",
                "Last 30 days",
                [
                    "Day 1", "Day 5", "Day 10", "Day 15", "Day 20", "Day 25", "Day 30",
                ],
            ),
        ];
        let Some((_, label, ticks)) = ranges.iter().find(|(target, _, _)| {
            shell
                .mounted_component_instance(&format!("component:platform.pulse.component.{target}"))
                == Some(target_instance)
        }) else {
            return Ok(());
        };
        let revision = &mut self.revision;
        let successor = revision
            .checked_add(1)
            .ok_or(UiNativeApplicationProgramDenial::ChangeCapacityExceeded)?;
        let mut changes = vec![UiNativeComponentSemanticTextChange::successor(
            "component:platform.pulse.component.confirmation_label",
            successor,
            *label,
        )?];
        for (index, tick) in ticks.iter().enumerate() {
            changes.push(UiNativeComponentSemanticTextChange::successor(
                format!("component:platform.pulse.component.chart_x_{index}"),
                successor,
                *tick,
            )?);
        }
        shell.apply_component_semantic_text(&changes)?;
        *revision = successor;
        Ok(())
    }
}
