use super::super::{sealing, WorthUiSemanticPackageSealingState};

pub(super) fn validate(state: &mut WorthUiSemanticPackageSealingState) {
    let portal_ids =
        match crate::source::resolve_portal_identity_names(state.portal_identities.clone()) {
            Ok(portal_ids) => portal_ids,
            Err(()) => {
                if let Some((_, provenance)) = state.backdrops.first() {
                    state.diagnostics.push(sealing::appearance_diagnostic(
                        crate::WorthUiDslCompileDiagnosticCode::DuplicateAppearanceDeclaration,
                        "portal identity is declared more than once",
                        provenance,
                    ));
                }
                return;
            }
        };
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
