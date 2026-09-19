use super::appearance_component_session_test_support::{
    appearance_component_builder, appearance_fixture, APPEARANCE_TOKEN,
};
use crate::runtime::tests::appearance_theme_test_support;

pub(crate) fn ownerless_focus_consumer_app() -> crate::facade::WorthUiApp {
    let role = focus_background_role();
    appearance_component_builder(&role)
        .register_appearance_theme_bundle(appearance_theme_test_support::bundle())
        .unwrap()
        .with_rust_authored_declaration_fixture(appearance_fixture(&role))
        .freeze()
        .map(appearance_theme_test_support::activate)
        .expect("ownerless Focus consumer should prepare before launch admission")
}

pub(crate) fn focus_background_role() -> worth_ui_dsl::UiAppearanceRoleDeclaration {
    axis_background_role(
        worth_ui_dsl::UiAppearanceStateAxis::Focus,
        "test.focus-background",
    )
}

fn axis_background_role(
    axis: worth_ui_dsl::UiAppearanceStateAxis,
    identity: &str,
) -> worth_ui_dsl::UiAppearanceRoleDeclaration {
    let contract = worth_ui_dsl::UiAppearanceAspectContract::component(
        [worth_ui_dsl::UiAppearanceAspect::Background],
        [],
    )
    .unwrap();
    let partition = worth_ui_dsl::UiAppearancePartitionAuthoring::new([
        worth_ui_dsl::UiAppearanceAxisDomain::complete(axis),
    ])
    .with_cell(
        worth_ui_dsl::UiAppearanceCell::when([worth_ui_dsl::UiAppearanceAxisPredicate::any(axis)])
            .uses_slot(
                worth_ui_dsl::UiThemeSlotIdentity::new(APPEARANCE_TOKEN).unwrap(),
                worth_ui_dsl::UiThemeValueKind::Color,
            ),
    )
    .compile(worth_ui_dsl::UiAppearanceAspect::Background)
    .unwrap();
    worth_ui_dsl::UiAppearanceRoleDeclaration::admit(
        worth_ui_dsl::UiAppearanceRoleIdentity::new(identity).unwrap(),
        worth_ui_dsl::UiAppearanceRoleRevision::new(1).unwrap(),
        worth_ui_dsl::UiAppearanceRoleApplicability::AnyComponent,
        &contract,
        [(worth_ui_dsl::UiAppearanceAspect::Background, partition)],
    )
    .unwrap()
}
