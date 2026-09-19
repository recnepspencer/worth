use super::*;

pub(in crate::facade::entry::active_application_session) fn radius_partition(
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

pub(in crate::facade::entry::active_application_session) fn update_theme(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    hex: &str,
) {
    update_theme_at_revision(session, hex, 0);
}

pub(in crate::facade::entry::active_application_session) fn update_theme_at_revision(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    hex: &str,
    _expected_revision: u64,
) {
    let token = crate::capability::ThemeTokenId::new(support::APPEARANCE_TOKEN).unwrap();
    let identity = match hex.to_ascii_lowercase().as_str() {
        "#405060" => "theme.appearance.receipts-405060",
        value => panic!("missing receipt theme definition for {value}"),
    };
    support::replace_appearance_theme_definition_for_test(session, identity, &token);
}

pub(in crate::facade::entry::active_application_session) fn update_radius_at_revision(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    corners: [i32; 4],
    _expected_revision: u64,
) {
    let token = crate::capability::ThemeTokenId::new(support::APPEARANCE_TOKEN).unwrap();
    let identity = radius_definition_identity(corners);
    support::replace_appearance_theme_definition_for_test(session, identity, &token);
}

pub(in crate::facade::entry::active_application_session) fn radius_value_from(
    corners: [i32; 4],
) -> worth_ui_dsl::UiThemeValue {
    let lengths = corners.map(worth_ui_dsl::UiLogicalLength::new);
    worth_ui_dsl::UiThemeValue::CornerRadii(
        worth_ui_dsl::UiThemeCornerRadii::new(lengths[0], lengths[1], lengths[2], lengths[3])
            .unwrap(),
    )
}

fn radius_definition_identity(corners: [i32; 4]) -> &'static str {
    match corners {
        value if value == [i32::MAX; 4] => "theme.appearance.radii",
        value if value == [i32::MAX - 1; 4] => "theme.appearance.radii-max-minus-1",
        value if value == [i32::MAX - 2; 4] => "theme.appearance.radii-max-minus-2",
        [1, 1, 1, 1] => "theme.appearance.radii-1",
        [2, 2, 2, 2] => "theme.appearance.radii-2",
        [3, 3, 3, 3] => "theme.appearance.radii-3",
        [4, 4, 4, 4] => "theme.appearance.radii-4",
        value => panic!("missing radius theme definition for {value:?}"),
    }
}

pub(in crate::facade::entry::active_application_session) fn theme_value_kind(
    aspect: worth_ui_dsl::UiAppearanceAspect,
) -> worth_ui_dsl::UiThemeValueKind {
    match aspect {
        worth_ui_dsl::UiAppearanceAspect::Background
        | worth_ui_dsl::UiAppearanceAspect::Foreground => worth_ui_dsl::UiThemeValueKind::Color,
        worth_ui_dsl::UiAppearanceAspect::Border => worth_ui_dsl::UiThemeValueKind::SolidStroke,
        worth_ui_dsl::UiAppearanceAspect::Radius => worth_ui_dsl::UiThemeValueKind::CornerRadii,
        worth_ui_dsl::UiAppearanceAspect::Opacity => worth_ui_dsl::UiThemeValueKind::Opacity,
        worth_ui_dsl::UiAppearanceAspect::Outline => worth_ui_dsl::UiThemeValueKind::SolidOutline,
    }
}

pub(in crate::facade::entry::active_application_session) fn initial_theme_color(
    hex: &str,
) -> worth_ui_dsl::UiThemeColor {
    let channels = match hex {
        "#405060" => [64, 80, 96, 255],
        "#112233" => [17, 34, 51, 255],
        _ => panic!("receipt fixture uses a declared theme color"),
    };
    worth_ui_dsl::UiThemeColor::from_channels(channels)
}
