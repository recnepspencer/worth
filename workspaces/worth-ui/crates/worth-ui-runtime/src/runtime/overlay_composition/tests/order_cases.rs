use super::super::*;
use super::independent_model::{
    model_order, ModelBackdrop, ModelBackdropScope, ModelDenial, ModelPlacement, ModelPortal,
    ModelPresence,
};
use super::support::{
    declaration, input, portal, portal_snapshot, portal_snapshot_with_rows, presentation,
    surface_extent,
};

fn production_nodes(
    snapshot: &UiOverlayStackSnapshot,
    portals: &[(crate::runtime::portal::UiPortalIdentity, u64, u64)],
) -> Vec<super::independent_model::ModelNode> {
    snapshot
        .participants()
        .iter()
        .map(|participant| match participant {
            UiOverlayStackParticipant::Portal(row) => {
                let (_, declaration, instance) = portals
                    .iter()
                    .find(|(portal, _, _)| *portal == row.portal())
                    .expect("model portal mapping remains complete");
                super::independent_model::ModelNode::Portal {
                    declaration: *declaration,
                    instance: *instance,
                }
            }
            UiOverlayStackParticipant::Backdrop(row) => {
                let instance = match row.identity().scope() {
                    UiOverlayBackdropInstanceScope::SurfaceSingleton => None,
                    UiOverlayBackdropInstanceScope::Portal(portal) => Some(
                        portals
                            .iter()
                            .find(|(candidate, _, _)| *candidate == portal)
                            .expect("model backdrop mapping remains complete")
                            .2,
                    ),
                };
                super::independent_model::ModelNode::Backdrop {
                    declaration: row.declaration().value(),
                    instance,
                }
            }
        })
        .collect()
}

fn model_backdrop(declaration: &worth_ui_dsl::UiBackdropDeclaration) -> ModelBackdrop {
    let scope = match declaration.scope() {
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton => ModelBackdropScope::Surface,
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(portal) => {
            ModelBackdropScope::PerPortalDeclaration(portal.value())
        }
    };
    let presence = match declaration.presence() {
        worth_ui_dsl::UiBackdropPresenceBasis::Always => ModelPresence::Always,
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(portal) => {
            ModelPresence::WhilePortalPresented(portal.value())
        }
    };
    let placement = match declaration.placement() {
        worth_ui_dsl::UiBackdropPlacement::AboveSurfaceContent => ModelPlacement::AboveContent,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(portal) => {
            ModelPlacement::BeforePortal(portal.value())
        }
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyAfterPortal(portal) => {
            ModelPlacement::AfterPortal(portal.value())
        }
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforeBackdrop(backdrop) => {
            ModelPlacement::BeforeBackdrop(backdrop.value())
        }
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyAfterBackdrop(backdrop) => {
            ModelPlacement::AfterBackdrop(backdrop.value())
        }
    };
    ModelBackdrop {
        declaration: declaration.identity().value(),
        scope,
        presence,
        placement,
    }
}

fn assert_model_order(
    prepared: &UiPreparedOverlayComposition,
    portals: &[(crate::runtime::portal::UiPortalIdentity, u64, u64)],
    model_portals: &[ModelPortal],
    declarations: &[worth_ui_dsl::UiBackdropDeclaration],
) {
    let expected = model_order(
        model_portals,
        &declarations.iter().map(model_backdrop).collect::<Vec<_>>(),
    )
    .unwrap()
    .into_iter()
    .filter(|node| !matches!(node, super::independent_model::ModelNode::Content))
    .collect::<Vec<_>>();
    assert_eq!(production_nodes(prepared.snapshot(), portals), expected);
}

#[test]
fn ordered_stack_matches_model_for_nested_sibling_portal_anchors() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let portal_declaration = worth_ui_dsl::UiPortalDeclarationId::new(10).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let first = portal(20);
    let second = portal(21);
    let third = portal(22);
    let before = declaration(
        1,
        surface,
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(portal_declaration),
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(portal_declaration),
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(portal_declaration),
    );
    let after = declaration(
        2,
        surface,
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(portal_declaration),
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(portal_declaration),
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyAfterPortal(portal_declaration),
    );
    let snapshot = portal_snapshot_with_rows(
        7,
        [
            (
                first,
                None,
                runtime_surface,
                1,
                crate::runtime::portal::UiPortalLifecyclePosture::Visible,
            ),
            (
                second,
                Some(first),
                runtime_surface,
                2,
                crate::runtime::portal::UiPortalLifecyclePosture::Visible,
            ),
            (
                third,
                None,
                runtime_surface,
                3,
                crate::runtime::portal::UiPortalLifecyclePosture::Visible,
            ),
        ],
    );
    let bindings = [
        UiOverlayPortalBinding::new(portal_declaration, first),
        UiOverlayPortalBinding::new(portal_declaration, second),
        UiOverlayPortalBinding::new(portal_declaration, third),
    ];
    let extent = surface_extent(surface, runtime_surface, 2);
    let declarations = [before, after];
    let state =
        UiOverlayCompositionState::admit(declarations.clone(), 3, Default::default()).unwrap();
    let prepared = state
        .prepare_initial(input(&extent, &snapshot, &bindings, None, presentation()))
        .unwrap();

    assert_model_order(
        &prepared,
        &[(first, 10, 1), (second, 10, 2), (third, 10, 3)],
        &[
            ModelPortal {
                declaration: 10,
                instance: 1,
                ordinal: 1,
            },
            ModelPortal {
                declaration: 10,
                instance: 2,
                ordinal: 2,
            },
            ModelPortal {
                declaration: 10,
                instance: 3,
                ordinal: 3,
            },
        ],
        &declarations,
    );
}

#[test]
fn ordered_stack_matches_model_for_before_and_after_backdrop_anchors() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let first = declaration(
        1,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::AboveSurfaceContent,
    );
    let before = declaration(
        2,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforeBackdrop(first.identity()),
    );
    let after = declaration(
        3,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyAfterBackdrop(first.identity()),
    );
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let extent = surface_extent(surface, runtime_surface, 2);
    let portals = portal_snapshot(runtime_surface, []);
    let declarations = [first, before, after];
    let state =
        UiOverlayCompositionState::admit(declarations.clone(), 3, Default::default()).unwrap();
    let prepared = state
        .prepare_initial(input(&extent, &portals, &[], None, presentation()))
        .unwrap();

    assert_model_order(&prepared, &[], &[], &declarations);
}

#[test]
fn independent_model_and_compiler_agree_on_ambiguity_and_cycle_denials() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let first = declaration(
        1,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::AboveSurfaceContent,
    );
    let second = declaration(
        2,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::AboveSurfaceContent,
    );
    assert_eq!(
        model_order(&[], &[model_backdrop(&first), model_backdrop(&second)]),
        Err(ModelDenial::AmbiguousOrder)
    );
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let extent = surface_extent(surface, runtime_surface, 2);
    let portals = portal_snapshot(runtime_surface, []);
    let state =
        UiOverlayCompositionState::admit([first.clone(), second.clone()], 3, Default::default())
            .unwrap();
    assert_eq!(
        state.prepare_initial(input(&extent, &portals, &[], None, presentation())),
        Err(UiOverlayCompositionDenial::AmbiguousOrder)
    );

    let cycle_first = declaration(
        3,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforeBackdrop(
            worth_ui_dsl::UiBackdropIdentity::new(4).unwrap(),
        ),
    );
    let cycle_second = declaration(
        4,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforeBackdrop(
            worth_ui_dsl::UiBackdropIdentity::new(3).unwrap(),
        ),
    );
    assert_eq!(
        model_order(
            &[],
            &[model_backdrop(&cycle_first), model_backdrop(&cycle_second)]
        ),
        Err(ModelDenial::Cycle)
    );
    assert_eq!(
        UiCompiledOverlayRelationGraph::compile(surface, &[cycle_first, cycle_second], []),
        Err(UiOverlayRelationCompilationDenial::Admission(
            worth_ui_dsl::UiOverlayRelationAdmissionDenial::Cycle
        ))
    );
}
