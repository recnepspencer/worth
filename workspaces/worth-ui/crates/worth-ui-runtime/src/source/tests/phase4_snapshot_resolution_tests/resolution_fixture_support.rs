use crate::capability::{
    CapabilitySnapshot, CapabilitySnapshotFreezeInput, CapabilitySupportCatalog,
    ComponentChildPolicy, ComponentDescriptor, ComponentId, ComponentPropSchema,
    ComponentStateOwnership, RegisteredCapabilitySet, RegistrationCandidate, SurfaceDescriptor,
    SurfaceId, SurfaceKind, SurfacePlacementClass, SurfaceStateClass, ThemeTokenAlias,
    ThemeTokenDescriptor, ThemeTokenFamily, ThemeTokenId, ThemeTokenSource, ThemeTokenValue,
    UiThemeColor, ViewBindingDescriptor, ViewBindingFamily, ViewBindingId, COMPONENT_FAMILY_NAME,
    SURFACE_FAMILY_NAME, THEME_TOKEN_FAMILY_NAME, VIEW_BINDING_FAMILY_NAME,
};
use crate::facade::{WorthUi, WorthUiApp};
use crate::source::{WorthUiResolutionDiagnosticCode, WorthUiResolutionReport};
use worth_ui_dsl::{
    WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule,
    WorthUiSealedSemanticPackage,
};

pub(super) fn standard_artifact_input() -> WorthUiSealedSemanticPackage {
    crate::source::test_compilation::compile_rust_authored(
        &WorthUiRustAuthoredArtifactInput::from_modules([
            WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
                .with_import("app/panels/inspector.wui")
                .with_component("workspace.component.dashboard")
                .with_surface("workspace.surface.inspector")
                .with_binding("workspace.view_binding.selection")
                .with_token("theme.text.default", "theme.text.primary"),
            WorthUiRustAuthoredArtifactInputModule::new("app/panels/inspector.wui")
                .with_component("workspace.component.inspector_panel"),
        ]),
    )
}

pub(super) fn admitted_app() -> WorthUiApp {
    let definition = worth_ui_query_binding::WorthUiQueryViewDefinition::measurement_snapshot(
        "workspace.view_binding.selection",
    )
    .expect("query definition should admit");

    WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .register_component(component_descriptor("workspace.component.dashboard"))
        .register_component(component_descriptor("workspace.component.inspector_panel"))
        .register_view_binding(ViewBindingDescriptor::from_definition(
            ViewBindingId::new("workspace.view_binding.selection").unwrap(),
            ViewBindingFamily::collection(),
            definition,
        ))
        .register_surface(
            SurfaceDescriptor::new(
                SurfaceId::new("workspace.surface.inspector").unwrap(),
                SurfaceKind::primary_content(),
                ComponentId::new("workspace.component.dashboard").unwrap(),
                SurfacePlacementClass::primary_region(),
                SurfaceStateClass::restorable(),
            )
            .with_view_binding(ViewBindingId::new("workspace.view_binding.selection").unwrap()),
        )
        .register_theme_token(ThemeTokenDescriptor::define(
            ThemeTokenId::new("theme.text.primary").unwrap(),
            ThemeTokenFamily::text(),
            ThemeTokenSource::application(),
            ThemeTokenValue::color(UiThemeColor::parse("#101820").unwrap()),
        ))
        .register_theme_token(ThemeTokenDescriptor::alias(
            ThemeTokenId::new("theme.text.default").unwrap(),
            ThemeTokenFamily::text(),
            ThemeTokenSource::application(),
            ThemeTokenAlias::to(ThemeTokenId::new("theme.text.primary").unwrap()),
        ))
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("application preparation should succeed")
}

pub(in crate::source::tests) fn empty_snapshot() -> WorthUiApp {
    WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("application preparation should succeed")
}

pub(super) fn component_descriptor(identity: &str) -> ComponentDescriptor {
    ComponentDescriptor::new(
        ComponentId::new(identity).unwrap(),
        ComponentPropSchema::named("workspace.props"),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
}

pub(in crate::source::tests) fn snapshot_with_support_catalog(
    base: &CapabilitySnapshot,
    support_catalog: CapabilitySupportCatalog,
) -> CapabilitySnapshot {
    CapabilitySnapshot::from_freeze_input(CapabilitySnapshotFreezeInput {
        registered_capabilities: clone_registered_capabilities(base),
        appearance_roles: base.appearance_roles().clone(),
        appearance_themes: base.appearance_themes().cloned(),
        commands: base.commands().clone(),
        command_projections: base.command_projections().clone(),
        components: base.components().clone(),
        icons: base.icons().clone(),
        intent_definitions: base.intent_definitions().clone(),
        surfaces: base.surfaces().clone(),
        mosaic_regions: base.mosaic_regions().clone(),
        mosaic_placement_policies: base.mosaic_placement_policies().clone(),
        mosaic_sizing_contracts: base.mosaic_sizing_contracts().clone(),
        mosaic_state_slots: base.mosaic_state_slots().clone(),
        native_capabilities: base.native_capabilities().clone(),
        plugin_slots: base.plugin_slots().clone(),
        view_bindings: base.view_bindings().clone(),
        runtime_outcome_projections: base.runtime_outcome_projections().clone(),
        settings: base.settings().clone(),
        task_presentations: base.task_presentations().clone(),
        theme_tokens: base.theme_tokens().clone(),
        support_catalog,
    })
}

pub(super) fn support_catalog_with_extra<const N: usize>(
    extra: [RegistrationCandidate; N],
) -> CapabilitySupportCatalog {
    let mut candidates = vec![
        RegistrationCandidate::admitted(COMPONENT_FAMILY_NAME, "workspace.component.dashboard"),
        RegistrationCandidate::admitted(
            COMPONENT_FAMILY_NAME,
            "workspace.component.inspector_panel",
        ),
        RegistrationCandidate::admitted(SURFACE_FAMILY_NAME, "workspace.surface.inspector"),
        RegistrationCandidate::admitted(
            VIEW_BINDING_FAMILY_NAME,
            "workspace.view_binding.selection",
        ),
        RegistrationCandidate::admitted(THEME_TOKEN_FAMILY_NAME, "theme.text.primary"),
        RegistrationCandidate::admitted(THEME_TOKEN_FAMILY_NAME, "theme.text.default"),
    ];
    candidates.extend(extra);
    CapabilitySupportCatalog::from_registration_candidates(&candidates)
}

pub(in crate::source::tests) fn diagnostic_codes(
    report: &WorthUiResolutionReport,
) -> Vec<WorthUiResolutionDiagnosticCode> {
    report
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code())
        .collect()
}

fn clone_registered_capabilities(snapshot: &CapabilitySnapshot) -> RegisteredCapabilitySet {
    let registered = snapshot.registered_capabilities();
    RegisteredCapabilitySet::from_counts(
        registered.registered_family_count(),
        registered.total_width(),
    )
}
