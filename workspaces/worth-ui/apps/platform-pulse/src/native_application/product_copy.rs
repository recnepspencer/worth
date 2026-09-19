use worth_ui::facade::app::WorthUiNativeApplicationShell;
use worth_ui_native_platform::{
    UiNativeApplicationProgramDenial, UiNativeComponentSemanticTextChange,
};
use worth_ui_platform_pulse::product_world::{dashboard_elements, DashboardContent};

pub(super) fn install(
    shell: &mut WorthUiNativeApplicationShell,
) -> Result<(), UiNativeApplicationProgramDenial> {
    let changes = dashboard_elements()
        .into_iter()
        .filter_map(|element| {
            if element.id == "projected_status" {
                return None;
            }
            let DashboardContent::Text { value, .. } = element.content else {
                return None;
            };
            Some(
                UiNativeComponentSemanticTextChange::new(
                    format!("component:{}", element.component_id()),
                    value,
                )
                .expect("authored dashboard copy is bounded"),
            )
        })
        .collect::<Vec<_>>();
    shell.apply_component_semantic_text(&changes)
}
