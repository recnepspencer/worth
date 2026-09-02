use super::super::{sealing, WorthUiSemanticPackageSealingState};

pub(super) fn validate(state: &mut WorthUiSemanticPackageSealingState) {
    if state.appearance_roles.len() > crate::UI_APPEARANCE_ROLE_CAPACITY {
        if let Some((_, provenance)) = state
            .appearance_roles
            .values()
            .nth(crate::UI_APPEARANCE_ROLE_CAPACITY)
        {
            state.diagnostics.push(sealing::appearance_diagnostic(
                crate::WorthUiDslCompileDiagnosticCode::AppearanceCapacityDenied,
                "appearance role capacity denied",
                provenance,
            ));
        }
    }
    if state.backdrops.len() > crate::UI_APPEARANCE_BACKDROP_RELATION_CAPACITY {
        if let Some((_, provenance)) = state
            .backdrops
            .get(crate::UI_APPEARANCE_BACKDROP_RELATION_CAPACITY)
        {
            state.diagnostics.push(sealing::appearance_diagnostic(
                crate::WorthUiDslCompileDiagnosticCode::OverlayCapacityDenied,
                "backdrop capacity denied",
                provenance,
            ));
        }
    }
}
