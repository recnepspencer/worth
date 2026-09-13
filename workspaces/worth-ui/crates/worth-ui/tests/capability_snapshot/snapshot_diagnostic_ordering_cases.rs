use worth_ui::facade::{app::WorthUi, diagnostics::CapabilityDiagnosticCode};

use super::snapshot_assertions::{diagnostic_codes, diagnostic_topology};
use super::snapshot_fixtures::{command, command_id, component, theme_token_id};

#[test]
fn snapshot_diagnostics_stable_under_invalid_input_permutation() {
    let component_with_missing_references = component("component.editor")
        .with_command_binding_slot(command_id("command.missing"))
        .with_semantic_text(
            worth_ui::facade::declaration::ComponentSemanticTextContract::body_default(
                theme_token_id("theme.missing"),
                0,
            ),
        );
    let duplicate_command = command("command.duplicate", "Duplicate");

    let first = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_component(component_with_missing_references.clone())
        .register_command(duplicate_command.clone())
        .register_command(duplicate_command.clone())
        .freeze_with_registration_report();
    let second = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_command(duplicate_command.clone())
        .register_command(duplicate_command)
        .register_component(component_with_missing_references)
        .freeze_with_registration_report();

    assert_eq!(
        diagnostic_codes(first.registration_diagnostics()),
        diagnostic_codes(second.registration_diagnostics())
    );
    assert_eq!(
        diagnostic_codes(first.registration_diagnostics()),
        vec![
            CapabilityDiagnosticCode::DuplicateCapabilityId,
            CapabilityDiagnosticCode::DuplicateCapabilityId,
            CapabilityDiagnosticCode::MissingDependency,
            CapabilityDiagnosticCode::MissingDependency,
        ]
    );
    assert_eq!(
        diagnostic_topology(first.registration_diagnostics()),
        vec![
            (
                CapabilityDiagnosticCode::DuplicateCapabilityId,
                Some("command".to_owned()),
                Some("command.duplicate".to_owned()),
                None,
                None,
            ),
            (
                CapabilityDiagnosticCode::DuplicateCapabilityId,
                Some("command".to_owned()),
                Some("command.duplicate".to_owned()),
                None,
                None,
            ),
            (
                CapabilityDiagnosticCode::MissingDependency,
                Some("component".to_owned()),
                Some("component.editor".to_owned()),
                Some("command".to_owned()),
                Some("command.missing".to_owned()),
            ),
            (
                CapabilityDiagnosticCode::MissingDependency,
                Some("component".to_owned()),
                Some("component.editor".to_owned()),
                Some("theme_token".to_owned()),
                Some("theme.missing".to_owned()),
            ),
        ]
    );
    assert!(first
        .accepted_snapshot()
        .validation_summary()
        .lowering_is_admissible());
    assert!(second
        .accepted_snapshot()
        .validation_summary()
        .lowering_is_admissible());
}
