use worth_ui::facade::appearance::{
    UiAppearanceAspect, UiAppearanceCell, UiAppearancePartitionAuthoring, UiAppearanceRole,
    UiAppearanceRoleIdentity, UiDslComponentReference, UiThemeSlotIdentity, UiThemeValueKind,
};

const GUIDE: &str = include_str!("../../../../docs/appearance-and-themes.md");

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
