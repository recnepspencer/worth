use super::super::super::support;
use super::super::fixture;
use worth_ui_dsl::*;

pub(super) fn shell() -> (
    crate::facade::WorthUiNativeApplicationShell,
    crate::certification_support::ScriptedPresentationHost,
) {
    let role = fixture::role();
    let builder = || {
        fixture::builder_with_component(&role, component(support::APPEARANCE_NODE_A, 24))
            .register_component(component(support::APPEARANCE_NODE_B, 200))
    };
    let source = WorthUiRustAuthoredArtifactInput::from_modules([
        WorthUiRustAuthoredArtifactInputModule::new("appearance/consumer")
            .with_control_routes(
                support::APPEARANCE_NODE_A,
                [WorthUiIntentInteractionRoute::product(
                    WorthUiIntentInteractionFamily::Activate,
                    fixture::ROUTE,
                )],
            )
            .with_control_routes(
                support::APPEARANCE_NODE_B,
                [WorthUiIntentInteractionRoute::confirmation(fixture::ROUTE)],
            )
            .with_intent_declaration(WorthUiIntentDeclarationSpec::new(
                fixture::ROUTE,
                "test.appearance.intent",
                WorthUiIntentInteractionFamily::Activate,
                WorthUiIntentOperabilityContractSpec::new(
                    "test.appearance.operability",
                    WorthUiIntentMutabilitySourceSpec::application_boolean(fixture::MUTABLE),
                    WorthUiIntentReadinessSourceSpec::application_boolean("test.appearance.ready"),
                    WorthUiIntentPolicySourceSpec::application_boolean(fixture::POLICY),
                ),
                WorthUiIntentConfirmationContractSpec::application_boolean(
                    "test.appearance.confirmation",
                    "test.appearance.ready",
                ),
                WorthUiIntentConcurrencyScope::TargetRouteSingleFlight,
                WorthUiIntentConsequenceContractSpec::none(),
            )),
    ]);
    let capabilities = builder().freeze().unwrap();
    let launch = crate::runtime::tests::source_ingress_boundary_test_support::lower_rust_submission(
        crate::runtime::WorthUiSourceProvider::rust_authored("pointer-confirmation-launch")
            .with_rust_authored_input(source),
        [crate::runtime::WorthUiWatcherEvent::provider_revision(
            "pointer-confirmation-launch",
        )],
        capabilities.capabilities(),
    );
    let app = builder()
        .with_candidate_submission(launch)
        .freeze()
        .unwrap();
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    host.set_capabilities(worth_ui_host_native::appearance_capability_report());
    let observer = host.clone();
    let mut session =
        crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
            app, host,
        )
        .launch_native_surface()
        .unwrap();
    let surface = session
        .session
        .inspect_mounted_identity()
        .surface_bindings()[0]
        .semantic_surface_identity();
    crate::facade::entry::mounted_occurrence_geometry_test_support::refresh_nonoverlapping_surface_geometry(
        &mut session.session,
        surface,
    );
    (session, observer)
}

fn component(name: &str, x: u16) -> crate::capability::ComponentDescriptor {
    use crate::capability::*;
    let allocation =
        ComponentAllocationMeasurementContract::viewport_region(ComponentViewportRegion::new(
            ComponentViewportAxisPlacement::fixed_from_start(x, 100).unwrap(),
            ComponentViewportAxisPlacement::fixed_from_start(24, 60).unwrap(),
        ));
    let token = ThemeTokenId::new(support::LEGACY_STATIC_PAINT_TOKEN).unwrap();
    crate::runtime::tests::appearance_component_session_test_support::static_paint_component_with_allocation(
        name, token, allocation,
    )
    .with_hit_test(ComponentHitTestContract::allocation_bounds(
        ComponentHitTestOrder::front_to_back(u32::from(name != support::APPEARANCE_NODE_A)),
        allocation,
    ))
}
