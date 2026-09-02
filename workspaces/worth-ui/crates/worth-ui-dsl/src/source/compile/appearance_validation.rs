use super::{WorthUiSemanticDeclaration, WorthUiSemanticPackageSealingState};
use crate::source::WorthUiArtifactInputNode;

impl WorthUiSemanticPackageSealingState {
    pub(super) fn collect_appearance_declaration(
        &mut self,
        declaration: &WorthUiSemanticDeclaration,
        input: &WorthUiArtifactInputNode,
    ) {
        match declaration {
            WorthUiSemanticDeclaration::AppearanceRole(role) => {
                let identity = role.role().role().as_str().to_owned();
                if self
                    .appearance_roles
                    .insert(
                        identity,
                        (
                            role.role().clone(),
                            super::sealing::input_node_provenance(input).clone(),
                        ),
                    )
                    .is_some()
                {
                    self.diagnostics.push(super::sealing::appearance_diagnostic(
                        crate::WorthUiDslCompileDiagnosticCode::DuplicateAppearanceDeclaration,
                        "appearance role identity is declared more than once",
                        super::sealing::input_node_provenance(input),
                    ));
                }
            }
            WorthUiSemanticDeclaration::Backdrop(backdrop) => self.backdrops.push((
                backdrop.declaration().clone(),
                super::sealing::input_node_provenance(input).clone(),
            )),
            WorthUiSemanticDeclaration::SemanticArtifact(artifact) => {
                if let Some(crate::WorthUiServiceDeclarationMeaning::Portal(portal)) =
                    artifact.declaration().service_declaration()
                {
                    self.portal_identities.push(portal.identity().to_owned());
                }
            }
            _ => {}
        }
    }

    pub(super) fn validate_appearance_declarations(&mut self) {
        if self.appearance_roles.len() > crate::UI_APPEARANCE_ROLE_CAPACITY {
            if let Some((_, provenance)) = self.appearance_roles.values().next() {
                self.diagnostics.push(super::sealing::appearance_diagnostic(
                    crate::WorthUiDslCompileDiagnosticCode::AppearanceCapacityDenied,
                    "appearance role capacity denied",
                    provenance,
                ));
            }
        }
        if self.backdrops.len() > crate::UI_APPEARANCE_BACKDROP_RELATION_CAPACITY {
            if let Some((_, provenance)) = self.backdrops.first() {
                self.diagnostics.push(super::sealing::appearance_diagnostic(
                    crate::WorthUiDslCompileDiagnosticCode::OverlayCapacityDenied,
                    "backdrop capacity denied",
                    provenance,
                ));
            }
        }
        for (role, provenance) in self.appearance_roles.values() {
            if role.partitions().iter().any(|(aspect, partition)| {
                partition
                    .cells()
                    .iter()
                    .any(|cell| cell.result().value_kind() != aspect.value_kind())
            }) {
                self.diagnostics.push(super::sealing::appearance_diagnostic(
                    crate::WorthUiDslCompileDiagnosticCode::WrongAppearanceValueKind,
                    "appearance role contains a wrong-kind decision result",
                    provenance,
                ));
            }
        }
        let backdrops = self
            .backdrops
            .iter()
            .map(|(declaration, _)| declaration.clone())
            .collect::<Vec<_>>();
        for (backdrop, provenance) in &self.backdrops {
            match self.appearance_roles.get(backdrop.role().as_str()) {
                None => self.diagnostics.push(super::sealing::appearance_diagnostic(
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
                    self.diagnostics.push(super::sealing::appearance_diagnostic(
                        crate::WorthUiDslCompileDiagnosticCode::WrongAppearanceDeclarationKind,
                        "backdrop references a component appearance role",
                        provenance,
                    ))
                }
                Some((role, _)) if role.revision() != backdrop.role_revision() => {
                    self.diagnostics.push(super::sealing::appearance_diagnostic(
                        crate::WorthUiDslCompileDiagnosticCode::StaleAppearanceRoleRevision,
                        "backdrop references a stale appearance role revision",
                        provenance,
                    ))
                }
                Some(_) => {}
            }
        }
        for module in self.modules.values() {
            for declaration in module.declarations() {
                let WorthUiSemanticDeclaration::Component(component_declaration) = declaration
                else {
                    continue;
                };
                let Some(attachment) = component_declaration.appearance_role_attachment() else {
                    continue;
                };
                let provenance = self
                    .provenance_table
                    .get(component_declaration.provenance_ref().0)
                    .expect("component provenance reference is valid");
                match self.appearance_roles.get(attachment.role().as_str()) {
                    None => self.diagnostics.push(super::sealing::appearance_diagnostic(
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
                        self.diagnostics.push(super::sealing::appearance_diagnostic(
                            crate::WorthUiDslCompileDiagnosticCode::WrongAppearanceDeclarationKind,
                            "component attachment references a backdrop role",
                            provenance,
                        ))
                    }
                    Some((role, _)) if role.revision() != attachment.revision() => {
                        self.diagnostics.push(super::sealing::appearance_diagnostic(
                            crate::WorthUiDslCompileDiagnosticCode::StaleAppearanceRoleRevision,
                            "component attachment references a stale appearance role revision",
                            provenance,
                        ))
                    }
                    Some((role, _))
                        if matches!(
                            role.applicability(),
                            crate::UiAppearanceRoleApplicability::Component(target)
                                if target.as_str() != component_declaration.name_text()
                        ) =>
                    {
                        self.diagnostics.push(super::sealing::appearance_diagnostic(
                            crate::WorthUiDslCompileDiagnosticCode::InvalidAppearanceAttachment,
                            "component attachment targets a different component",
                            provenance,
                        ))
                    }
                    Some(_) => {}
                }
            }
        }
        match crate::UiStaticOverlayRelationGraph::admit(&self.portal_identities, &backdrops) {
            Ok(graph) => self.overlay_relation_graph = Some(graph),
            Err(denial) => {
                if let Some((_, provenance)) = self.backdrops.first() {
                    self.diagnostics
                        .push(super::sealing::overlay_diagnostic(denial, provenance));
                }
            }
        }
    }
}
