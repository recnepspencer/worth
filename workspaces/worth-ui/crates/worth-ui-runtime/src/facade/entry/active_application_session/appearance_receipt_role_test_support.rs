use crate::runtime::tests::appearance_component_session_test_support as support;

pub(super) const SWITCHED_SLOT: &str = "theme.appearance_consumer.switched";
const RADIUS_BACKGROUND_TOKEN: &str = "theme.appearance_radius.background";

pub(super) fn staged_test_host_profile(
    identity: &str,
    version: u16,
) -> worth_ui_host_contract::UiHostAppearanceProfileContract {
    worth_ui_host_contract::UiHostAppearanceProfileContract::admit(
        identity,
        version,
        [
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::SurfaceFill,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::SurfaceBorder,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::CornerRadii,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::Outline,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::TextRangeForeground,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::PortalSurface,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::Backdrop,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::OverlayOrder,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::PointerAffordance,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::Damage,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::Clip,
        ],
        Some(worth_ui_host_contract::UiHostPrimaryPointerKind::Mouse),
        worth_ui_host_contract::UiHostAppearanceGeometryQualification::admit([
            worth_ui_host_contract::UiHostAppearanceScaleGeometryQualification::new(
                1_000,
                1,
                worth_ui_host_contract::UiAppearanceLogicalLength::new(1_000).unwrap(),
                worth_ui_host_contract::UiHostAppearanceGeometryQualificationBasis::AnalyticSignedDistancePixelCenter,
            ),
        ])
        .expect("the staged test geometry qualification must admit"),
    )
    .expect("the explicit staged test host profile must admit")
}

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
    crate::capability::FrozenAppearanceThemeCapabilities::admit(
        catalog,
        crate::capability::UiThemeDefinitionIdentity::new("theme.appearance.receipts").unwrap(),
        vec![definition],
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
    let definition = crate::capability::UiThemeDefinition::admit(
        crate::capability::UiThemeDefinitionIdentity::new("theme.appearance.radii").unwrap(),
        1,
        &catalog,
        [
            (token, radius_value(i32::MAX)),
            (
                background,
                worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels([
                    17, 34, 51, 255,
                ])),
            ),
        ],
    )
    .unwrap();
    crate::capability::FrozenAppearanceThemeCapabilities::admit(
        catalog,
        crate::capability::UiThemeDefinitionIdentity::new("theme.appearance.radii").unwrap(),
        vec![definition],
    )
    .unwrap()
}

fn radius_partition(
    slot: &str,
    aspect: worth_ui_dsl::UiAppearanceAspect,
) -> worth_ui_dsl::UiAppearanceDecisionPartition {
    worth_ui_dsl::UiAppearancePartitionAuthoring::new([
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
    .unwrap()
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
    session
        .admit_application_theme_values(&[change])
        .expect("receipt theme update should commit");
}

pub(super) fn update_radius_at_revision(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    corners: [i32; 4],
    expected_revision: u64,
) {
    let token = crate::capability::ThemeTokenId::new(support::APPEARANCE_TOKEN).unwrap();
    let value = crate::capability::ThemeTokenValue::typed(radius_value_from(corners));
    let change = super::super::super::UiNativeThemeTokenValueChange::successor(
        token,
        expected_revision,
        value,
    )
    .unwrap();
    session
        .admit_application_theme_values(&[change])
        .expect("receipt radius update should commit");
}

pub(super) fn radius_value(base: i32) -> worth_ui_dsl::UiThemeValue {
    radius_value_from([base, base, base, base])
}

pub(super) fn radius_value_from(corners: [i32; 4]) -> worth_ui_dsl::UiThemeValue {
    let lengths = corners.map(worth_ui_dsl::UiLogicalLength::new);
    worth_ui_dsl::UiThemeValue::CornerRadii(
        worth_ui_dsl::UiThemeCornerRadii::new(lengths[0], lengths[1], lengths[2], lengths[3])
            .unwrap(),
    )
}

fn theme_value_kind(aspect: worth_ui_dsl::UiAppearanceAspect) -> worth_ui_dsl::UiThemeValueKind {
    match aspect {
        worth_ui_dsl::UiAppearanceAspect::Background
        | worth_ui_dsl::UiAppearanceAspect::Foreground => worth_ui_dsl::UiThemeValueKind::Color,
        worth_ui_dsl::UiAppearanceAspect::Border => worth_ui_dsl::UiThemeValueKind::SolidStroke,
        worth_ui_dsl::UiAppearanceAspect::Radius => worth_ui_dsl::UiThemeValueKind::CornerRadii,
        worth_ui_dsl::UiAppearanceAspect::Opacity => worth_ui_dsl::UiThemeValueKind::Opacity,
        worth_ui_dsl::UiAppearanceAspect::Outline => worth_ui_dsl::UiThemeValueKind::SolidOutline,
    }
}

fn initial_theme_color(hex: &str) -> worth_ui_dsl::UiThemeColor {
    let channels = match hex {
        "#405060" => [64, 80, 96, 255],
        "#112233" => [17, 34, 51, 255],
        _ => panic!("receipt fixture uses a declared theme color"),
    };
    worth_ui_dsl::UiThemeColor::from_channels(channels)
}
