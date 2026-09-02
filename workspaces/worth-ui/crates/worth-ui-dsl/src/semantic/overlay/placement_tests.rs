use super::*;

fn identity(value: u64) -> super::super::UiBackdropIdentity {
    super::super::UiBackdropIdentity::new(value).unwrap()
}

#[test]
fn relation_admission_rejects_cycles_and_immediate_conflicts() {
    let a = identity(1);
    let b = identity(2);
    assert_eq!(
        UiOverlayRelationGraph::admit(
            [],
            [
                (a, UiBackdropPlacement::ImmediatelyBeforeBackdrop(b)),
                (b, UiBackdropPlacement::ImmediatelyBeforeBackdrop(a))
            ]
        ),
        Err(UiOverlayRelationAdmissionDenial::Cycle)
    );
    let portal = super::super::UiPortalDeclarationId::new(7).unwrap();
    let c = identity(3);
    assert_eq!(
        UiOverlayRelationGraph::admit(
            [portal],
            [
                (a, UiBackdropPlacement::ImmediatelyBeforePortal(portal)),
                (c, UiBackdropPlacement::ImmediatelyBeforePortal(portal)),
            ],
        ),
        Err(UiOverlayRelationAdmissionDenial::ConflictingImmediateAdjacency)
    );
}

#[test]
fn typed_surface_facts_reject_foreign_relations() {
    let surface_a = super::super::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let surface_b = super::super::UiSemanticSurfaceDeclarationIdentity::new(2).unwrap();
    let a = identity(1);
    let b = identity(2);
    assert!(super::validation::different_surfaces(
        UiOverlayAnchor::Backdrop(a),
        UiOverlayAnchor::Backdrop(b),
        &[
            UiOverlayParticipantSurface {
                anchor: UiOverlayAnchor::Backdrop(a),
                surface: surface_a,
            },
            UiOverlayParticipantSurface {
                anchor: UiOverlayAnchor::Backdrop(b),
                surface: surface_b,
            },
        ],
    ));
    let portal = super::super::UiPortalDeclarationId::new(1).unwrap();
    assert_eq!(
        UiOverlayPortalParticipant::new(portal, surface_a).surface(),
        surface_a
    );
}

#[test]
fn relation_kind_is_part_of_canonical_graph_bytes() {
    let surface = super::super::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let backdrop = identity(1);
    let ordinary = UiOverlayRelationGraph {
        relations: Box::new([UiOverlayRelation {
            lower: UiOverlayAnchor::SurfaceContentOn(surface),
            upper: UiOverlayAnchor::Backdrop(backdrop),
            kind: UiOverlayRelationKind::Precedes,
        }]),
        participant_surfaces: Box::new([]),
    };
    let immediate = UiOverlayRelationGraph {
        relations: Box::new([UiOverlayRelation {
            lower: UiOverlayAnchor::SurfaceContentOn(surface),
            upper: UiOverlayAnchor::Backdrop(backdrop),
            kind: UiOverlayRelationKind::ImmediatelyPrecedes,
        }]),
        participant_surfaces: Box::new([]),
    };
    assert_ne!(ordinary.canonical_bytes(), immediate.canonical_bytes());
}

#[test]
fn relation_admission_enforces_backdrop_capacity() {
    let backdrops =
        (1..=4_097).map(|value| (identity(value), UiBackdropPlacement::AboveSurfaceContent));
    assert_eq!(
        UiOverlayRelationGraph::admit([], backdrops),
        Err(UiOverlayRelationAdmissionDenial::BackdropCapacityExceeded)
    );
}

fn backdrop_role() -> crate::UiAppearanceRoleDeclaration {
    let partition = |aspect, kind| {
        crate::UiAppearanceDecisionPartition::compile(
            [],
            [crate::UiAppearanceDecisionRule::new(
                [],
                crate::UiAppearanceDecisionResult::theme_slot(
                    crate::UiThemeSlotIdentity::new(format!("overlay.{aspect:?}")).unwrap(),
                    kind,
                ),
            )],
        )
        .unwrap()
    };
    crate::UiAppearanceRoleDeclaration::admit(
        crate::UiAppearanceRoleIdentity::new("overlay.test").unwrap(),
        crate::UiAppearanceRoleRevision::new(1).unwrap(),
        crate::UiAppearanceRoleApplicability::Backdrop,
        &crate::UiAppearanceAspectContract::backdrop(),
        [
            (
                crate::UiAppearanceAspect::Background,
                partition(
                    crate::UiAppearanceAspect::Background,
                    crate::UiThemeValueKind::Color,
                ),
            ),
            (
                crate::UiAppearanceAspect::Opacity,
                partition(
                    crate::UiAppearanceAspect::Opacity,
                    crate::UiThemeValueKind::Opacity,
                ),
            ),
        ],
    )
    .unwrap()
}

fn backdrop(
    value: u64,
    surface: super::super::UiSemanticSurfaceDeclarationIdentity,
    placement: UiBackdropPlacement,
) -> super::super::UiBackdropDeclaration {
    crate::UiBackdropDeclaration::admit(
        identity(value),
        surface,
        crate::UiBackdropScope::SurfaceSingleton,
        crate::UiBackdropExtentBasis::SurfaceViewport(surface),
        crate::UiBackdropPresenceBasis::Always,
        crate::UiBackdropMotionBasis::None,
        placement,
        &backdrop_role(),
    )
    .unwrap()
}

#[test]
fn typed_admission_rejects_missing_foreign_and_ambiguous_anchors() {
    let surface_a = super::super::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let surface_b = super::super::UiSemanticSurfaceDeclarationIdentity::new(2).unwrap();
    let missing = identity(2);
    assert_eq!(
        UiOverlayRelationGraph::admit(
            [],
            [(
                identity(1),
                UiBackdropPlacement::ImmediatelyBeforeBackdrop(missing)
            )],
        ),
        Err(UiOverlayRelationAdmissionDenial::MissingAnchor)
    );

    let foreign = backdrop(
        1,
        surface_a,
        UiBackdropPlacement::ImmediatelyBeforeBackdrop(identity(2)),
    );
    let other_surface = backdrop(2, surface_b, UiBackdropPlacement::AboveSurfaceContent);
    assert_eq!(
        UiOverlayRelationGraph::admit_with_backdrop_surface_facts([], [&foreign, &other_surface]),
        Err(UiOverlayRelationAdmissionDenial::ForeignSurfaceAnchor)
    );

    let first = backdrop(1, surface_a, UiBackdropPlacement::AboveSurfaceContent);
    let second = backdrop(2, surface_a, UiBackdropPlacement::AboveSurfaceContent);
    assert_eq!(
        UiOverlayRelationGraph::admit_with_backdrop_surface_facts([], [&first, &second]),
        Err(UiOverlayRelationAdmissionDenial::AmbiguousOrder)
    );
}
