use super::*;

fn role(with_axis: bool) -> super::super::super::UiAppearanceRoleDeclaration {
    let contract = super::super::super::UiAppearanceAspectContract::backdrop();
    let partition = |aspect, kind| {
        let axes = with_axis
            .then(|| {
                super::super::super::UiAppearanceAxisDomain::complete(
                    super::super::super::UiAppearanceStateAxis::Validation,
                )
            })
            .into_iter()
            .collect::<Vec<_>>();
        let predicates = with_axis
            .then(|| {
                super::super::super::UiAppearanceAxisPredicate::any(
                    super::super::super::UiAppearanceStateAxis::Validation,
                )
            })
            .into_iter()
            .collect::<Vec<_>>();
        super::super::super::UiAppearanceDecisionPartition::compile(
            axes,
            [super::super::super::UiAppearanceDecisionRule::new(
                predicates,
                super::super::super::UiAppearanceDecisionResult::theme_slot(
                    super::super::super::UiThemeSlotIdentity::new(format!("backdrop.{aspect:?}"))
                        .unwrap(),
                    kind,
                ),
            )],
        )
        .unwrap()
    };
    super::super::super::UiAppearanceRoleDeclaration::admit(
        super::super::super::UiAppearanceRoleIdentity::new("backdrop.test").unwrap(),
        super::super::super::UiAppearanceRoleRevision::new(1).unwrap(),
        super::super::super::UiAppearanceRoleApplicability::Backdrop,
        &contract,
        [
            (
                super::super::super::UiAppearanceAspect::Background,
                partition(
                    super::super::super::UiAppearanceAspect::Background,
                    super::super::super::UiThemeValueKind::Color,
                ),
            ),
            (
                super::super::super::UiAppearanceAspect::Opacity,
                partition(
                    super::super::super::UiAppearanceAspect::Opacity,
                    super::super::super::UiThemeValueKind::Opacity,
                ),
            ),
        ],
    )
    .unwrap()
}

#[test]
fn backdrop_admission_rejects_foreign_and_stateful_bases() {
    let surface = crate::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let foreign = crate::UiSemanticSurfaceDeclarationIdentity::new(2).unwrap();
    let portal = crate::UiPortalDeclarationId::new(3).unwrap();
    let other_portal = crate::UiPortalDeclarationId::new(4).unwrap();
    let identity = crate::UiBackdropIdentity::new(5).unwrap();
    let valid_role = role(false);
    let stateful_role = role(true);
    let admit = |extent, presence, motion, placement, role| {
        UiBackdropDeclaration::admit(
            identity,
            surface,
            crate::UiBackdropScope::PerPortalInstance(portal),
            extent,
            presence,
            motion,
            placement,
            role,
        )
    };
    assert_eq!(
        admit(
            crate::UiBackdropExtentBasis::SurfaceViewport(foreign),
            crate::UiBackdropPresenceBasis::WhilePortalPresented(portal),
            crate::UiBackdropMotionBasis::PortalPresentation(portal),
            crate::UiBackdropPlacement::ImmediatelyBeforePortal(portal),
            &valid_role,
        ),
        Err(UiBackdropDeclarationDenial::ForeignSurfaceExtent)
    );
    assert_eq!(
        admit(
            crate::UiBackdropExtentBasis::SurfaceViewport(surface),
            crate::UiBackdropPresenceBasis::WhilePortalPresented(other_portal),
            crate::UiBackdropMotionBasis::PortalPresentation(portal),
            crate::UiBackdropPlacement::ImmediatelyBeforePortal(portal),
            &valid_role,
        ),
        Err(UiBackdropDeclarationDenial::PerPortalScopeMismatch)
    );
    assert_eq!(
        admit(
            crate::UiBackdropExtentBasis::SurfaceViewport(surface),
            crate::UiBackdropPresenceBasis::WhilePortalPresented(portal),
            crate::UiBackdropMotionBasis::PortalPresentation(portal),
            crate::UiBackdropPlacement::ImmediatelyBeforePortal(other_portal),
            &valid_role,
        ),
        Err(UiBackdropDeclarationDenial::ForeignPortalPlacement)
    );
    assert_eq!(
        admit(
            crate::UiBackdropExtentBasis::SurfaceViewport(surface),
            crate::UiBackdropPresenceBasis::WhilePortalPresented(portal),
            crate::UiBackdropMotionBasis::PortalPresentation(portal),
            crate::UiBackdropPlacement::ImmediatelyBeforePortal(portal),
            &stateful_role,
        ),
        Err(UiBackdropDeclarationDenial::IncompatibleAppearanceRole)
    );
    assert!(admit(
        crate::UiBackdropExtentBasis::SurfaceViewport(surface),
        crate::UiBackdropPresenceBasis::WhilePortalPresented(portal),
        crate::UiBackdropMotionBasis::PortalPresentation(portal),
        crate::UiBackdropPlacement::ImmediatelyBeforePortal(portal),
        &valid_role,
    )
    .is_ok());
    assert!(admit(
        crate::UiBackdropExtentBasis::SurfaceViewport(surface),
        crate::UiBackdropPresenceBasis::WhilePortalPresented(portal),
        crate::UiBackdropMotionBasis::None,
        crate::UiBackdropPlacement::ImmediatelyBeforePortal(portal),
        &valid_role,
    )
    .is_ok());
}

#[test]
fn component_contract_with_backdrop_aspects_is_not_a_backdrop_contract() {
    let component_contract = crate::UiAppearanceAspectContract::component(
        [
            crate::UiAppearanceAspect::Background,
            crate::UiAppearanceAspect::Opacity,
        ],
        [],
    )
    .unwrap();
    assert_ne!(
        component_contract,
        crate::UiAppearanceAspectContract::backdrop()
    );

    let partition = |aspect: crate::UiAppearanceAspect| {
        crate::UiAppearanceDecisionPartition::compile(
            [],
            [crate::UiAppearanceDecisionRule::new(
                [],
                crate::UiAppearanceDecisionResult::theme_slot(
                    crate::UiThemeSlotIdentity::new(format!("component.{aspect:?}")).unwrap(),
                    aspect.value_kind(),
                ),
            )],
        )
        .unwrap()
    };
    let role = crate::UiAppearanceRoleDeclaration::admit(
        crate::UiAppearanceRoleIdentity::new("component.same-aspects").unwrap(),
        crate::UiAppearanceRoleRevision::new(1).unwrap(),
        crate::UiAppearanceRoleApplicability::Component(
            crate::UiDslComponentReference::new("test.component").unwrap(),
        ),
        &component_contract,
        [
            (
                crate::UiAppearanceAspect::Background,
                partition(crate::UiAppearanceAspect::Background),
            ),
            (
                crate::UiAppearanceAspect::Opacity,
                partition(crate::UiAppearanceAspect::Opacity),
            ),
        ],
    )
    .unwrap();
    let surface = crate::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    assert_eq!(
        UiBackdropDeclaration::admit(
            crate::UiBackdropIdentity::new(1).unwrap(),
            surface,
            crate::UiBackdropScope::SurfaceSingleton,
            crate::UiBackdropExtentBasis::SurfaceViewport(surface),
            crate::UiBackdropPresenceBasis::Always,
            crate::UiBackdropMotionBasis::None,
            crate::UiBackdropPlacement::AboveSurfaceContent,
            &role,
        ),
        Err(UiBackdropDeclarationDenial::IncompatibleAppearanceRole)
    );
}
