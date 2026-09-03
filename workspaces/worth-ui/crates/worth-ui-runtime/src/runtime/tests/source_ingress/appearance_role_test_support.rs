pub(crate) fn validation_background_role(slot: &str) -> worth_ui_dsl::UiAppearanceRoleDeclaration {
    validation_background_role_for(slot, worth_ui_dsl::UiAppearanceStateAxis::Validation)
}

pub(crate) fn validation_background_role_with_axis(
    slot: &str,
    axis: worth_ui_dsl::UiAppearanceStateAxis,
) -> worth_ui_dsl::UiAppearanceRoleDeclaration {
    validation_background_role_for(slot, axis)
}

fn validation_background_role_for(
    slot: &str,
    axis: worth_ui_dsl::UiAppearanceStateAxis,
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
                worth_ui_dsl::UiThemeSlotIdentity::new(slot).unwrap(),
                worth_ui_dsl::UiThemeValueKind::Color,
            ),
    )
    .compile(worth_ui_dsl::UiAppearanceAspect::Background)
    .unwrap();
    worth_ui_dsl::UiAppearanceRoleDeclaration::admit(
        worth_ui_dsl::UiAppearanceRoleIdentity::new("test.validation-background").unwrap(),
        worth_ui_dsl::UiAppearanceRoleRevision::new(1).unwrap(),
        worth_ui_dsl::UiAppearanceRoleApplicability::AnyComponent,
        &contract,
        [(worth_ui_dsl::UiAppearanceAspect::Background, partition)],
    )
    .unwrap()
}
