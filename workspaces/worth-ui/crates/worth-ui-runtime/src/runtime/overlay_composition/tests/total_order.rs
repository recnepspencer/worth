use std::collections::{BTreeMap, BTreeSet};

use super::super::*;
use super::independent_model::{
    model_order, ModelBackdrop, ModelBackdropScope, ModelNode, ModelPlacement, ModelPortal,
    ModelPresence,
};
use super::support::{
    declaration, input, portal, portal_snapshot_with_rows, presentation, surface_extent,
};

type BackdropDeclaration = worth_ui_dsl::UiBackdropDeclaration;
type PortalIdentity = crate::runtime::portal::UiPortalIdentity;

struct TotalOrderFixture {
    surface: worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity,
    first_declaration: worth_ui_dsl::UiPortalDeclarationId,
    second_declaration: worth_ui_dsl::UiPortalDeclarationId,
    third_declaration: worth_ui_dsl::UiPortalDeclarationId,
    first: PortalIdentity,
    second: PortalIdentity,
    third: PortalIdentity,
    declarations: Vec<BackdropDeclaration>,
    reordered: Vec<BackdropDeclaration>,
}

#[test]
fn total_order_matches_the_model_and_ignores_source_order() {
    let fixture = total_order_fixture();
    let (forward_relations, reverse_relations) = compile_total_order_relations(&fixture);
    assert_eq!(forward_relations.relations(), reverse_relations.relations());
    assert_source_order_independence(&fixture, &forward_relations);
}

fn total_order_fixture() -> TotalOrderFixture {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(90).unwrap();
    let first_declaration = worth_ui_dsl::UiPortalDeclarationId::new(901).unwrap();
    let second_declaration = worth_ui_dsl::UiPortalDeclarationId::new(902).unwrap();
    let third_declaration = worth_ui_dsl::UiPortalDeclarationId::new(903).unwrap();
    let declarations = total_order_declarations(surface, first_declaration, second_declaration);
    let reordered = declarations.iter().rev().cloned().collect::<Vec<_>>();
    TotalOrderFixture {
        surface,
        first_declaration,
        second_declaration,
        third_declaration,
        first: portal(910),
        second: portal(911),
        third: portal(912),
        declarations,
        reordered,
    }
}

fn compile_total_order_relations(
    fixture: &TotalOrderFixture,
) -> (
    UiCompiledOverlayRelationGraph,
    UiCompiledOverlayRelationGraph,
) {
    let forward = UiCompiledOverlayRelationGraph::compile(
        fixture.surface,
        &fixture.declarations,
        [
            fixture.first_declaration,
            fixture.second_declaration,
            fixture.third_declaration,
        ],
    )
    .unwrap();
    let reverse = UiCompiledOverlayRelationGraph::compile(
        fixture.surface,
        &fixture.reordered,
        [
            fixture.third_declaration,
            fixture.second_declaration,
            fixture.first_declaration,
        ],
    )
    .unwrap();
    (forward, reverse)
}

fn assert_source_order_independence(
    fixture: &TotalOrderFixture,
    relations: &UiCompiledOverlayRelationGraph,
) {
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let extent = surface_extent(fixture.surface, runtime_surface, 2);
    let portals = portal_snapshot_with_rows(
        7,
        [
            (
                fixture.first,
                None,
                runtime_surface,
                1,
                crate::runtime::portal::UiPortalLifecyclePosture::Visible,
            ),
            (
                fixture.second,
                Some(fixture.first),
                runtime_surface,
                2,
                crate::runtime::portal::UiPortalLifecyclePosture::Visible,
            ),
            (
                fixture.third,
                None,
                runtime_surface,
                3,
                crate::runtime::portal::UiPortalLifecyclePosture::Visible,
            ),
        ],
    );
    let bindings = [
        UiOverlayPortalBinding::new(fixture.first_declaration, fixture.first),
        UiOverlayPortalBinding::new(fixture.second_declaration, fixture.second),
        UiOverlayPortalBinding::new(fixture.third_declaration, fixture.third),
    ];
    let presentation = presentation();
    let forward =
        UiOverlayCompositionState::admit(fixture.declarations.clone(), 3, Default::default())
            .unwrap()
            .prepare_initial(input(&extent, &portals, &bindings, None, presentation))
            .unwrap();
    let reverse =
        UiOverlayCompositionState::admit(fixture.reordered.clone(), 3, Default::default())
            .unwrap()
            .prepare_initial(input(&extent, &portals, &bindings, None, presentation))
            .unwrap();

    assert_source_order_outputs_match(&forward, &reverse);
    assert_total_order_invariants(forward.snapshot(), relations);
    assert_matches_model(
        forward.snapshot(),
        &[fixture.first, fixture.second, fixture.third],
        &fixture.declarations,
    );
}

fn assert_source_order_outputs_match(
    forward: &UiPreparedOverlayComposition,
    reverse: &UiPreparedOverlayComposition,
) {
    assert_eq!(forward.snapshot(), reverse.snapshot());
    assert_eq!(
        forward.snapshot().participants(),
        reverse.snapshot().participants()
    );
    assert_eq!(
        forward.snapshot().semantic_digest(),
        reverse.snapshot().semantic_digest()
    );
}

fn total_order_declarations(
    surface: worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity,
    first_portal: worth_ui_dsl::UiPortalDeclarationId,
    second_portal: worth_ui_dsl::UiPortalDeclarationId,
) -> Vec<BackdropDeclaration> {
    let before_first = declaration(
        901,
        surface,
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(first_portal),
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(first_portal),
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(first_portal),
    );
    let after_first = declaration(
        902,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyAfterPortal(first_portal),
    );
    let after_backdrop = declaration(
        903,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyAfterBackdrop(after_first.identity()),
    );
    let after_second = declaration(
        904,
        surface,
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(second_portal),
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(second_portal),
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyAfterPortal(second_portal),
    );
    vec![before_first, after_first, after_backdrop, after_second]
}

fn assert_total_order_invariants(
    snapshot: &UiOverlayStackSnapshot,
    relations: &UiCompiledOverlayRelationGraph,
) {
    let identities = snapshot
        .participants()
        .iter()
        .map(UiOverlayStackParticipant::identity)
        .collect::<Vec<_>>();
    assert_eq!(
        identities.len(),
        identities.iter().copied().collect::<BTreeSet<_>>().len(),
        "every overlay participant must be emitted once"
    );
    let positions = identities
        .iter()
        .copied()
        .enumerate()
        .map(|(position, identity)| (identity, position))
        .collect::<BTreeMap<_, _>>();
    for relation in relations
        .relations()
        .iter()
        .filter(|relation| relation.kind() == UiOverlayRelationKind::ImmediatelyPrecedes)
    {
        let lower = position_for_anchor(snapshot, relation.lower());
        let upper = position_for_anchor(snapshot, relation.upper());
        assert_eq!(
            positions[&lower] + 1,
            positions[&upper],
            "declared immediate relation must remain adjacent"
        );
    }
}

fn position_for_anchor(
    snapshot: &UiOverlayStackSnapshot,
    anchor: UiOverlayAnchor,
) -> UiOverlayParticipantIdentity {
    let matches = snapshot
        .participants()
        .iter()
        .filter_map(|participant| match (anchor, participant) {
            (UiOverlayAnchor::Portal(declaration), UiOverlayStackParticipant::Portal(row))
                if row.declaration() == declaration =>
            {
                Some(participant.identity())
            }
            (UiOverlayAnchor::Backdrop(identity), UiOverlayStackParticipant::Backdrop(row))
                if row.identity().declaration() == identity =>
            {
                Some(participant.identity())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "relation anchor must identify one participant"
    );
    matches[0]
}

fn assert_matches_model(
    snapshot: &UiOverlayStackSnapshot,
    portals: &[PortalIdentity],
    declarations: &[BackdropDeclaration],
) {
    let model_portals = portals
        .iter()
        .enumerate()
        .map(|(index, _)| ModelPortal {
            declaration: 901 + index as u64,
            instance: index as u64 + 1,
            ordinal: index as u64 + 1,
        })
        .collect::<Vec<_>>();
    let expected = model_order(
        &model_portals,
        &declarations.iter().map(model_backdrop).collect::<Vec<_>>(),
    )
    .unwrap()
    .into_iter()
    .filter(|node| !matches!(node, ModelNode::Content))
    .collect::<Vec<_>>();
    let actual = snapshot
        .participants()
        .iter()
        .map(|participant| match participant {
            UiOverlayStackParticipant::Portal(row) => ModelNode::Portal {
                declaration: row.declaration().value(),
                instance: portals
                    .iter()
                    .position(|portal| *portal == row.portal())
                    .unwrap() as u64
                    + 1,
            },
            UiOverlayStackParticipant::Backdrop(row) => ModelNode::Backdrop {
                declaration: row.declaration().value(),
                instance: match row.identity().scope() {
                    UiOverlayBackdropInstanceScope::SurfaceSingleton => None,
                    UiOverlayBackdropInstanceScope::Portal(portal) => Some(
                        portals
                            .iter()
                            .position(|candidate| *candidate == portal)
                            .unwrap() as u64
                            + 1,
                    ),
                },
            },
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

fn model_backdrop(declaration: &BackdropDeclaration) -> ModelBackdrop {
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
