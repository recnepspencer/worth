use super::*;

#[test]
fn switching_enforces_surface_application_and_predecessor_cas() {
    let surface = next_surface();
    let other_surface = next_surface();
    let (origins, application) = admitted_source_origins("theme-origin-cas", 2);
    let origin = origins[0].clone();
    let mut state = UiAppearanceThemeState::default();
    state
        .install_initial(capability("theme.initial", surface, application.clone()))
        .unwrap();
    state
        .install_initial(capability(
            "theme.other-initial",
            other_surface,
            application.clone(),
        ))
        .unwrap();

    assert_eq!(
        state.prepare_theme_switch(request(
            origin.clone(),
            surface,
            2,
            capability("theme.stale", surface, application.clone()),
        )),
        Err(UiThemeSwitchDenial::StaleBinding)
    );
    assert_eq!(
        state.prepare_theme_switch(UiThemeSwitchRequest::new(
            origin.clone(),
            other_surface,
            1,
            capability("theme.wrong-surface", surface, application.clone()),
        )),
        Err(UiThemeSwitchDenial::WrongSurfaceCapability)
    );
    assert_eq!(
        state.prepare_theme_switch(request(
            origin.clone(),
            surface,
            1,
            capability("theme.foreign-app", surface, generation(42)),
        )),
        Err(UiThemeSwitchDenial::WrongOriginSession)
    );

    let first = state
        .prepare_theme_switch(request(
            origin.clone(),
            surface,
            1,
            capability("theme.first", surface, application.clone()),
        ))
        .unwrap();
    let stale = state
        .prepare_theme_switch(request(
            origins[1].clone(),
            surface,
            1,
            capability("theme.second", surface, application.clone()),
        ))
        .unwrap();
    let unrelated = state
        .prepare_theme_switch(request(
            origin,
            other_surface,
            1,
            capability("theme.other-next", other_surface, application),
        ))
        .unwrap();
    state.commit_published_switch(first).unwrap();
    assert_eq!(state.prepared_switch_count(), 1);
    assert_eq!(
        state.commit_published_switch(stale),
        Err(UiThemeSwitchDenial::UnknownPreparedSwitch)
    );
    drop(unrelated);
    assert_eq!(state.prepared_switch_count(), 0);
}

#[test]
fn prepared_switches_are_affine_bounded_and_cancellable() {
    let surface = next_surface();
    let (origins, application) = admitted_source_origins("theme-origin-bounded", 5);
    let mut state = UiAppearanceThemeState::default();
    state
        .install_initial(capability("theme.initial", surface, application.clone()))
        .unwrap();
    assert_eq!(
        state.install_initial(capability("theme.duplicate", surface, application.clone())),
        Err(UiThemeInitialBindingDenial::SurfaceAlreadyBound)
    );

    let mut prepared = Vec::new();
    for (index, origin) in origins.iter().take(4).enumerate() {
        prepared.push(
            state
                .prepare_theme_switch(request(
                    origin.clone(),
                    surface,
                    1,
                    capability(&format!("theme.{index}"), surface, application.clone()),
                ))
                .unwrap(),
        );
    }
    assert_eq!(state.prepared_switch_count(), 4);
    assert_eq!(
        state.prepare_theme_switch(request(
            origins[4].clone(),
            surface,
            1,
            capability("theme.overflow", surface, application.clone()),
        )),
        Err(UiThemeSwitchDenial::PreparedSwitchCapacityExceeded)
    );
    let cancelled = prepared.pop().unwrap();
    let replay = duplicate_prepared(&cancelled);
    drop(cancelled);
    assert_eq!(
        state.commit_published_switch(replay),
        Err(UiThemeSwitchDenial::UnknownPreparedSwitch)
    );
    assert_eq!(state.prepared_switch_count(), 3);
    assert!(state
        .prepare_theme_switch(request(
            origins[4].clone(),
            surface,
            1,
            capability("theme.replacement", surface, application.clone()),
        ))
        .is_ok());
    assert_eq!(
        state.prepared_switch_count(),
        3,
        "dropped preparation releases capacity"
    );
    drop(prepared);
    assert_eq!(
        state.prepared_switch_count(),
        0,
        "abandonment releases every reservation"
    );
    assert_eq!(
        state.prepare_theme_switch(request(
            origins[4].clone(),
            surface,
            1,
            capability("theme.replayed-drop", surface, application.clone())
        )),
        Err(UiThemeSwitchDenial::DuplicateOrigin)
    );
    assert_eq!(
        state.prepare_theme_switch(request(
            origins[3].clone(),
            surface,
            1,
            capability("theme.replayed-cancellation", surface, application)
        )),
        Err(UiThemeSwitchDenial::SupersededOrigin)
    );
}

#[test]
fn unchanged_binding_consumes_origin_without_reserving_or_republishing() {
    let surface = next_surface();
    let (origin, application) = admitted_source_origin("theme-origin-unchanged");
    let mut state = UiAppearanceThemeState::default();
    let initial = capability("theme.initial", surface, application.clone());
    state.install_initial(initial.clone()).unwrap();
    state
        .settle_unchanged_switch(request(origin.clone(), surface, 1, initial.clone()))
        .unwrap();
    assert_eq!(state.prepared_switch_count(), 0);
    assert_eq!(
        state.active_binding(surface).unwrap().binding_generation(),
        1
    );
    assert_eq!(
        state.settle_unchanged_switch(request(origin.clone(), surface, 1, initial)),
        Err(UiThemeSwitchDenial::DuplicateOrigin)
    );
    assert_eq!(
        state.prepare_theme_switch(request(
            origin,
            surface,
            1,
            capability("theme.next", surface, application)
        )),
        Err(UiThemeSwitchDenial::DuplicateOrigin)
    );
}

#[test]
fn prepared_switch_cannot_cross_same_application_owner_with_colliding_reservation() {
    let surface = next_surface();
    let mut first = UiAppearanceThemeState::default();
    let mut second = UiAppearanceThemeState::default();
    let (origin, first_application) = admitted_source_origin("theme-origin-owner-affinity");
    let second_application = first_application.clone();
    first
        .install_initial(capability(
            "theme.first",
            surface,
            first_application.clone(),
        ))
        .unwrap();
    second
        .install_initial(capability(
            "theme.second",
            surface,
            second_application.clone(),
        ))
        .unwrap();
    let foreign = first
        .prepare_theme_switch(request(
            origin.clone(),
            surface,
            1,
            capability("theme.first-next", surface, first_application),
        ))
        .unwrap();
    let local = second
        .prepare_theme_switch(request(
            origin,
            surface,
            1,
            capability("theme.second-next", surface, second_application),
        ))
        .unwrap();
    assert_eq!(
        second.commit_published_switch(foreign),
        Err(UiThemeSwitchDenial::UnknownPreparedSwitch)
    );
    assert_eq!(second.prepared_switch_count(), 1);
    second.commit_published_switch(local).unwrap();
}

fn duplicate_prepared(prepared: &UiPreparedThemeSwitch) -> UiPreparedThemeSwitch {
    UiPreparedThemeSwitch {
        reservation: prepared.reservation,
        predecessor_generation: prepared.predecessor_generation,
        successor: prepared.successor.clone(),
        origin: prepared.origin.clone(),
        owner_affinity: prepared.owner_affinity,
        reservations: prepared.reservations.clone(),
    }
}

fn request(
    origin: UiThemeSwitchOrigin,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    expected_generation: u64,
    capability: UiThemeCapabilityReceipt,
) -> UiThemeSwitchRequest {
    UiThemeSwitchRequest::new(origin, surface, expected_generation, capability)
}

fn admitted_source_origin(
    source_name: &str,
) -> (
    UiThemeSwitchOrigin,
    crate::runtime::WorthUiActiveApplicationGenerationIdentity,
) {
    let (mut origins, application) = admitted_source_origins(source_name, 1);
    (origins.pop().unwrap(), application)
}

fn admitted_source_origins(
    source_name: &str,
    count: usize,
) -> (
    Vec<UiThemeSwitchOrigin>,
    crate::runtime::WorthUiActiveApplicationGenerationIdentity,
) {
    use crate::runtime::tests::active_application_session_test_support::component_candidate_submission;
    use crate::runtime::tests::appearance_component_session_test_support::source_backed_appearance_consumer_session;

    let mut session = source_backed_appearance_consumer_session();
    let application = session.active_generation_identity();
    let mut origins = Vec::new();
    for index in 0..count {
        let candidate = component_candidate_submission(
            &session,
            &format!("{source_name}-{index}"),
            "workspace.component.active_session_current",
        );
        let mut turn = session.begin_observation_turn().unwrap();
        turn.admit_source(candidate).unwrap();
        let admitted = turn.seal().unwrap();
        origins.push(
            session
                .issue_theme_switch_origin(
                    &admitted,
                    UiThemeSwitchOriginFamily::SourceEditObservation,
                )
                .unwrap(),
        );
    }
    let _ = session.shutdown();
    (origins, application)
}

fn next_surface() -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
    worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap()
}

fn generation(seed: u64) -> crate::runtime::WorthUiActiveApplicationGenerationIdentity {
    use worth_ui_dsl::{
        UiDslSemanticArtifactSpec, UiDslSemanticFamily, UiDslSemanticKey, UiDslSourceProvenance,
        UiDslStructuralToken,
    };
    let app = crate::facade::WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .with_rust_authored_declaration_fixture(
            crate::facade::WorthUiRustAuthoredDeclarationFixture::named(format!(
                "appearance-theme-fixture-{seed}"
            ))
            .with_semantic_artifact_spec(
                UiDslSemanticArtifactSpec::new(
                    UiDslSemanticKey::new(format!("ui.appearance.theme.{seed}")),
                    UiDslSemanticFamily::Control,
                    UiDslSourceProvenance::rust_authored("appearance/theme", 0),
                )
                .with_structural_token(UiDslStructuralToken::new("control:appearance-theme")),
            ),
        )
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .unwrap();
    crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
        crate::lifecycle::WorthUiActiveApplicationSessionIdentity::from_host_session_value(seed),
        app.generation_identity(),
    )
}

#[path = "state_tests/palette.rs"]
mod palette;
use palette::capability;
