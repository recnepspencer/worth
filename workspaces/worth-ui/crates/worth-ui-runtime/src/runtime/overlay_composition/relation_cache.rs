use std::collections::{BTreeMap, BTreeSet};

use worth_ui_dsl::{UiBackdropDeclaration, UiSemanticSurfaceDeclarationIdentity};

use super::relation_graph::{UiCompiledOverlayRelationGraph, UiOverlayRelationCompilationDenial};

pub(super) type UiOverlayRelationCache =
    BTreeMap<UiSemanticSurfaceDeclarationIdentity, UiCompiledOverlayRelationGraph>;

pub(super) fn build(
    declarations: &[UiBackdropDeclaration],
) -> Result<UiOverlayRelationCache, UiOverlayRelationCompilationDenial> {
    let surfaces = declarations
        .iter()
        .map(UiBackdropDeclaration::surface)
        .collect::<BTreeSet<_>>();
    surfaces
        .into_iter()
        .map(|surface| {
            UiCompiledOverlayRelationGraph::compile(surface, declarations, [])
                .map(|graph| (surface, graph))
        })
        .collect()
}

pub(super) fn for_surface(
    cache: &UiOverlayRelationCache,
    surface: UiSemanticSurfaceDeclarationIdentity,
) -> UiCompiledOverlayRelationGraph {
    cache.get(&surface).cloned().unwrap_or_else(|| {
        UiCompiledOverlayRelationGraph::compile(surface, &[], [])
            .expect("an empty overlay relation graph is always valid")
    })
}
