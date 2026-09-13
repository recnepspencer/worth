use crate::runtime::tests::appearance_component_session_test_support as support;

#[path = "appearance_receipt_role_test_support/radius.rs"]
mod radius;
pub(super) use radius::*;

pub(super) const SWITCHED_SLOT: &str = "theme.appearance_consumer.switched";
const RADIUS_BACKGROUND_TOKEN: &str = "theme.appearance_radius.background";

pub(super) fn theme_definition<'a>(
    themes: &'a crate::capability::FrozenAppearanceThemeCapabilities,
    identity: &str,
) -> &'a crate::capability::UiThemeDefinition {
    let identity = crate::capability::UiThemeDefinitionIdentity::new(identity).unwrap();
    themes
        .get(&identity)
        .expect("the receipt fixture must contain its explicit theme definition")
}

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
            theme_value_kind(aspect),
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

pub(super) fn radius_role(identity: &str) -> worth_ui_dsl::UiAppearanceRoleDeclaration {
    let background = worth_ui_dsl::UiAppearanceAspect::Background;
    let radius = worth_ui_dsl::UiAppearanceAspect::Radius;
    let contract =
        worth_ui_dsl::UiAppearanceAspectContract::component([background, radius], []).unwrap();
    let background_partition = radius_partition(RADIUS_BACKGROUND_TOKEN, background);
    let radius_partition = radius_partition(support::APPEARANCE_TOKEN, radius);
    worth_ui_dsl::UiAppearanceRoleDeclaration::admit(
        worth_ui_dsl::UiAppearanceRoleIdentity::new(identity).unwrap(),
        worth_ui_dsl::UiAppearanceRoleRevision::new(1).unwrap(),
        worth_ui_dsl::UiAppearanceRoleApplicability::AnyComponent,
        &contract,
        [
            (background, background_partition),
            (radius, radius_partition),
        ],
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
    let definition = |identity: &str, color: &str| {
        crate::capability::UiThemeDefinition::admit(
            crate::capability::UiThemeDefinitionIdentity::new(identity).unwrap(),
            1,
            &catalog,
            tokens.iter().cloned().map(|token| {
                (
                    token,
                    worth_ui_dsl::UiThemeValue::Color(initial_theme_color(color)),
                )
            }),
        )
        .unwrap()
    };
    let definitions = vec![
        definition("theme.appearance.receipts", initial),
        definition("theme.appearance.receipts-405060", "#405060"),
    ];
    crate::capability::FrozenAppearanceThemeCapabilities::admit(
        catalog,
        crate::capability::UiThemeDefinitionIdentity::new("theme.appearance.receipts").unwrap(),
        definitions,
    )
    .unwrap()
}

pub(super) fn multi_definition_theme_bundle() -> crate::capability::FrozenAppearanceThemeCapabilities
{
    let token = crate::capability::ThemeTokenId::new(support::APPEARANCE_TOKEN).unwrap();
    let catalog = crate::capability::UiThemeSlotCatalog::admit(
        1,
        [crate::capability::UiThemeSlotDeclaration::new(
            token.clone(),
            crate::capability::ThemeTokenFamily::surface(),
            worth_ui_dsl::UiThemeValueKind::Color,
            crate::capability::ThemeTokenSource::application(),
            crate::capability::UiThemeSlotDisclosure::Public,
            crate::capability::UiThemeSlotSuccessorCompatibility::ExactMeaning,
            None,
        )],
    )
    .unwrap();
    let definition = |identity: &str, color| {
        crate::capability::UiThemeDefinition::admit(
            crate::capability::UiThemeDefinitionIdentity::new(identity).unwrap(),
            1,
            &catalog,
            [(
                token.clone(),
                worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels(color)),
            )],
        )
        .unwrap()
    };
    crate::capability::FrozenAppearanceThemeCapabilities::admit(
        catalog.clone(),
        crate::capability::UiThemeDefinitionIdentity::new("theme.appearance.default").unwrap(),
        vec![
            definition("theme.appearance.surface-a", [17, 34, 51, 255]),
            definition("theme.appearance.surface-b", [68, 85, 102, 255]),
            definition("theme.appearance.default", [119, 136, 153, 255]),
        ],
    )
    .unwrap()
}

pub(super) fn radius_theme_bundle() -> crate::capability::FrozenAppearanceThemeCapabilities {
    let token = crate::capability::ThemeTokenId::new(support::APPEARANCE_TOKEN).unwrap();
    let background = crate::capability::ThemeTokenId::new(RADIUS_BACKGROUND_TOKEN).unwrap();
    let catalog = crate::capability::UiThemeSlotCatalog::admit(
        1,
        [
            crate::capability::UiThemeSlotDeclaration::new(
                token.clone(),
                crate::capability::ThemeTokenFamily::surface(),
                worth_ui_dsl::UiThemeValueKind::CornerRadii,
                crate::capability::ThemeTokenSource::application(),
                crate::capability::UiThemeSlotDisclosure::Public,
                crate::capability::UiThemeSlotSuccessorCompatibility::ExactMeaning,
                None,
            ),
            crate::capability::UiThemeSlotDeclaration::new(
                background.clone(),
                crate::capability::ThemeTokenFamily::surface(),
                worth_ui_dsl::UiThemeValueKind::Color,
                crate::capability::ThemeTokenSource::application(),
                crate::capability::UiThemeSlotDisclosure::Public,
                crate::capability::UiThemeSlotSuccessorCompatibility::ExactMeaning,
                None,
            ),
        ],
    )
    .unwrap();
    let definition = |identity: &str, corners| {
        crate::capability::UiThemeDefinition::admit(
            crate::capability::UiThemeDefinitionIdentity::new(identity).unwrap(),
            1,
            &catalog,
            [
                (token.clone(), radius_value_from(corners)),
                (
                    background.clone(),
                    worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels([
                        17, 34, 51, 255,
                    ])),
                ),
            ],
        )
        .unwrap()
    };
    let definitions = vec![
        definition("theme.appearance.radii", [i32::MAX; 4]),
        definition("theme.appearance.radii-max-minus-1", [i32::MAX - 1; 4]),
        definition("theme.appearance.radii-max-minus-2", [i32::MAX - 2; 4]),
        definition("theme.appearance.radii-1", [1; 4]),
        definition("theme.appearance.radii-2", [2; 4]),
        definition("theme.appearance.radii-3", [3; 4]),
        definition("theme.appearance.radii-4", [4; 4]),
    ];
    crate::capability::FrozenAppearanceThemeCapabilities::admit(
        catalog,
        crate::capability::UiThemeDefinitionIdentity::new("theme.appearance.radii").unwrap(),
        definitions,
    )
    .unwrap()
}
