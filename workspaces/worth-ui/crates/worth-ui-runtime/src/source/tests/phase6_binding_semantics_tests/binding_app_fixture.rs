use crate::capability::{
    CommandDescriptor, CommandId, CommandProjectionCommandReference, CommandProjectionDescriptor,
    CommandProjectionId, CommandProjectionSurface, ComponentChildPolicy, ComponentDescriptor,
    ComponentId, ComponentPropSchema, ComponentStateOwnership, IconDescriptor, IconFamily, IconId,
    IconSourceDescriptor, SurfaceDescriptor, SurfaceId, SurfaceKind, SurfacePlacementClass,
    SurfaceStateClass, ThemeTokenAlias, ThemeTokenDescriptor, ThemeTokenFamily, ThemeTokenId,
    ThemeTokenSource, ThemeTokenValue, UiThemeColor, ViewBindingDescriptor, ViewBindingId,
};
use crate::facade::{WorthUi, WorthUiApp};
use worth_ui_dsl::{WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule};

use super::binding_query_fixture::standard_query_owned_view_binding_descriptor;

pub(super) fn standard_artifact_input() -> WorthUiRustAuthoredArtifactInput {
    WorthUiRustAuthoredArtifactInput::from_modules([
        main_artifact_input_module(),
        inspector_artifact_input_module(),
    ])
}

pub(super) fn reordered_artifact_input() -> WorthUiRustAuthoredArtifactInput {
    WorthUiRustAuthoredArtifactInput::from_modules([
        inspector_artifact_input_module(),
        main_artifact_input_module(),
    ])
}

pub(super) fn admitted_app() -> WorthUiApp {
    app_with_view_binding_descriptor(standard_query_owned_view_binding_descriptor())
}

pub(super) fn app_with_view_binding_descriptor(
    view_binding_descriptor: ViewBindingDescriptor,
) -> WorthUiApp {
    WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .register_command(
            CommandDescriptor::new(
                CommandId::new("workspace.command.inspect").unwrap(),
                "Inspect",
            )
            .with_icon(IconId::new("workspace.icon.inspect").unwrap())
            .with_projection_eligibility(
                CommandProjectionId::new("workspace.command_projection.inspect_actions").unwrap(),
            ),
        )
        .register_component(component_descriptor("workspace.component.dashboard"))
        .register_component(component_descriptor("workspace.component.inspector_panel"))
        .register_icon(IconDescriptor::new(
            IconId::new("workspace.icon.inspect").unwrap(),
            IconFamily::command(),
            IconSourceDescriptor::symbol("inspect"),
        ))
        .register_icon(IconDescriptor::new(
            IconId::new("workspace.icon.surface.inspector").unwrap(),
            IconFamily::surface(),
            IconSourceDescriptor::symbol("panel"),
        ))
        .register_command_projection(
            CommandProjectionDescriptor::new(
                CommandProjectionId::new("workspace.command_projection.inspect_actions").unwrap(),
                CommandProjectionSurface::toolbar(),
            )
            .with_command_reference(CommandProjectionCommandReference::command(
                CommandId::new("workspace.command.inspect").unwrap(),
            )),
        )
        .register_view_binding(view_binding_descriptor)
        .register_surface(
            SurfaceDescriptor::new(
                SurfaceId::new("workspace.surface.inspector").unwrap(),
                SurfaceKind::primary_content(),
                ComponentId::new("workspace.component.dashboard").unwrap(),
                SurfacePlacementClass::primary_region(),
                SurfaceStateClass::restorable(),
            )
            .with_command_slot(CommandId::new("workspace.command.inspect").unwrap())
            .with_icon(IconId::new("workspace.icon.surface.inspector").unwrap())
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

fn main_artifact_input_module() -> WorthUiRustAuthoredArtifactInputModule {
    WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
        .with_import("app/panels/inspector.wui")
        .with_component("workspace.component.dashboard")
        .with_surface("workspace.surface.inspector")
        .with_binding("workspace.view_binding.selection")
        .with_token("theme.text.default", "theme.text.primary")
}

fn inspector_artifact_input_module() -> WorthUiRustAuthoredArtifactInputModule {
    WorthUiRustAuthoredArtifactInputModule::new("app/panels/inspector.wui")
        .with_component("workspace.component.inspector_panel")
}

fn component_descriptor(identity: &str) -> ComponentDescriptor {
    ComponentDescriptor::new(
        ComponentId::new(identity).unwrap(),
        ComponentPropSchema::named("workspace.props"),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
}
