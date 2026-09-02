use worth_ui_dsl::{
    UiAppearanceAspect, UiAppearanceDecisionPartition, UiAppearanceDecisionResult,
    UiAppearanceRoleApplicability, UiAppearanceRoleDeclaration, UiAppearanceRoleIdentity,
    UiAppearanceRoleRevision, UiBackdropDeclaration, UiBackdropExtentBasis, UiBackdropIdentity,
    UiBackdropMotionBasis, UiBackdropPlacement, UiBackdropPresenceBasis, UiBackdropScope,
    UiSemanticSurfaceDeclarationIdentity, UiThemeSlotIdentity, UiThemeValue, UiThemeValueKind,
};

use super::super::state::UiBackdropAppearanceStateVector;
use super::super::theme::UiThemeResolutionView;
use super::{
    UiAppearanceResolutionDenial, UiAppearanceResolutionSubject, UiAppearanceResolver,
    UiBackdropAppearanceProjection, UiBackdropInstanceIdentity, UiOverlayStackSnapshot,
};
use crate::runtime::overlay_composition::UiOverlayStackRow;

#[test]
fn identical_backdrop_inputs_are_exactly_equivalent_and_digest_identical() {
    super::tests::run_on_appearance_fixture_stack(|| {
        let (session, role, surface, vector, theme) = inputs();
        let first = projection_for(&session, &role, surface, &vector, &theme, 7, 1, 1);
        let second = projection_for(&session, &role, surface, &vector, &theme, 7, 1, 1);
        assert!(first.exactly_equivalent(&second));
        assert_eq!(first.semantic_digest(), second.semantic_digest());
        assert_eq!(first.catalog_revision(), theme.catalog_revision());
        assert_eq!(first.aspects().len(), 2);
        let _ = session.shutdown();
    });
}
#[test]
fn backdrop_digest_carries_instance_declaration_and_overlay_evidence() {
    super::tests::run_on_appearance_fixture_stack(|| {
        let (session, role, surface, vector, theme) = inputs();
        let base = projection_for(&session, &role, surface, &vector, &theme, 7, 1, 1);
        let changed_instance = projection_for(&session, &role, surface, &vector, &theme, 8, 1, 1);
        let changed_overlay = projection_for(&session, &role, surface, &vector, &theme, 7, 2, 1);
        let changed_declaration_revision =
            projection_for(&session, &role, surface, &vector, &theme, 7, 1, 2);
        for changed in [
            changed_instance,
            changed_overlay,
            changed_declaration_revision,
        ] {
            assert_ne!(base.semantic_digest(), changed.semantic_digest());
            assert!(!base.exactly_equivalent(&changed));
        }
        let _ = session.shutdown();
    });
}
#[test]
fn backdrop_denial_evidence_is_resolver_owned_and_effect_free() {
    super::tests::run_on_appearance_fixture_stack(|| {
        let (session, role, surface, vector, theme) = inputs();
        let declaration_surface = UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
        let identity = UiBackdropIdentity::new(9).unwrap();
        let declaration = declaration(&role, identity, declaration_surface);
        let instance = UiBackdropInstanceIdentity::surface_singleton(identity);
        let overlay = overlay(
            &session,
            surface,
            declaration_surface,
            instance,
            false,
            1,
            1,
        );
        let before_value = current_value(&session);
        let before_frames = session.inspect_mounted_identity().frame_receipts().len();

        let evidence = UiAppearanceResolver::new()
            .resolve_backdrop(instance, &declaration, &role, &vector, &theme, &overlay)
            .expect_err("a missing overlay participant must be denied by the resolver");

        assert_eq!(
            evidence.denial(),
            UiAppearanceResolutionDenial::OverlayParticipantMissing
        );
        assert_eq!(
            evidence.subject(),
            UiAppearanceResolutionSubject::Backdrop(instance)
        );
        assert_ne!(evidence.input_digest(), 0);
        assert_zero_effects(evidence.effects());
        assert_eq!(current_value(&session), before_value);
        assert_eq!(
            session.inspect_mounted_identity().frame_receipts().len(),
            before_frames
        );
        let _ = session.shutdown();
    });
}
#[test]
fn backdrop_surface_denial_has_independent_zero_effect_evidence() {
    super::tests::run_on_appearance_fixture_stack(|| {
        let (session, role, surface, vector, _theme) = inputs();
        let declaration_surface = UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
        let identity = UiBackdropIdentity::new(10).unwrap();
        let declaration = declaration(&role, identity, declaration_surface);
        let instance = UiBackdropInstanceIdentity::surface_singleton(identity);
        let overlay = overlay(&session, surface, declaration_surface, instance, true, 1, 1);
        let foreign_surface =
            worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let foreign_theme = backdrop_theme_view(&session, &role, foreign_surface);
        let before_value = current_value(&session);

        let evidence = UiAppearanceResolver::new()
            .resolve_backdrop(
                instance,
                &declaration,
                &role,
                &vector,
                &foreign_theme,
                &overlay,
            )
            .expect_err("a vector/theme surface mismatch must be denied");

        assert_eq!(
            evidence.denial(),
            UiAppearanceResolutionDenial::WrongSurface
        );
        assert_zero_effects(evidence.effects());
        assert_eq!(current_value(&session), before_value);
        assert!(session
            .inspect_mounted_identity()
            .frame_receipts()
            .is_empty());
        let _ = session.shutdown();
    });
}
#[test]
fn unadmitted_backdrop_role_denial_is_independent_and_effect_free() {
    super::tests::run_on_appearance_fixture_stack(|| {
        let (session, _role, surface, vector, theme) = inputs();
        let unadmitted = backdrop_role("test.backdrop.unadmitted");
        let declaration_surface = UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
        let identity = UiBackdropIdentity::new(11).unwrap();
        let declaration = declaration(&unadmitted, identity, declaration_surface);
        let instance = UiBackdropInstanceIdentity::surface_singleton(identity);
        let overlay = overlay(&session, surface, declaration_surface, instance, true, 1, 1);
        let before_value = current_value(&session);

        let evidence = UiAppearanceResolver::new()
            .resolve_backdrop(
                instance,
                &declaration,
                &unadmitted,
                &vector,
                &theme,
                &overlay,
            )
            .expect_err("an unadmitted role must be denied by the resolver");

        assert_eq!(
            evidence.denial(),
            UiAppearanceResolutionDenial::MissingRoleCapability
        );
        assert_zero_effects(evidence.effects());
        assert_eq!(current_value(&session), before_value);
        assert!(session
            .inspect_mounted_identity()
            .frame_receipts()
            .is_empty());
        let _ = session.shutdown();
    });
}
fn inputs() -> (
    crate::facade::WorthUiActiveApplicationSession,
    UiAppearanceRoleDeclaration,
    worth_ui_host_contract::UiSemanticSurfaceIdentity,
    UiBackdropAppearanceStateVector,
    UiThemeResolutionView,
) {
    let role = backdrop_role("test.backdrop.digest");
    let mut session = backdrop_session(&role);
    let candidate = crate::runtime::tests::appearance_component_session_test_support::
        attached_appearance_candidate_submission(
            &session,
            "backdrop-resolver-test",
            "workspace.component.active_session_current",
        );
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let observations = turn.seal().unwrap();
    session.classify_observations(observations).unwrap();
    let snapshot = session.appearance_owner_snapshot_for_test().unwrap();
    let surface = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let vector = UiBackdropAppearanceStateVector::seal(snapshot, surface);
    let theme = backdrop_theme_view(&session, &role, surface);
    (session, role, surface, vector, theme)
}
fn backdrop_session(
    backdrop_role: &UiAppearanceRoleDeclaration,
) -> crate::facade::WorthUiActiveApplicationSession {
    use crate::runtime::tests::appearance_component_session_test_support::{
        appearance_component_builder, appearance_fixture, validation_background_role,
    };
    let component_role = validation_background_role("theme.appearance_consumer");
    appearance_component_builder(&component_role)
        .register_appearance_role(backdrop_role.clone())
        .unwrap()
        .with_rust_authored_declaration_fixture(appearance_fixture(&component_role))
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("backdrop resolver fixture should prepare")
        .launch()
        .expect("backdrop resolver fixture should launch")
}
fn backdrop_theme_view(
    session: &crate::facade::WorthUiActiveApplicationSession,
    role: &UiAppearanceRoleDeclaration,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
) -> UiThemeResolutionView {
    let color = crate::capability::ThemeTokenId::new("backdrop.background").unwrap();
    let opacity = crate::capability::ThemeTokenId::new("backdrop.opacity").unwrap();
    let catalog = crate::capability::UiThemeSlotCatalog::admit(
        1,
        [
            slot(color.clone(), UiThemeValueKind::Color),
            slot(opacity.clone(), UiThemeValueKind::Opacity),
        ],
    )
    .unwrap();
    let definition_identity =
        crate::capability::UiThemeDefinitionIdentity::new("theme.backdrop").unwrap();
    let definition = crate::capability::UiThemeDefinition::admit(
        definition_identity.clone(),
        1,
        &catalog,
        [
            (
                color,
                UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels([16, 32, 48, 255])),
            ),
            (
                opacity,
                UiThemeValue::Opacity(worth_ui_dsl::UiThemeOpacity::from_ratio(1, 2).unwrap()),
            ),
        ],
    )
    .unwrap();
    let bundle =
        crate::capability::FrozenAppearanceThemeCapabilities::admit(catalog, vec![definition])
            .unwrap();
    let host_profile = worth_ui_host_contract::UiHostAppearanceProfileContract::admit(
        "backdrop-resolver-host",
        1,
        worth_ui_host_contract::UiHostAppearanceMechanicFamily::ALL,
        Some(worth_ui_host_contract::UiHostPrimaryPointerKind::Mouse),
    )
    .unwrap();
    let capability =
        crate::runtime::appearance::theme::UiThemeCapabilityAdmission::from_frozen_capabilities(
            &bundle,
            &definition_identity,
            session.capabilities().appearance_roles(),
            &host_profile,
        )
        .unwrap()
        .issue(
            [role.role().clone()],
            surface,
            session.active_generation_identity(),
        )
        .unwrap();
    UiThemeResolutionView::from_capability(&capability, &bundle).unwrap()
}
fn slot(
    id: crate::capability::ThemeTokenId,
    kind: UiThemeValueKind,
) -> crate::capability::UiThemeSlotDeclaration {
    crate::capability::UiThemeSlotDeclaration::new(
        id,
        crate::capability::ThemeTokenFamily::surface(),
        kind,
        crate::capability::ThemeTokenSource::application(),
        crate::capability::UiThemeSlotDisclosure::Public,
        crate::capability::UiThemeSlotSuccessorCompatibility::ExactMeaning,
        None,
    )
}
fn declaration(
    role: &UiAppearanceRoleDeclaration,
    identity: UiBackdropIdentity,
    surface: UiSemanticSurfaceDeclarationIdentity,
) -> UiBackdropDeclaration {
    UiBackdropDeclaration::admit(
        identity,
        surface,
        UiBackdropScope::SurfaceSingleton,
        UiBackdropExtentBasis::SurfaceViewport(surface),
        UiBackdropPresenceBasis::Always,
        UiBackdropMotionBasis::None,
        UiBackdropPlacement::AboveSurfaceContent,
        role,
    )
    .unwrap()
}
fn overlay(
    session: &crate::facade::WorthUiActiveApplicationSession,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    declaration_surface: UiSemanticSurfaceDeclarationIdentity,
    instance: UiBackdropInstanceIdentity,
    include_instance: bool,
    presentation_basis: u64,
    declaration_revision: u64,
) -> UiOverlayStackSnapshot {
    let portal_state = crate::runtime::portal::UiPortalRuntimeState::new(
        crate::runtime::UiServiceStatePersistencePosture::Ephemeral,
    );
    UiOverlayStackSnapshot::seal(
        session.active_generation_identity(),
        surface,
        declaration_surface,
        presentation_basis,
        portal_state.stack_snapshot(),
        declaration_revision,
        UiOverlayStackRow::new(instance, 1)
            .into_iter()
            .filter(|_| include_instance),
    )
    .unwrap()
}
fn projection_for(
    session: &crate::facade::WorthUiActiveApplicationSession,
    role: &UiAppearanceRoleDeclaration,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    vector: &UiBackdropAppearanceStateVector,
    theme: &UiThemeResolutionView,
    identity_value: u64,
    presentation_basis: u64,
    declaration_revision: u64,
) -> Box<UiBackdropAppearanceProjection> {
    let declaration_surface = UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let identity = UiBackdropIdentity::new(identity_value).unwrap();
    let declaration = declaration(role, identity, declaration_surface);
    let instance = UiBackdropInstanceIdentity::surface_singleton(identity);
    let overlay = overlay(
        session,
        surface,
        declaration_surface,
        instance,
        true,
        presentation_basis,
        declaration_revision,
    );
    Box::new(
        UiAppearanceResolver::new()
            .resolve_backdrop(instance, &declaration, role, vector, theme, &overlay)
            .expect("sealed backdrop inputs should resolve through the resolver"),
    )
}
fn backdrop_role(identity: &str) -> UiAppearanceRoleDeclaration {
    let contract = worth_ui_dsl::UiAppearanceAspectContract::backdrop();
    let partition = |aspect: UiAppearanceAspect| {
        UiAppearanceDecisionPartition::compile(
            [],
            [worth_ui_dsl::UiAppearanceDecisionRule::new(
                [],
                UiAppearanceDecisionResult::theme_slot(
                    UiThemeSlotIdentity::new(format!("backdrop.{aspect:?}").to_ascii_lowercase())
                        .unwrap(),
                    aspect.value_kind(),
                ),
            )],
        )
        .unwrap()
    };
    UiAppearanceRoleDeclaration::admit(
        UiAppearanceRoleIdentity::new(identity).unwrap(),
        UiAppearanceRoleRevision::new(1).unwrap(),
        UiAppearanceRoleApplicability::Backdrop,
        &contract,
        [
            (
                UiAppearanceAspect::Background,
                partition(UiAppearanceAspect::Background),
            ),
            (
                UiAppearanceAspect::Opacity,
                partition(UiAppearanceAspect::Opacity),
            ),
        ],
    )
    .unwrap()
}
fn current_value(
    session: &crate::facade::WorthUiActiveApplicationSession,
) -> Option<crate::capability::ThemeTokenValue> {
    let token = crate::capability::ThemeTokenId::new("theme.appearance_consumer").unwrap();
    session
        .complete_application_theme_values_source()
        .current_value(&token)
        .cloned()
}
fn assert_zero_effects(effects: super::UiAppearanceResolutionEffectPosture) {
    assert_eq!(effects.host_commands(), 0);
    assert!(!effects.live_values_changed());
    assert!(!effects.mounted_output_changed());
}
