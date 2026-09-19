use std::{collections::BTreeMap, path::Path};
use worth_ui_dsl::{
    WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule, WorthUiSourceModuleId,
};

use crate::capability::{
    CommandDescriptor, CommandId, ComponentChildPolicy, ComponentDescriptor, ComponentId,
    ComponentPropSchema, ComponentStateOwnership, SurfaceDescriptor, SurfaceId, SurfaceKind,
    SurfacePlacementClass, SurfaceStateClass, ThemeTokenDescriptor, ThemeTokenFamily, ThemeTokenId,
    ThemeTokenSource, ThemeTokenValue, UiThemeColor,
};
use crate::facade::{WorthUi, WorthUiApp};
use crate::runtime::replacement::candidate::rust_authored_replacement_candidate;
use crate::runtime::{
    WorthUiAdmittedReplacementCandidate, WorthUiCandidateAdmission, WorthUiReplacementCause,
    WorthUiRuntime, WorthUiRuntimeLaunch,
};
use crate::source::{
    WorthUiArtifact, WorthUiArtifactHandle, WorthUiArtifactIdentitySeed,
    WorthUiArtifactImportHandle, WorthUiArtifactImportNode, WorthUiArtifactModule,
    WorthUiArtifactNode, WorthUiBindingSemanticsLowerer, WorthUiCanonicalArtifactAssembler,
    WorthUiDurableStateEligibility, WorthUiDurableStateIneligibilityReason,
    WorthUiIdentitySeedLowerer, WorthUiStructuralLegalityLowerer,
};

pub(super) fn admitted_candidate(
    app: &WorthUiApp,
    runtime: &WorthUiRuntime,
    artifact: WorthUiArtifact,
) -> WorthUiAdmittedReplacementCandidate {
    let candidate = rust_authored_replacement_candidate(
        artifact,
        app.capabilities().digest(),
        WorthUiReplacementCause::rust_authored_input_change(13),
    )
    .expect("candidate seals");
    WorthUiCandidateAdmission::for_active_basis(runtime.replacement_admission_basis())
        .admit(candidate)
        .expect("candidate admits")
}

pub(super) fn launch_runtime(
    app: &WorthUiApp,
    artifact: WorthUiArtifact,
) -> crate::runtime::WorthUiRuntimeFrameworkLoop {
    app.launch_runtime(WorthUiRuntimeLaunch::from_canonical_artifact(artifact))
        .expect("runtime launches")
}

pub(super) fn artifact_from_modules<const N: usize>(
    app: &WorthUiApp,
    modules: [WorthUiRustAuthoredArtifactInputModule; N],
) -> WorthUiArtifact {
    let artifact_input = crate::source::test_compilation::compile_rust_authored(
        &WorthUiRustAuthoredArtifactInput::from_modules(modules),
    );
    let snapshot = app.capabilities();
    let resolved = crate::source::WorthUiArtifactInputResolver::resolve(&artifact_input, snapshot)
        .expect("artifact input resolves");
    let structured =
        WorthUiStructuralLegalityLowerer::lower(&resolved, snapshot).expect("structure lowers");
    let bound =
        WorthUiBindingSemanticsLowerer::lower(&structured, snapshot).expect("semantics lower");
    let identity_seeded = WorthUiIdentitySeedLowerer::lower(&bound)
        .expect("identity seeds lower")
        .0;
    WorthUiCanonicalArtifactAssembler::assemble(&identity_seeded)
        .expect("canonical artifact assembles")
}

pub(super) fn impact_test_app() -> WorthUiApp {
    WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .register_command(CommandDescriptor::new(
            command_id("workspace.command.save"),
            "Save",
        ))
        .register_command(CommandDescriptor::new(
            command_id("workspace.command.open"),
            "Open",
        ))
        .register_component(component("workspace.component.dashboard"))
        .register_component(component("workspace.component.replacement"))
        .register_surface(surface(
            "workspace.surface.main",
            SurfacePlacementClass::primary_region(),
            None,
        ))
        .register_surface(surface(
            "workspace.surface.overlay",
            SurfacePlacementClass::overlay_layer(),
            None,
        ))
        .register_surface(surface(
            "workspace.surface.command_save",
            SurfacePlacementClass::primary_region(),
            Some("workspace.command.save"),
        ))
        .register_surface(surface(
            "workspace.surface.command_open",
            SurfacePlacementClass::primary_region(),
            Some("workspace.command.open"),
        ))
        .register_surface(surface(
            "workspace.surface.overlay_command_open",
            SurfacePlacementClass::overlay_layer(),
            Some("workspace.command.open"),
        ))
        .register_theme_token(theme_token("theme.text.primary", "#101820"))
        .register_theme_token(theme_token("theme.text.secondary", "#C7492A"))
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("application preparation should succeed")
}

pub(super) fn surface_module(surface_id: &str) -> WorthUiRustAuthoredArtifactInputModule {
    WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
        .with_surface_authored_identity(surface_id, "impact.surface")
}

pub(super) fn component_module(component_id: &str) -> WorthUiRustAuthoredArtifactInputModule {
    WorthUiRustAuthoredArtifactInputModule::new("app/main.wui").with_component(component_id)
}

pub(super) fn token_module(token_id: &str) -> WorthUiRustAuthoredArtifactInputModule {
    WorthUiRustAuthoredArtifactInputModule::new("app/main.wui").with_token_authored_identity(
        token_id,
        "impact.token",
        token_id,
    )
}

pub(super) fn import_artifact<const N: usize>(targets: [&str; N]) -> WorthUiArtifact {
    let module_id = module_id("app/main.wui");
    let nodes = targets
        .into_iter()
        .enumerate()
        .map(|(node_index, target)| import_node(&module_id, node_index, target))
        .collect::<Vec<_>>();
    artifact_from_module_map([(
        module_id.clone(),
        WorthUiArtifactModule::new(module_id, nodes),
    )])
}

pub(super) fn two_module_import_artifact() -> WorthUiArtifact {
    let first_module_id = module_id("app/main.wui");
    let second_module_id = module_id("app/other.wui");
    artifact_from_module_map([
        (
            first_module_id.clone(),
            WorthUiArtifactModule::new(
                first_module_id.clone(),
                vec![import_node(&first_module_id, 0, "app/panels/inspector.wui")],
            ),
        ),
        (
            second_module_id.clone(),
            WorthUiArtifactModule::new(
                second_module_id.clone(),
                vec![import_node(&second_module_id, 0, "app/panels/settings.wui")],
            ),
        ),
    ])
}

fn component(id: &str) -> ComponentDescriptor {
    ComponentDescriptor::new(
        ComponentId::new(id).expect("valid component id"),
        ComponentPropSchema::named("workspace.props"),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
}

fn surface(
    id: &str,
    placement_class: SurfacePlacementClass,
    command_slot: Option<&str>,
) -> SurfaceDescriptor {
    let descriptor = SurfaceDescriptor::new(
        SurfaceId::new(id).expect("valid surface id"),
        SurfaceKind::primary_content(),
        ComponentId::new("workspace.component.dashboard").expect("valid component id"),
        placement_class,
        SurfaceStateClass::restorable(),
    );
    match command_slot {
        Some(command_slot) => descriptor.with_command_slot(command_id(command_slot)),
        None => descriptor,
    }
}

fn command_id(id: &str) -> CommandId {
    CommandId::new(id).expect("valid command id")
}

fn theme_token(id: &str, color: &str) -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::define(
        ThemeTokenId::new(id).expect("valid token id"),
        ThemeTokenFamily::text(),
        ThemeTokenSource::application(),
        ThemeTokenValue::color(UiThemeColor::parse(color).expect("valid color")),
    )
}

fn artifact_from_module_map<const N: usize>(
    modules: [(WorthUiSourceModuleId, WorthUiArtifactModule); N],
) -> WorthUiArtifact {
    let module_ids = modules
        .iter()
        .map(|(module_id, _)| module_id.clone())
        .collect::<Vec<_>>();
    WorthUiArtifact::new(BTreeMap::from(modules), module_ids)
}

fn import_node(
    module_id: &WorthUiSourceModuleId,
    node_index: usize,
    target: &str,
) -> WorthUiArtifactNode {
    WorthUiArtifactNode::Import(WorthUiArtifactImportNode::new(
        WorthUiArtifactHandle::Import(WorthUiArtifactImportHandle::new(
            module_id.clone(),
            node_index,
        )),
        crate::source::test_compilation::semantic_import(target)
            .target()
            .clone(),
        0,
        WorthUiArtifactIdentitySeed::structural_fallback(format!(
            "module:{}|import:{}",
            module_id.as_str(),
            target
        )),
        WorthUiDurableStateEligibility::Ineligible {
            reason: WorthUiDurableStateIneligibilityReason::NoDurableStateSurface,
        },
    ))
}

fn module_id(path: &str) -> WorthUiSourceModuleId {
    WorthUiSourceModuleId::from_relative_path(Path::new(path)).expect("valid module id")
}
