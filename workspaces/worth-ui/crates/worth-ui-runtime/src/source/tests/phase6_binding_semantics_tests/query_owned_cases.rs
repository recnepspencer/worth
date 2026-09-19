use super::binding_app_fixture::admitted_app;
use super::binding_phase_fixture::bound_artifact_input;
use crate::capability::{
    ThemeTokenAlias, ThemeTokenDescriptor, ThemeTokenFamily, ThemeTokenId, ThemeTokenSource,
    ThemeTokenValue, UiThemeColor,
};
use crate::facade::{WorthUi, WorthUiApp};
use crate::source::{
    WorthUiArtifactInputResolver, WorthUiBindingDiagnosticCode, WorthUiBindingSemanticsLowerer,
    WorthUiBindingSemanticsReport, WorthUiBoundArtifactInput, WorthUiBoundArtifactInputNode,
    WorthUiStructuralLegalityLowerer,
};
use worth_ui_dsl::{
    WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule,
    WorthUiSealedSemanticPackage,
};

#[test]
fn nested_command_and_surface_semantics_preserve_typed_identity() {
    let app = admitted_app();
    let snapshot = app.capabilities();
    let bound = bound_artifact_input(snapshot);

    let module = bound.module(bound.module_ids().first().unwrap()).unwrap();
    let surface = module
        .nodes()
        .iter()
        .find_map(|node| match node {
            WorthUiBoundArtifactInputNode::Surface(surface) => Some(surface),
            _ => None,
        })
        .unwrap();
    let surface_semantics = surface.semantics();
    let command = surface_semantics.command_slots().first().unwrap();

    assert_eq!(
        surface_semantics.icon().unwrap().icon().id().as_str(),
        "workspace.icon.surface.inspector"
    );
    assert_eq!(
        command.semantics().icon().unwrap().icon().id().as_str(),
        "workspace.icon.inspect"
    );
    assert_eq!(
        command
            .semantics()
            .projection_eligibility()
            .unwrap()
            .command_projection()
            .id()
            .as_str(),
        "workspace.command_projection.inspect_actions"
    );
}

#[test]
fn query_bound_view_reference_preserves_the_single_admitted_definition() {
    let app = admitted_app();
    let snapshot = app.capabilities();
    let bound = bound_artifact_input(snapshot);

    let module = bound.module(bound.module_ids().first().unwrap()).unwrap();
    let binding = module
        .nodes()
        .iter()
        .find_map(|node| match node {
            WorthUiBoundArtifactInputNode::Binding(binding) => Some(binding),
            _ => None,
        })
        .unwrap();
    let semantics = binding.view_binding_reference().query_semantics();
    let descriptor = binding.view_binding_reference().entry().descriptor();

    assert_eq!(semantics.definition(), descriptor.definition());
    assert_eq!(
        semantics.definition().lifecycle(),
        worth_ui_query_binding::WorthUiQueryViewLifecycle::Snapshot
    );
    assert_eq!(
        semantics.definition().shape(),
        worth_ui_query_binding::WorthUiQueryViewShape::Collection
    );
    assert_eq!(semantics.definition().required_facts().len(), 1);
    assert_eq!(
        semantics.denial_presentation(),
        descriptor.denial_presentation()
    );
}

#[test]
fn theme_token_resolution_preserves_frozen_target_identity() {
    let app = admitted_app();
    let snapshot = app.capabilities();
    let bound = bound_artifact_input(snapshot);

    let module = bound.module(bound.module_ids().first().unwrap()).unwrap();
    let token = module
        .nodes()
        .iter()
        .find_map(|node| match node {
            WorthUiBoundArtifactInputNode::Token(token) => Some(token),
            _ => None,
        })
        .unwrap();

    assert_eq!(
        token
            .semantics()
            .resolved_target_theme_token()
            .id()
            .as_str(),
        "theme.text.primary"
    );
    assert_eq!(
        token
            .semantics()
            .resolved_target_entry()
            .descriptor()
            .id()
            .as_str(),
        "theme.text.primary"
    );
}

#[test]
fn file_authored_token_reference_can_rebind_the_registered_alias_target() {
    let package = crate::source::test_compilation::compile_source([(
        "app/main.wui",
        "token theme.test.fill = \"theme.test.green\";",
    )]);
    let bound = lower_token_package(package).expect("admitted file alias should bind");

    assert_eq!(bound_token_target(&bound), "theme.test.green");
}

#[test]
fn invalid_file_authored_token_reference_is_not_replaced_by_the_registered_default() {
    let package = crate::source::test_compilation::compile_source([(
        "app/main.wui",
        "token theme.test.fill = \"theme.test.missing\";",
    )]);
    let report = lower_token_package(package).expect_err("unknown file alias should be denied");

    assert_eq!(
        report.diagnostics()[0].code(),
        WorthUiBindingDiagnosticCode::MissingSemanticThemeTokenReference
    );
}

#[test]
fn rust_authored_token_payload_retains_the_registered_alias_target() {
    let package = crate::source::test_compilation::compile_rust_authored(
        &WorthUiRustAuthoredArtifactInput::from_modules([
            WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
                .with_token("theme.test.fill", "#payload-not-a-token-id"),
        ]),
    );
    let bound = lower_token_package(package).expect("Rust-authored payload should bind");

    assert_eq!(bound_token_target(&bound), "theme.test.blue");
}

fn lower_token_package(
    package: WorthUiSealedSemanticPackage,
) -> Result<WorthUiBoundArtifactInput, WorthUiBindingSemanticsReport> {
    let app = token_target_app();
    let snapshot = app.capabilities();
    let resolved = WorthUiArtifactInputResolver::resolve(&package, snapshot)
        .expect("token name should resolve");
    let structured = WorthUiStructuralLegalityLowerer::lower(&resolved, snapshot)
        .expect("token should remain structurally legal");
    WorthUiBindingSemanticsLowerer::lower(&structured, snapshot)
}

fn bound_token_target(bound: &WorthUiBoundArtifactInput) -> &str {
    bound
        .module(bound.module_ids().first().expect("one module"))
        .expect("module remains bound")
        .nodes()
        .iter()
        .find_map(|node| match node {
            WorthUiBoundArtifactInputNode::Token(token) => Some(
                token
                    .semantics()
                    .resolved_target_theme_token()
                    .id()
                    .as_str(),
            ),
            _ => None,
        })
        .expect("one token remains bound")
}

fn token_target_app() -> WorthUiApp {
    WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .register_theme_token(color_token("theme.test.blue", "#2f81f7"))
        .register_theme_token(color_token("theme.test.green", "#3fb950"))
        .register_theme_token(ThemeTokenDescriptor::alias(
            ThemeTokenId::new("theme.test.fill").expect("valid fill token"),
            ThemeTokenFamily::surface(),
            ThemeTokenSource::application(),
            ThemeTokenAlias::to(ThemeTokenId::new("theme.test.blue").expect("valid blue token")),
        ))
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("token target app should freeze")
}

fn color_token(id: &str, color: &str) -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::define(
        ThemeTokenId::new(id).expect("valid test token"),
        ThemeTokenFamily::surface(),
        ThemeTokenSource::application(),
        ThemeTokenValue::color(UiThemeColor::parse(color).expect("valid test color")),
    )
}
