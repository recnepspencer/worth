use worth_ui_dsl::{
    UiAppearanceAspect, UiAppearanceDecisionPartition, UiAppearanceDecisionResult,
    UiAppearanceRoleApplicability, UiAppearanceRoleDeclaration, UiAppearanceRoleIdentity,
    UiAppearanceRoleRevision, UiBackdropDeclaration, UiBackdropExtentBasis, UiBackdropIdentity,
    UiBackdropMotionBasis, UiBackdropPlacement, UiBackdropPresenceBasis, UiBackdropScope,
    UiSemanticSurfaceDeclarationIdentity, UiThemeSlotIdentity,
};

use super::super::state::UiAppearanceStateVector;
use super::super::state::UiAppearanceTarget;
use super::super::theme::UiThemeResolutionView;
use super::{
    UiAppearanceResolver, UiBackdropAppearanceProjection, UiBackdropInstanceIdentity,
    UiOverlayStackSnapshot, UiResolvedAppearanceAspect,
};
use crate::runtime::overlay_composition::UiOverlayStackRow;

#[test]
fn identical_backdrop_inputs_are_exactly_equivalent_and_digest_identical() {
    super::tests::run_on_appearance_fixture_stack(|| {
        let (session, role, target, vector, theme) = super::tests::inputs();
        let aspects = node_aspects(&target, &role, &vector, &theme);
        let first = projection_for(&session, &target, &vector, &theme, aspects.clone(), 7, 1, 1);
        let second = projection_for(&session, &target, &vector, &theme, aspects, 7, 1, 1);

        assert!(first.exactly_equivalent(&second));
        assert_eq!(first.semantic_digest(), second.semantic_digest());
        assert_eq!(first.catalog_revision(), theme.catalog_revision());
        let _ = session.shutdown();
    });
}

#[test]
fn backdrop_digest_carries_instance_declaration_and_overlay_evidence() {
    super::tests::run_on_appearance_fixture_stack(|| {
        let (session, role, target, vector, theme) = super::tests::inputs();
        let aspects = node_aspects(&target, &role, &vector, &theme);
        let base = projection_for(&session, &target, &vector, &theme, aspects.clone(), 7, 1, 1);
        let changed_instance =
            projection_for(&session, &target, &vector, &theme, aspects.clone(), 8, 1, 1);
        let changed_overlay =
            projection_for(&session, &target, &vector, &theme, aspects.clone(), 7, 2, 1);
        let changed_declaration_revision =
            projection_for(&session, &target, &vector, &theme, aspects, 7, 1, 2);

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

fn projection_for(
    session: &crate::facade::WorthUiActiveApplicationSession,
    target: &UiAppearanceTarget,
    vector: &UiAppearanceStateVector,
    theme: &UiThemeResolutionView,
    aspects: Box<[UiResolvedAppearanceAspect]>,
    identity_value: u64,
    presentation_basis: u64,
    declaration_revision: u64,
) -> Box<UiBackdropAppearanceProjection> {
    let declaration_surface = UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let identity = UiBackdropIdentity::new(identity_value).unwrap();
    let role = backdrop_role();
    let declaration = UiBackdropDeclaration::admit(
        identity,
        declaration_surface,
        UiBackdropScope::SurfaceSingleton,
        UiBackdropExtentBasis::SurfaceViewport(declaration_surface),
        UiBackdropPresenceBasis::Always,
        UiBackdropMotionBasis::None,
        UiBackdropPlacement::AboveSurfaceContent,
        &role,
    )
    .unwrap();
    let instance = UiBackdropInstanceIdentity::surface_singleton(identity);
    let portal_state = crate::runtime::portal::UiPortalRuntimeState::new(
        crate::runtime::UiServiceStatePersistencePosture::Ephemeral,
    );
    let row = UiOverlayStackRow::new(instance, 1).unwrap();
    let overlay = UiOverlayStackSnapshot::seal(
        session.active_generation_identity(),
        target.surface(),
        declaration_surface,
        presentation_basis,
        portal_state.stack_snapshot(),
        declaration_revision,
        [row],
    )
    .unwrap();
    Box::new(UiBackdropAppearanceProjection::seal(
        instance,
        &declaration,
        vector.clone(),
        theme,
        overlay,
        aspects,
    ))
}

fn node_aspects(
    target: &UiAppearanceTarget,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    vector: &UiAppearanceStateVector,
    theme: &UiThemeResolutionView,
) -> Box<[UiResolvedAppearanceAspect]> {
    UiAppearanceResolver::new()
        .resolve_node(target, role, vector, theme)
        .unwrap()
        .aspects()
        .to_vec()
        .into_boxed_slice()
}

fn backdrop_role() -> UiAppearanceRoleDeclaration {
    let contract = worth_ui_dsl::UiAppearanceAspectContract::backdrop();
    let partition = |aspect: UiAppearanceAspect| {
        UiAppearanceDecisionPartition::compile(
            [],
            [worth_ui_dsl::UiAppearanceDecisionRule::new(
                [],
                UiAppearanceDecisionResult::theme_slot(
                    UiThemeSlotIdentity::new(format!("backdrop.{aspect:?}")).unwrap(),
                    aspect.value_kind(),
                ),
            )],
        )
        .unwrap()
    };
    UiAppearanceRoleDeclaration::admit(
        UiAppearanceRoleIdentity::new("test.backdrop.digest").unwrap(),
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
