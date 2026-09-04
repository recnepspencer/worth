use crate::runtime::tests::appearance_component_session_test_support as support;

pub(super) const SWITCHED_SLOT: &str = "theme.appearance_consumer.switched";

pub(super) fn single_aspect_role(
    identity: &str,
    aspect: worth_ui_dsl::UiAppearanceAspect,
    slot: &str,
) -> worth_ui_dsl::UiAppearanceRoleDeclaration {
    let contract = worth_ui_dsl::UiAppearanceAspectContract::component([aspect], []).unwrap();
    let partition = worth_ui_dsl::UiAppearancePartitionAuthoring::new([
        worth_ui_dsl::UiAppearanceAxisDomain::complete(
            worth_ui_dsl::UiAppearanceStateAxis::Validation,
        ),
    ])
    .with_cell(
        worth_ui_dsl::UiAppearanceCell::when([worth_ui_dsl::UiAppearanceAxisPredicate::any(
            worth_ui_dsl::UiAppearanceStateAxis::Validation,
        )])
        .uses_slot(
            worth_ui_dsl::UiThemeSlotIdentity::new(slot).unwrap(),
            worth_ui_dsl::UiThemeValueKind::Color,
        ),
    )
    .compile(aspect)
    .unwrap();
    worth_ui_dsl::UiAppearanceRoleDeclaration::admit(
        worth_ui_dsl::UiAppearanceRoleIdentity::new(identity).unwrap(),
        worth_ui_dsl::UiAppearanceRoleRevision::new(1).unwrap(),
        worth_ui_dsl::UiAppearanceRoleApplicability::AnyComponent,
        &contract,
        [(aspect, partition)],
    )
    .unwrap()
}

pub(super) fn switched_validation_role() -> worth_ui_dsl::UiAppearanceRoleDeclaration {
    let aspect = worth_ui_dsl::UiAppearanceAspect::Background;
    let contract = worth_ui_dsl::UiAppearanceAspectContract::component([aspect], []).unwrap();
    let partition = worth_ui_dsl::UiAppearancePartitionAuthoring::new([
        worth_ui_dsl::UiAppearanceAxisDomain::complete(
            worth_ui_dsl::UiAppearanceStateAxis::Validation,
        ),
    ])
    .with_cell(
        worth_ui_dsl::UiAppearanceCell::named("default")
            .when([worth_ui_dsl::UiAppearanceAxisPredicate::exact(
                worth_ui_dsl::UiAppearanceAxisClass::ValidationUnspecified,
            )])
            .uses_slot(
                worth_ui_dsl::UiThemeSlotIdentity::new(support::APPEARANCE_TOKEN).unwrap(),
                worth_ui_dsl::UiThemeValueKind::Color,
            ),
    )
    .with_cell(
        worth_ui_dsl::UiAppearanceCell::named("invalid")
            .when([worth_ui_dsl::UiAppearanceAxisPredicate::exact(
                worth_ui_dsl::UiAppearanceAxisClass::ValidationInvalid,
            )])
            .uses_slot(
                worth_ui_dsl::UiThemeSlotIdentity::new(SWITCHED_SLOT).unwrap(),
                worth_ui_dsl::UiThemeValueKind::Color,
            ),
    )
    .otherwise_same_as("default")
    .compile(aspect)
    .unwrap();
    worth_ui_dsl::UiAppearanceRoleDeclaration::admit(
        worth_ui_dsl::UiAppearanceRoleIdentity::new("test.receipt-semantic").unwrap(),
        worth_ui_dsl::UiAppearanceRoleRevision::new(1).unwrap(),
        worth_ui_dsl::UiAppearanceRoleApplicability::AnyComponent,
        &contract,
        [(aspect, partition)],
    )
    .unwrap()
}

pub(super) fn theme_bundle(
    extra: &[&str],
    initial_color: Option<&str>,
) -> crate::capability::FrozenAppearanceThemeCapabilities {
    let mut tokens = vec![crate::capability::ThemeTokenId::new(support::APPEARANCE_TOKEN).unwrap()];
    tokens.extend(
        extra
            .iter()
            .map(|slot| crate::capability::ThemeTokenId::new(*slot).unwrap()),
    );
    let catalog = crate::capability::UiThemeSlotCatalog::admit(
        1,
        tokens.iter().cloned().map(|token| {
            crate::capability::UiThemeSlotDeclaration::new(
                token,
                crate::capability::ThemeTokenFamily::surface(),
                worth_ui_dsl::UiThemeValueKind::Color,
                crate::capability::ThemeTokenSource::application(),
                crate::capability::UiThemeSlotDisclosure::Public,
                crate::capability::UiThemeSlotSuccessorCompatibility::ExactMeaning,
                None,
            )
        }),
    )
    .unwrap();
    let initial = initial_color.unwrap_or("#112233");
    let definition = crate::capability::UiThemeDefinition::admit(
        crate::capability::UiThemeDefinitionIdentity::new("theme.appearance.receipts").unwrap(),
        1,
        &catalog,
        tokens.into_iter().map(|token| {
            (
                token,
                worth_ui_dsl::UiThemeValue::Color(initial_theme_color(initial)),
            )
        }),
    )
    .unwrap();
    crate::capability::FrozenAppearanceThemeCapabilities::admit(catalog, vec![definition]).unwrap()
}

pub(super) fn update_theme(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    hex: &str,
) {
    update_theme_at_revision(session, hex, 0);
}

pub(super) fn update_theme_at_revision(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    hex: &str,
    expected_revision: u64,
) {
    let token = crate::capability::ThemeTokenId::new(support::APPEARANCE_TOKEN).unwrap();
    let value = crate::capability::ThemeTokenValue::color(
        crate::capability::ThemeColorValue::hex(hex).unwrap(),
    );
    let change = super::super::super::UiNativeThemeTokenValueChange::successor(
        token.clone(),
        expected_revision,
        value,
    )
    .unwrap();
    session.admit_application_theme_values(&[change]).unwrap();
}

fn initial_theme_color(hex: &str) -> worth_ui_dsl::UiThemeColor {
    let channels = match hex {
        "#405060" => [64, 80, 96, 255],
        "#112233" => [17, 34, 51, 255],
        _ => panic!("receipt fixture uses a declared theme color"),
    };
    worth_ui_dsl::UiThemeColor::from_channels(channels)
}
