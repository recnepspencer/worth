use super::native_application_identity_trace_test_support::{completed, frame_receipt};
use super::native_identity_trace_host::NativeIdentityTraceHost;
use crate::capability::{
    ComponentAllocationMeasurementContract, ComponentChildPolicy, ComponentDescriptor, ComponentId,
    ComponentPropSchema, ComponentStateOwnership, ComponentStaticPaintContract,
    ComponentStaticPaintOrder, ThemeColorValue, ThemeTokenDescriptor, ThemeTokenFamily,
    ThemeTokenId, ThemeTokenSource, ThemeTokenValue,
};

const COMPONENT: &str = "test.native.presentation_attribution";
const TOKEN: &str = "theme.test.native.presentation_attribution";
const ZERO_CONSUMER_TOKEN: &str = "theme.test.native.zero_consumer_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[test]
fn presentation_attribution_follows_the_latest_physical_publication() {
    let host = NativeIdentityTraceHost::default();
    let app = crate::facade::WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .register_theme_token(theme_token("#102030"))
        .register_component(component())
        .with_rust_authored_input(
            worth_ui_dsl::WorthUiRustAuthoredArtifactInput::from_modules([
                worth_ui_dsl::WorthUiRustAuthoredArtifactInputModule::new("app/native-paint.wui")
                    .with_token(TOKEN, "#102030")
                    .with_component_authored_identity(COMPONENT, "native-paint"),
            ]),
        )
        .freeze()
        .map(|application| {
            super::WorthUiCertificationApplicationTransition::activate_test_host(application, host)
        })
        .expect("painted native fixture should prepare");
    let mut shell = app
        .launch_native_surface()
        .expect("painted native fixture should launch");

    let predecessor = frame_receipt(completed(shell.present_frame(100, 1)));
    let predecessor_attribution = shell
        .current_presentation_attribution()
        .expect("physical paint must expose retained attribution");
    assert_publication_matches(&predecessor, predecessor_attribution);

    let token = ThemeTokenId::new(TOKEN).expect("fixture token id");
    let successor_value =
        ThemeTokenValue::color(ThemeColorValue::hex("#405060").expect("fixture color"));
    shell
        .apply_theme_token_values(&[super::UiNativeThemeTokenValueChange::new(
            token.clone(),
            successor_value.clone(),
        )
        .expect("application-owned token successor")])
        .expect("theme successor should be admitted");

    let source = shell.session.complete_application_theme_values_source();
    assert_eq!(source.current_value(&token), Some(&successor_value));
    assert!(source.has_theme_changes());

    let successor = frame_receipt(completed(shell.present_frame(200, 2)));
    assert_eq!(
        successor.cost_report().work_class(),
        crate::mounting::UiMountWorkClass::SemanticDelta
    );
    let successor_attribution = shell
        .current_presentation_attribution()
        .expect("successor physical paint must replace attribution");
    assert_publication_matches(&successor, successor_attribution);
    assert_ne!(
        successor_attribution.frame(),
        predecessor_attribution.frame()
    );
    assert!(!shell
        .session
        .complete_application_theme_values_source()
        .has_theme_changes());
    assert!(shell.shutdown().host_session_released());
}

#[test]
fn admitted_zero_consumer_theme_slot_commits_without_mounted_work() {
    let host = NativeIdentityTraceHost::default();
    let app = crate::facade::WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .register_theme_token(theme_token("#102030"))
        .register_theme_token(theme_token_with_id(ZERO_CONSUMER_TOKEN, "#203040"))
        .register_component(component())
        .with_rust_authored_input(
            worth_ui_dsl::WorthUiRustAuthoredArtifactInput::from_modules([
                worth_ui_dsl::WorthUiRustAuthoredArtifactInputModule::new("app/native-paint.wui")
                    .with_token(TOKEN, "#102030")
                    .with_component_authored_identity(COMPONENT, "native-paint"),
            ]),
        )
        .freeze()
        .map(|application| {
            super::WorthUiCertificationApplicationTransition::activate_test_host(application, host)
        })
        .expect("zero-consumer fixture should prepare");
    let mut shell = app
        .launch_native_surface()
        .expect("zero-consumer fixture should launch");
    completed(shell.present_frame(100, 1));

    let zero_consumer_token = ThemeTokenId::new(ZERO_CONSUMER_TOKEN).expect("fixture token id");
    let successor = ThemeTokenValue::color(ThemeColorValue::hex("#405060").expect("fixture color"));
    shell
        .apply_theme_token_values(&[super::UiNativeThemeTokenValueChange::new(
            zero_consumer_token.clone(),
            successor.clone(),
        )
        .expect("long application-owned token successor")])
        .expect("valid zero-consumer token should commit");

    let source = shell.session.complete_application_theme_values_source();
    assert_eq!(source.current_value(&zero_consumer_token), Some(&successor));
    assert!(!source.has_theme_changes());

    let outcome = completed(shell.present_frame(200, 2));
    assert_eq!(
        outcome
            .cost_report()
            .expect("ordinary frame should report mounting cost")
            .work_class(),
        crate::mounting::UiMountWorkClass::UnchangedReuse
    );

    let unknown = ThemeTokenId::new("theme.test.native.unknown").expect("fixture token id");
    assert_eq!(
        shell
            .session
            .application
            .appearance_slot_consumers(&unknown),
        Err(
            crate::graph::UiGraphFactLookupDenial::UnknownAuthoredDeclaration {
                authored_identity: unknown.as_str().into(),
            }
        )
    );
    assert!(shell.shutdown().host_session_released());
}

fn assert_publication_matches(
    publication: &crate::mounting::UiMountedFramePublicationReceipt,
    attribution: worth_ui_host_native::UiNativeClientPresentationAttribution,
) {
    assert_eq!(attribution.frame(), publication.frame().diagnostic_value());
    assert_eq!(
        attribution.presentation_attempt(),
        publication.attempt().diagnostic_value()
    );
}

fn component() -> ComponentDescriptor {
    ComponentDescriptor::new(
        ComponentId::new(COMPONENT).expect("fixture component id"),
        ComponentPropSchema::named("test.native.presentation_attribution.props"),
        ComponentChildPolicy::no_children(),
        ComponentStateOwnership::runtime_owned(),
    )
    .with_static_paint(
        ComponentStaticPaintContract::opaque_fill(
            ThemeTokenId::new(TOKEN).expect("fixture token id"),
            ComponentStaticPaintOrder::back_to_front(0),
        ),
        ComponentAllocationMeasurementContract::fill_viewport(),
    )
}

fn theme_token(color: &str) -> ThemeTokenDescriptor {
    theme_token_with_id(TOKEN, color)
}

fn theme_token_with_id(token: &str, color: &str) -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::define(
        ThemeTokenId::new(token).expect("fixture token id"),
        ThemeTokenFamily::surface(),
        ThemeTokenSource::application(),
        ThemeTokenValue::color(ThemeColorValue::hex(color).expect("fixture color")),
    )
}
