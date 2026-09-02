use super::super::*;
use super::support::{declaration, role};

#[test]
fn compiles_declared_backdrop_relations_into_canonical_runtime_anchors() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let backdrop = declaration(
        7,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::AboveSurfaceContent,
    );

    let backdrop_identity = backdrop.identity();
    let graph = UiCompiledOverlayRelationGraph::compile(surface, &[backdrop], []).unwrap();
    assert_eq!(graph.relations().len(), 1);
    let relation = graph.relations()[0];
    assert_eq!(relation.lower(), UiOverlayAnchor::SurfaceContent);
    assert_eq!(
        relation.upper(),
        UiOverlayAnchor::Backdrop(backdrop_identity)
    );
    assert_eq!(relation.kind(), UiOverlayRelationKind::Precedes);
    assert_eq!(graph.relation_for(backdrop_identity), Some(relation));
}

#[test]
fn canonical_relation_order_does_not_depend_on_declaration_order() {
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
    let forward =
        UiCompiledOverlayRelationGraph::compile(surface, &[first.clone(), second.clone()], [])
            .unwrap();
    let reverse = UiCompiledOverlayRelationGraph::compile(surface, &[second, first], []).unwrap();

    assert_eq!(forward.relations(), reverse.relations());
}

#[test]
fn rejects_cross_surface_anchors_and_cycles_before_materialization() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let foreign_surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(2).unwrap();
    let first = worth_ui_dsl::UiBackdropIdentity::new(1).unwrap();
    let second = worth_ui_dsl::UiBackdropIdentity::new(2).unwrap();
    let foreign = declaration(
        2,
        foreign_surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::AboveSurfaceContent,
    );
    let foreign_identity = foreign.identity();
    let anchored = worth_ui_dsl::UiBackdropDeclaration::admit(
        first,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropExtentBasis::SurfaceViewport(surface),
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforeBackdrop(foreign.identity()),
        &role(),
    )
    .unwrap();
    assert_eq!(
        UiCompiledOverlayRelationGraph::compile(surface, &[anchored, foreign], []),
        Err(
            UiOverlayRelationCompilationDenial::CrossSurfaceBackdropAnchor {
                source: first,
                target: foreign_identity,
            }
        )
    );

    let cycle_first = worth_ui_dsl::UiBackdropDeclaration::admit(
        first,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropExtentBasis::SurfaceViewport(surface),
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforeBackdrop(second),
        &role(),
    )
    .unwrap();
    let cycle_second = worth_ui_dsl::UiBackdropDeclaration::admit(
        second,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropExtentBasis::SurfaceViewport(surface),
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforeBackdrop(first),
        &role(),
    )
    .unwrap();
    assert_eq!(
        UiCompiledOverlayRelationGraph::compile(surface, &[cycle_first, cycle_second], []),
        Err(UiOverlayRelationCompilationDenial::Admission(
            worth_ui_dsl::UiOverlayRelationAdmissionDenial::Cycle
        ))
    );
}
