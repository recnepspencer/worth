use super::super::{sealing, WorthUiSemanticPackageSealingState};

pub(super) fn validate(state: &mut WorthUiSemanticPackageSealingState) {
    let portal_ids = state.overlay_declaration_bindings.portal_ids();
    match crate::UiOverlayRelationGraph::admit_with_backdrop_surface_facts(
        portal_ids,
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
