use worth_ui::facade::appearance::{
    UiAppearanceAspect, UiAppearanceCell, UiAppearancePartitionAuthoring, UiAppearanceRole,
    UiAppearanceRoleIdentity, UiDslComponentReference, UiThemeSlotIdentity, UiThemeValueKind,
};

const GUIDE: &str = include_str!("../../../../docs/appearance-and-themes.md");
const PULSE_DSL: &str = include_str!("../../../../apps/platform-pulse/app/main.wui");

#[test]
fn documented_rust_role_uses_the_public_facade() {
    assert!(GUIDE.contains("compiled-example:appearance-role-rust"));
    let role =
        UiAppearanceRole::authoring(UiAppearanceRoleIdentity::new("app.appearance.card").unwrap())
            .applies_to_component(UiDslComponentReference::new("app.component.card").unwrap())
            .cover(
                UiAppearanceAspect::Background,
                UiAppearancePartitionAuthoring::new([]).with_cell(
                    UiAppearanceCell::when([]).uses_slot(
                        UiThemeSlotIdentity::new("app.theme.surface").unwrap(),
                        UiThemeValueKind::Color,
                    ),
                ),
            )
            .unwrap()
            .build()
            .unwrap();
    let _ = role;
}

#[test]
fn documented_dsl_role_is_the_production_pulse_source() {
    assert!(GUIDE.contains("compiled-example:appearance-role-dsl"));
    for line in [
        "appearance role platform.pulse.appearance.query_card",
        "applies_to platform.pulse.component.query_card",
        "background use token(theme.platform_pulse.raised_surface)",
        "border use token(theme.platform_pulse.border.query_card)",
        "radius use token(theme.platform_pulse.radius.query_card)",
        "appearance { role platform.pulse.appearance.query_card }",
    ] {
        assert!(GUIDE.contains(line), "guide omitted `{line}`");
        assert!(
            PULSE_DSL.contains(line),
            "production source omitted `{line}`"
        );
    }
}
