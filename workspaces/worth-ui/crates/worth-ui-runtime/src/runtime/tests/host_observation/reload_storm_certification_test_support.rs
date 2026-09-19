use super::replacement_impact_test_support::{
    artifact_from_modules, impact_test_app, token_module,
};
use crate::capability::{
    ComponentChildPolicy, ComponentDescriptor, ComponentId, ComponentPropSchema,
    ComponentStateOwnership, SurfaceDescriptor, SurfaceId, SurfaceKind, SurfacePlacementClass,
    SurfaceStateClass, ThemeTokenDescriptor, ThemeTokenFamily, ThemeTokenId, ThemeTokenSource,
    ThemeTokenValue, UiThemeColor, WorthUiQueryViewRegistration,
};
use crate::facade::{WorthUi, WorthUiApp};
use crate::runtime::{WorthUiRuntimeLaunch, WorthUiSourceProvider};
use crate::source::WorthUiArtifact;
use worth_ui_dsl::{WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule};

pub(super) fn storm_app() -> WorthUiApp {
    impact_test_app()
}

pub(super) fn rich_storm_app() -> WorthUiApp {
    let installed = worth_ui_query_binding::certification::worth_ui_installed_test_domain(
        "reload-storm-query-app",
    );
    WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .register_component(component("workspace.component.dashboard"))
        .register_surface(surface("workspace.surface.main"))
        .register_query_view(query_binding(
            &installed,
            "workspace.view_binding.selection",
        ))
        .expect("installed selection view registers")
        .register_theme_token(theme_token("theme.text.primary", "#101820"))
        .register_theme_token(theme_token("theme.text.secondary", "#C7492A"))
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("application preparation should succeed")
}

pub(super) fn token_artifact(app: &WorthUiApp, token_id: &str) -> WorthUiArtifact {
    artifact_from_modules(app, [token_module(token_id)])
}

pub(super) fn rich_artifact(app: &WorthUiApp, token_id: &str) -> WorthUiArtifact {
    artifact_from_modules(
        app,
        [WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
            .with_component("workspace.component.dashboard")
            .with_surface("workspace.surface.main")
            .with_binding("workspace.view_binding.selection")
            .with_token(token_id, token_id)],
    )
}

pub(super) fn runtime_with_token(
    app: &WorthUiApp,
    token_id: &str,
) -> crate::runtime::WorthUiRuntimeFrameworkLoop {
    app.launch_runtime(WorthUiRuntimeLaunch::from_canonical_artifact(
        token_artifact(app, token_id),
    ))
    .expect("runtime launches")
}

pub(super) fn runtime_with_rich_artifact(
    app: &WorthUiApp,
    token_id: &str,
) -> crate::runtime::WorthUiRuntimeFrameworkLoop {
    app.launch_runtime(WorthUiRuntimeLaunch::from_canonical_artifact(
        rich_artifact(app, token_id),
    ))
    .expect("runtime launches")
}

pub(super) fn file_token_provider(token_id: &str) -> WorthUiSourceProvider {
    WorthUiSourceProvider::in_memory("reload-storm-file-token").with_file(
        "app/main.wui",
        format!(r#"token {token_id} = "{token_id}";"#),
    )
}

pub(super) fn rust_token_provider(_app: &WorthUiApp, token_id: &str) -> WorthUiSourceProvider {
    WorthUiSourceProvider::rust_authored(format!("rust-authored-{token_id}"))
        .with_rust_authored_input(WorthUiRustAuthoredArtifactInput::from_modules([
            token_module(token_id),
        ]))
}

pub(super) fn rich_file_provider(token_id: &str) -> WorthUiSourceProvider {
    WorthUiSourceProvider::in_memory("reload-storm-invalid-file").with_file(
        "app/main.wui",
        format!(
            r#"
            component workspace.component.dashboard {{}}
            surface workspace.surface.main {{}}
            binding workspace.view_binding.selection {{}}
            token {token_id} = "{token_id}";
            "#
        ),
    )
}

pub(super) fn rich_rust_provider(_app: &WorthUiApp, token_id: &str) -> WorthUiSourceProvider {
    WorthUiSourceProvider::rust_authored(format!("rust-authored-rich-{token_id}"))
        .with_rust_authored_input(WorthUiRustAuthoredArtifactInput::from_modules([
            WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
                .with_component("workspace.component.dashboard")
                .with_surface("workspace.surface.main")
                .with_binding("workspace.view_binding.selection")
                .with_token(token_id, token_id),
        ]))
}

pub(super) fn invalid_file_provider(label: &str) -> WorthUiSourceProvider {
    WorthUiSourceProvider::in_memory("reload-storm-labeled-file").with_file("app/main.wui", label)
}

fn component(id: &str) -> ComponentDescriptor {
    ComponentDescriptor::new(
        ComponentId::new(id).expect("valid component id"),
        ComponentPropSchema::named("workspace.props"),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
}

fn surface(id: &str) -> SurfaceDescriptor {
    SurfaceDescriptor::new(
        SurfaceId::new(id).expect("valid surface id"),
        SurfaceKind::primary_content(),
        ComponentId::new("workspace.component.dashboard").expect("valid component id"),
        SurfacePlacementClass::primary_region(),
        SurfaceStateClass::restorable(),
    )
}

fn theme_token(id: &str, color: &str) -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::define(
        ThemeTokenId::new(id).expect("valid token id"),
        ThemeTokenFamily::text(),
        ThemeTokenSource::application(),
        ThemeTokenValue::color(UiThemeColor::parse(color).expect("valid color")),
    )
}

fn query_binding(
    installed: &worth_ui_query_binding::WorthUiInstalledQueryDomain,
    id: &str,
) -> WorthUiQueryViewRegistration {
    WorthUiQueryViewRegistration::new(
        installed
            .measurement_view(id)
            .expect("installed Query view admits"),
    )
}
