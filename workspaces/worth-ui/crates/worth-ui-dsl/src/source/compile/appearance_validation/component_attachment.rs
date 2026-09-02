use super::super::{sealing, WorthUiSemanticPackageSealingState};

pub(super) fn validate(state: &mut WorthUiSemanticPackageSealingState) {
    for module in state.modules.values() {
        for declaration in module.declarations() {
            let crate::source::compile::WorthUiSemanticDeclaration::Component(
                component_declaration,
            ) = declaration
            else {
                continue;
            };
            let Some(attachment) = component_declaration.appearance_role_attachment() else {
                continue;
            };
            let provenance = state
                .provenance_table
                .get(component_declaration.provenance_ref().0)
                .expect("component provenance reference is valid");
            match state.appearance_roles.get(attachment.role().as_str()) {
                None => state.diagnostics.push(sealing::appearance_diagnostic(
                    crate::WorthUiDslCompileDiagnosticCode::MissingAppearanceDeclaration,
                    "component attachment references a missing appearance role",
                    provenance,
                )),
                Some((role, _))
                    if matches!(
                        role.applicability(),
                        crate::UiAppearanceRoleApplicability::Backdrop
                    ) =>
                {
                    state.diagnostics.push(sealing::appearance_diagnostic(
                        crate::WorthUiDslCompileDiagnosticCode::WrongAppearanceDeclarationKind,
                        "component attachment references a backdrop role",
                        provenance,
                    ));
                }
                Some((role, _)) if role.revision() != attachment.revision() => {
                    state.diagnostics.push(sealing::appearance_diagnostic(
                        crate::WorthUiDslCompileDiagnosticCode::StaleAppearanceRoleRevision,
                        "component attachment references a stale appearance role revision",
                        provenance,
                    ));
                }
                Some((role, _))
                    if matches!(
                        role.applicability(),
                        crate::UiAppearanceRoleApplicability::Component(target)
                            if target.as_str() != component_declaration.name_text()
                    ) =>
                {
                    state.diagnostics.push(sealing::appearance_diagnostic(
                        crate::WorthUiDslCompileDiagnosticCode::InvalidAppearanceAttachment,
                        "component attachment targets a different component",
                        provenance,
                    ));
                }
                Some(_) => {}
            }
        }
    }
}
