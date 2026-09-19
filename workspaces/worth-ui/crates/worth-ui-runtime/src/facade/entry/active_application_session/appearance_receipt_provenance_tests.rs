use super::support;

#[test]
fn radius_value_kind_mismatch_denies_theme_admission_before_resolution() {
    let themes = super::role_support::radius_theme_bundle();
    let token = crate::capability::ThemeTokenId::new(support::APPEARANCE_TOKEN).unwrap();
    let definition = crate::capability::UiThemeDefinition::admit(
        crate::capability::UiThemeDefinitionIdentity::new("theme.appearance.radii-invalid-kind")
            .unwrap(),
        1,
        themes.catalog(),
        themes
            .get(themes.initial_definition_identity())
            .unwrap()
            .values()
            .map(|(slot, value)| {
                (
                    slot.clone(),
                    if slot == &token {
                        worth_ui_dsl::UiThemeValue::Color(
                            worth_ui_dsl::UiThemeColor::from_channels([9, 9, 9, 255]),
                        )
                    } else {
                        *value
                    },
                )
            }),
    );
    assert_eq!(
        definition,
        Err(crate::capability::UiThemeDefinitionDenial::ValueKindMismatch(token))
    );
}
