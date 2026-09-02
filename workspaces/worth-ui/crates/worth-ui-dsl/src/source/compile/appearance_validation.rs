#[path = "appearance_validation/backdrop_role.rs"]
mod backdrop_role;
#[path = "appearance_validation/capacity.rs"]
mod capacity;
#[path = "appearance_validation/component_attachment.rs"]
mod component_attachment;
#[path = "appearance_validation/overlay_graph.rs"]
mod overlay_graph;
#[path = "appearance_validation/value_kind.rs"]
mod value_kind;

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
        capacity::validate(self);
        value_kind::validate(self);
        backdrop_role::validate(self);
        component_attachment::validate(self);
        overlay_graph::validate(self);
    }
}
