use super::{WorthUiSemanticDeclaration, WorthUiSemanticPackageSealingState};

pub(super) fn validate(state: &mut WorthUiSemanticPackageSealingState) {
    let mut missing = Vec::new();
    for module in state.modules.values() {
        for declaration in module.declarations() {
            let WorthUiSemanticDeclaration::Component(component) = declaration else {
                continue;
            };
            let provenance = state.provenance_table[component.provenance_ref().0].clone();
            for route in component.structure().interaction_routes() {
                let Some(portal) = route.opened_portal_identity() else {
                    continue;
                };
                if state
                    .overlay_declaration_bindings
                    .portal_named(portal)
                    .is_none()
                {
                    missing.push((portal.to_owned(), provenance.clone()));
                }
            }
        }
    }
    for (portal, provenance) in missing {
        state
            .diagnostics
            .push(super::sealing::appearance_diagnostic(
                crate::WorthUiDslCompileDiagnosticCode::MissingOverlayAnchor,
                format!("interaction route references missing portal identity '{portal}'"),
                &provenance,
            ));
    }
}
