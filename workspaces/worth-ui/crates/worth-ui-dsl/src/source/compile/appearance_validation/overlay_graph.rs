use super::super::{sealing, WorthUiSemanticDeclaration, WorthUiSemanticPackageSealingState};

pub(super) fn validate(state: &mut WorthUiSemanticPackageSealingState) {
    let portal_facts = state
        .modules
        .values()
        .flat_map(|module| module.declarations())
        .filter_map(|declaration| {
            let WorthUiSemanticDeclaration::SemanticArtifact(artifact) = declaration else {
                return None;
            };
            let crate::WorthUiServiceDeclarationMeaning::Portal(portal) =
                artifact.declaration().service_declaration()?
            else {
                return None;
            };
            let identity = state
                .overlay_declaration_bindings
                .portal_named(portal.identity())
                .expect("sealed portal bindings contain every Portal service");
            let surface = portal.surface().map(|surface| {
                state
                    .overlay_declaration_bindings
                    .surface_named(surface)
                    .expect("validated Portal surface references are sealed")
            });
            Some((identity, surface))
        })
        .collect::<Vec<_>>();
    match crate::UiOverlayRelationGraph::admit_with_optional_surface_facts(
        portal_facts,
        state.backdrops.iter().map(|(declaration, _)| declaration),
    ) {
        Ok(graph) => state.overlay_relation_graph = Some(graph),
        Err(denial) => {
            if let Some((_, provenance)) = state.backdrops.first() {
                state
                    .diagnostics
                    .push(sealing::overlay_diagnostic(denial, provenance));
            }
        }
    }
}
