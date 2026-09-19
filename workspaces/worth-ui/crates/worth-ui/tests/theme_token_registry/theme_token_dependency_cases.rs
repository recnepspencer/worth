use worth_ui::facade::{
    app::WorthUi,
    declaration::{
        ComponentAccessibilitySupport, ComponentChildPolicy, ComponentDescriptor,
        ComponentExecutionLane, ComponentFocusSupport, ComponentId, ComponentPropSchema,
        ComponentStateOwnership,
    },
    diagnostics::CapabilityDiagnosticCode,
};

use super::theme_token_assertions::{assert_diagnostic_codes, assert_registered_theme_token_ids};
use super::theme_token_fixtures::{
    alias_theme_token, color_theme_token, platform_color_theme_token, theme_token_id,
};

#[test]
fn missing_token_dependency_is_rejected() {
    let report = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_theme_token(alias_theme_token("theme.text.alias", "theme.text.missing"))
        .freeze_with_registration_report();

    assert_diagnostic_codes(&report, &[CapabilityDiagnosticCode::MissingDependency]);
    assert!(report.accepted_snapshot().theme_tokens().is_empty());
}

#[test]
fn component_theme_token_dependency_resolves_when_token_is_registered() {
    let app = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_theme_token(platform_color_theme_token("theme.text.primary", "#101820"))
        .register_component(component_referencing_token(
            "component.label",
            "theme.text.primary",
        ))
        .freeze()
        .expect("application preparation should succeed");

    assert_eq!(app.capabilities().components().len(), 1);
    assert_registered_theme_token_ids(app.capabilities().theme_tokens(), &["theme.text.primary"]);
}

#[test]
fn component_theme_token_dependency_is_rejected_when_token_is_missing() {
    let report = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_component(component_referencing_token(
            "component.label",
            "theme.text.missing",
        ))
        .freeze_with_registration_report();

    assert_diagnostic_codes(&report, &[CapabilityDiagnosticCode::MissingDependency]);
    assert!(report.accepted_snapshot().components().is_empty());
}

#[test]
fn alias_to_rejected_cycle_does_not_enter_frozen_registry() {
    let report = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_theme_token(alias_theme_token("theme.a", "theme.b"))
        .register_theme_token(alias_theme_token("theme.b", "theme.a"))
        .register_theme_token(alias_theme_token("theme.c", "theme.a"))
        .freeze_with_registration_report();

    assert!(report.accepted_snapshot().theme_tokens().is_empty());
}

#[test]
fn rejected_alias_cycle_does_not_poison_valid_theme_token() {
    let report = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_theme_token(color_theme_token("theme.text.valid", "#101820"))
        .register_theme_token(alias_theme_token("theme.a", "theme.b"))
        .register_theme_token(alias_theme_token("theme.b", "theme.a"))
        .freeze_with_registration_report();

    assert_registered_theme_token_ids(
        report.accepted_snapshot().theme_tokens(),
        &["theme.text.valid"],
    );
}

fn component_referencing_token(component_id: &str, token_id: &str) -> ComponentDescriptor {
    ComponentDescriptor::new(
        ComponentId::new(component_id).unwrap(),
        ComponentPropSchema::named("component.label.props"),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
    .with_accessibility(ComponentAccessibilitySupport::semantic())
    .with_focus(ComponentFocusSupport::not_focusable())
    .with_execution_lane(ComponentExecutionLane::Passive)
    .with_semantic_text(
        worth_ui::facade::declaration::ComponentSemanticTextContract::body_default(
            theme_token_id(token_id),
            0,
        ),
    )
}
