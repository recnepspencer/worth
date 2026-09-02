use super::super::{sealing, WorthUiSemanticPackageSealingState};

pub(super) fn validate(state: &mut WorthUiSemanticPackageSealingState) {
    for (backdrop, provenance) in &state.backdrops {
        match state.appearance_roles.get(backdrop.role().as_str()) {
            None => state.diagnostics.push(sealing::appearance_diagnostic(
                crate::WorthUiDslCompileDiagnosticCode::MissingAppearanceDeclaration,
                "backdrop references a missing appearance role",
                provenance,
            )),
            Some((role, _))
                if !matches!(
                    role.applicability(),
                    crate::UiAppearanceRoleApplicability::Backdrop
                ) =>
            {
                state.diagnostics.push(sealing::appearance_diagnostic(
                    crate::WorthUiDslCompileDiagnosticCode::WrongAppearanceDeclarationKind,
                    "backdrop references a component appearance role",
                    provenance,
                ));
            }
            Some((role, _)) if role.revision() != backdrop.role_revision() => {
                state.diagnostics.push(sealing::appearance_diagnostic(
                    crate::WorthUiDslCompileDiagnosticCode::StaleAppearanceRoleRevision,
                    "backdrop references a stale appearance role revision",
                    provenance,
                ));
            }
            Some(_) => {}
        }
    }
}
