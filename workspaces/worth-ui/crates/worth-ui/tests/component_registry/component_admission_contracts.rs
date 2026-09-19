use worth_ui::facade::{
    app::WorthUi,
    declaration::{
        ComponentChildPolicy, ComponentDescriptor, ComponentExecutionLane, ComponentPropSchema,
        ComponentStateOwnership, ThemeTokenId,
    },
    diagnostics::CapabilityDiagnosticCode,
};

use super::component_registry_assertions::{
    assert_dependency_diagnostics, assert_diagnostic_codes, assert_diagnostic_codes_and_identities,
    assert_registered_component_ids,
};
use super::component_registry_fixtures::{command_id, component_descriptor, component_id};

#[test]
fn duplicate_component_id_rejected_before_snapshot_freeze() {
    let report = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_component(component_descriptor("workspace.component.editor"))
        .register_component(component_descriptor("workspace.component.editor"))
        .freeze_with_registration_report();

    assert!(report.has_errors());
    assert!(report.accepted_snapshot().components().is_empty());
    assert_diagnostic_codes(
        report.registration_diagnostics(),
        &[
            CapabilityDiagnosticCode::DuplicateCapabilityId,
            CapabilityDiagnosticCode::DuplicateCapabilityId,
        ],
    );
}

#[test]
fn duplicate_component_id_rejects_only_the_duplicate_identity() {
    let report = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_component(component_descriptor("workspace.component.valid"))
        .register_component(component_descriptor("workspace.component.editor"))
        .register_component(component_descriptor("workspace.component.editor"))
        .freeze_with_registration_report();

    assert!(report.has_errors());
    assert_registered_component_ids(
        report.accepted_snapshot().components(),
        &["workspace.component.valid"],
    );
    assert_diagnostic_codes_and_identities(
        report.registration_diagnostics(),
        &[
            (
                CapabilityDiagnosticCode::DuplicateCapabilityId,
                "workspace.component.editor",
            ),
            (
                CapabilityDiagnosticCode::DuplicateCapabilityId,
                "workspace.component.editor",
            ),
        ],
    );
}

#[test]
fn component_with_untyped_props_rejected() {
    let report = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_component(ComponentDescriptor::new(
            component_id("workspace.component.editor"),
            ComponentPropSchema::untyped_for_diagnostics("workspace.editor.props"),
            ComponentChildPolicy::no_children(),
            ComponentStateOwnership::runtime_owned(),
        ))
        .freeze_with_registration_report();

    assert!(report.has_errors());
    assert!(report.accepted_snapshot().components().is_empty());
    assert_diagnostic_codes(
        report.registration_diagnostics(),
        &[CapabilityDiagnosticCode::MissingComponentPropSchema],
    );
}

#[test]
fn component_missing_state_ownership_rejected() {
    let report = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_component(
            ComponentDescriptor::without_state_ownership_for_diagnostics(
                component_id("workspace.component.editor"),
                ComponentPropSchema::named("workspace.editor.props"),
                ComponentChildPolicy::no_children(),
            ),
        )
        .freeze_with_registration_report();

    assert!(report.has_errors());
    assert!(report.accepted_snapshot().components().is_empty());
    assert_diagnostic_codes(
        report.registration_diagnostics(),
        &[CapabilityDiagnosticCode::MissingComponentStateOwnership],
    );
}

#[test]
fn component_with_illegal_child_policy_rejected() {
    let report = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_component(ComponentDescriptor::new(
            component_id("workspace.component.editor"),
            ComponentPropSchema::named("workspace.editor.props"),
            ComponentChildPolicy::shell_layout_authority_for_diagnostics(),
            ComponentStateOwnership::runtime_owned(),
        ))
        .freeze_with_registration_report();

    assert!(report.has_errors());
    assert!(report.accepted_snapshot().components().is_empty());
    assert_diagnostic_codes(
        report.registration_diagnostics(),
        &[CapabilityDiagnosticCode::IllegalComponentChildPolicy],
    );
}

#[test]
fn canvas_lane_without_canvas_contract_is_rejected_at_capability_freeze() {
    let report = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_component(
            component_descriptor("workspace.component.canvas")
                .with_execution_lane(ComponentExecutionLane::CanvasSpatial),
        )
        .freeze_with_registration_report();

    assert!(report.has_errors());
    assert!(report.accepted_snapshot().components().is_empty());
    assert_diagnostic_codes(
        report.registration_diagnostics(),
        &[CapabilityDiagnosticCode::MissingComponentCanvasSpatialContract],
    );
}

#[test]
fn realtime_lane_without_frame_policy_is_rejected_at_capability_freeze() {
    let report = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_component(
            component_descriptor("workspace.component.hud")
                .with_execution_lane(ComponentExecutionLane::RealtimeOverlay),
        )
        .freeze_with_registration_report();

    assert!(report.has_errors());
    assert!(report.accepted_snapshot().components().is_empty());
    assert_diagnostic_codes(
        report.registration_diagnostics(),
        &[CapabilityDiagnosticCode::MissingComponentRealtimeOverlayContract],
    );
}

#[test]
fn component_references_missing_theme_token_rejected() {
    let missing_token_id =
        ThemeTokenId::new("workspace.theme_token.accent").expect("valid token id");
    let report = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_component(
            component_descriptor("workspace.component.editor").with_semantic_text(
                worth_ui::facade::declaration::ComponentSemanticTextContract::body_default(
                    missing_token_id,
                    0,
                ),
            ),
        )
        .freeze_with_registration_report();

    assert!(report.has_errors());
    assert!(report.accepted_snapshot().components().is_empty());
    assert_dependency_diagnostics(
        report.registration_diagnostics(),
        &[(
            CapabilityDiagnosticCode::MissingDependency,
            "workspace.component.editor",
            "theme_token",
            "workspace.theme_token.accent",
        )],
    );
}

#[test]
fn component_missing_theme_token_does_not_poison_valid_component() {
    let report = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_component(component_descriptor("workspace.component.valid"))
        .register_component(
            component_descriptor("workspace.component.editor").with_semantic_text(
                worth_ui::facade::declaration::ComponentSemanticTextContract::body_default(
                    ThemeTokenId::new("workspace.theme_token.accent").expect("valid token id"),
                    0,
                ),
            ),
        )
        .freeze_with_registration_report();

    assert!(report.has_errors());
    assert_registered_component_ids(
        report.accepted_snapshot().components(),
        &["workspace.component.valid"],
    );
    assert_diagnostic_codes_and_identities(
        report.registration_diagnostics(),
        &[(
            CapabilityDiagnosticCode::MissingDependency,
            "workspace.component.editor",
        )],
    );
}

#[test]
fn component_references_missing_command_binding_rejected() {
    let report = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_component(
            component_descriptor("workspace.component.editor")
                .with_command_binding_slot(command_id("workspace.command.open")),
        )
        .freeze_with_registration_report();

    assert!(report.has_errors());
    assert!(report.accepted_snapshot().components().is_empty());
    assert_dependency_diagnostics(
        report.registration_diagnostics(),
        &[(
            CapabilityDiagnosticCode::MissingDependency,
            "workspace.component.editor",
            "command",
            "workspace.command.open",
        )],
    );
}

#[test]
fn invalid_component_descriptor_does_not_poison_valid_component() {
    let report = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_component(component_descriptor("workspace.component.valid"))
        .register_component(ComponentDescriptor::without_prop_schema_for_diagnostics(
            component_id("workspace.component.invalid"),
            ComponentChildPolicy::no_children(),
            ComponentStateOwnership::runtime_owned(),
        ))
        .freeze_with_registration_report();

    assert!(report.has_errors());
    assert_registered_component_ids(
        report.accepted_snapshot().components(),
        &["workspace.component.valid"],
    );
    assert_diagnostic_codes_and_identities(
        report.registration_diagnostics(),
        &[(
            CapabilityDiagnosticCode::MissingComponentPropSchema,
            "workspace.component.invalid",
        )],
    );
}

#[test]
fn component_descriptor_reports_multiple_independent_violations() {
    let report = WorthUi::app()
        .with_change_profile(worth_ui_runtime::facade::rebind::UiChangeProfile::platform_pulse())
        .register_component(ComponentDescriptor::new(
            component_id("workspace.component.invalid"),
            ComponentPropSchema::untyped_for_diagnostics("workspace.invalid.props"),
            ComponentChildPolicy::shell_layout_authority_for_diagnostics(),
            ComponentStateOwnership::runtime_owned(),
        ))
        .freeze_with_registration_report();

    assert!(report.has_errors());
    assert!(report.accepted_snapshot().components().is_empty());
    assert_diagnostic_codes_and_identities(
        report.registration_diagnostics(),
        &[
            (
                CapabilityDiagnosticCode::MissingComponentPropSchema,
                "workspace.component.invalid",
            ),
            (
                CapabilityDiagnosticCode::IllegalComponentChildPolicy,
                "workspace.component.invalid",
            ),
        ],
    );
}
