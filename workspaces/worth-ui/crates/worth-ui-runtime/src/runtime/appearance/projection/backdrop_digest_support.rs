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
    UiAppearanceResolver, UiBackdropAppearanceProjection, UiBackdropInstanceIdentity,
    UiOverlayStackSnapshot,
};
use crate::runtime::overlay_composition::{
    UiOverlayApplicationGeneration, UiOverlayBackdropRow, UiOverlayExtent,
    UiOverlayStackParticipant,
};

pub(super) fn inputs() -> (
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

pub(super) fn backdrop_session(
    backdrop_role: &UiAppearanceRoleDeclaration,
) -> crate::facade::WorthUiActiveApplicationSession {
    use crate::runtime::tests::appearance_component_session_test_support::{
        appearance_component_builder, appearance_fixture, validation_background_role,
    };
    let component_role = validation_background_role("theme.appearance_consumer");
    appearance_component_builder(&component_role)
        .register_appearance_role(backdrop_role.clone())
        .unwrap()
        .register_appearance_theme_bundle(
            crate::runtime::tests::appearance_theme_test_support::bundle(),
        )
        .unwrap()
        .with_rust_authored_declaration_fixture(appearance_fixture(&component_role))
        .freeze()
        .map(crate::runtime::tests::appearance_theme_test_support::activate)
        .expect("backdrop resolver fixture should prepare")
        .launch()
        .expect("backdrop resolver fixture should launch")
}

pub(super) fn backdrop_theme_view(
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
    let bundle = crate::capability::FrozenAppearanceThemeCapabilities::admit(
        catalog,
        definition_identity.clone(),
        vec![definition],
    )
    .unwrap();
    let host_profile = worth_ui_host_contract::UiHostAppearanceProfileContract::admit(
        "backdrop-resolver-host",
        1,
        [
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::SurfaceFill,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::SurfaceBorder,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::CornerRadii,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::Outline,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::TextRangeForeground,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::PortalSurface,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::Backdrop,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::OverlayOrder,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::PointerAffordance,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::Damage,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::Clip,
        ],
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

pub(super) fn slot(
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

pub(super) fn declaration(
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

pub(super) fn overlay(
    session: &crate::facade::WorthUiActiveApplicationSession,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    declaration_surface: UiSemanticSurfaceDeclarationIdentity,
    declaration: &UiBackdropDeclaration,
    instance: UiBackdropInstanceIdentity,
    include_instance: bool,
    presentation_basis: u64,
    declaration_revision: u64,
) -> UiOverlayStackSnapshot {
    let bounds = worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
            coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
        },
    )
    .unwrap();
    let row = UiOverlayBackdropRow::for_test(
        instance,
        declaration,
        UiOverlayExtent::SurfaceViewport {
            basis: UiBackdropExtentBasis::SurfaceViewport(declaration_surface),
            bounds,
        },
    );
    UiOverlayStackSnapshot::seal_for_test(
        UiOverlayApplicationGeneration::from_prepared(session.generation_identity().clone()),
        declaration_surface,
        surface,
        test_presentation_attempt(),
        presentation_basis,
        declaration_revision,
        1,
        None,
        include_instance.then_some(UiOverlayStackParticipant::Backdrop(row)),
    )
}

fn test_presentation_attempt() -> worth_ui_host_contract::UiMountedPresentationAttemptIdentity {
    use std::sync::OnceLock;
    static ATTEMPT: OnceLock<worth_ui_host_contract::UiMountedPresentationAttemptIdentity> =
        OnceLock::new();
    *ATTEMPT.get_or_init(|| {
        worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap()
    })
}

pub(super) fn projection_for(
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
        &declaration,
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

pub(super) fn backdrop_role(identity: &str) -> UiAppearanceRoleDeclaration {
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

pub(super) fn current_value(
    session: &crate::facade::WorthUiActiveApplicationSession,
) -> Option<crate::capability::ThemeTokenValue> {
    let token = crate::capability::ThemeTokenId::new("theme.appearance_consumer").unwrap();
    session
        .complete_application_theme_values_source()
        .current_value(&token)
        .cloned()
}
