use worth_ui::facade::declaration::{ComponentSemanticTextContract, ThemeTokenId};
use worth_ui_dsl::{WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule};
use worth_ui_host_headless::WorthUiHeadlessRecorder;
use worth_ui_query_binding::UiScalarProjectionRegistration;

use super::super::scalar_query_only::{
    component_descriptor, projection_module, text_token_descriptor, ACTIVE_COMPONENT, TEXT_COLOR,
};

pub(super) fn application(
    registration: UiScalarProjectionRegistration,
    recorder: WorthUiHeadlessRecorder,
) -> worth_ui::facade::app::WorthUiApp {
    let component = component_descriptor(ACTIVE_COMPONENT)
        .with_semantic_text(
            ComponentSemanticTextContract::body_default(ThemeTokenId::new(TEXT_COLOR).unwrap(), 1)
                .with_appearance_foreground(),
        )
        .with_appearance_aspect_contract(
            super::super::query_text_appearance::white_role()
                .aspect_contract()
                .clone(),
        )
        .unwrap();
    let frozen = worth_ui::facade::app::WorthUi::app()
        .with_change_profile(worth_ui::facade::rebind::UiChangeProfile::platform_pulse())
        .register_component(component)
        .register_theme_token(text_token_descriptor())
        .register_scalar_projection(registration)
        .unwrap()
        .register_appearance_role(super::super::query_text_appearance::white_role())
        .unwrap()
        .register_appearance_role(super::super::query_text_appearance::green_role())
        .unwrap()
        .register_appearance_theme_bundle(super::super::query_text_appearance::theme())
        .unwrap()
        .with_rust_authored_input(WorthUiRustAuthoredArtifactInput::from_modules([module(
            false,
        )]))
        .freeze()
        .unwrap();
    worth_ui_runtime::facade::entry::WorthUiCertificationApplicationTransition::activate_recorder(
        frozen, recorder,
    )
}

pub(super) fn module(green: bool) -> WorthUiRustAuthoredArtifactInputModule {
    super::super::query_text_appearance::attach(
        projection_module(ACTIVE_COMPONENT),
        ACTIVE_COMPONENT,
        green,
    )
}
