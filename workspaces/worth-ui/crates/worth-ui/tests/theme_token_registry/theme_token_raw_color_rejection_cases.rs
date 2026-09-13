use worth_ui::facade::{
    app::WorthUi,
    declaration::{
        RawColorOutsideTokenDefinition, ThemeTokenDescriptor, ThemeTokenFamily, ThemeTokenSource,
        UiThemeColor, UiThemeColorParseDenial,
    },
    diagnostics::CapabilityDiagnosticCode,
};

use super::theme_token_assertions::assert_diagnostic_codes;
use super::theme_token_fixtures::theme_token_id;

#[test]
fn raw_color_outside_token_definition_is_rejected() {
    let report = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_theme_token(
            ThemeTokenDescriptor::raw_color_outside_token_definition_for_diagnostics(
                theme_token_id("theme.raw.leak"),
                RawColorOutsideTokenDefinition::new("#ffffff"),
            ),
        )
        .freeze_with_registration_report();

    assert_diagnostic_codes(
        &report,
        &[
            CapabilityDiagnosticCode::MissingThemeTokenDefinition,
            CapabilityDiagnosticCode::RawColorOutsideThemeTokenDefinition,
        ],
    );
    assert!(report.accepted_snapshot().theme_tokens().is_empty());
}

#[test]
fn theme_token_without_value_or_alias_is_rejected() {
    let report = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_theme_token(ThemeTokenDescriptor::missing_definition_for_diagnostics(
            theme_token_id("theme.text.missing_definition"),
            ThemeTokenFamily::text(),
            ThemeTokenSource::application(),
        ))
        .freeze_with_registration_report();

    assert_diagnostic_codes(
        &report,
        &[CapabilityDiagnosticCode::MissingThemeTokenDefinition],
    );
    assert!(report.accepted_snapshot().theme_tokens().is_empty());
}

#[test]
fn invalid_theme_color_literal_is_rejected() {
    assert_eq!(
        UiThemeColor::parse("white"),
        Err(UiThemeColorParseDenial::MissingHash)
    );
}
