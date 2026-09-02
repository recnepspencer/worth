use std::collections::BTreeMap;
use std::path::Path;

use crate::source::{
    WorthUiArtifactInput, WorthUiArtifactInputBlockNode, WorthUiArtifactInputModule,
    WorthUiArtifactInputNode, WorthUiArtifactInputNormalizer, WorthUiArtifactInputProvenance,
    WorthUiArtifactInputSemanticArtifactNode, WorthUiRustAuthoredArtifactInput,
    WorthUiRustAuthoredArtifactInputModule, WorthUiSourceModuleId,
};
use crate::source::{
    WorthUiArtifactInputImportNode, WorthUiArtifactInputReference, WorthUiArtifactInputTokenNode,
};

use super::worth_ui_rust_authored_artifact_input_module::WorthUiRustAuthoredDeclaration;

#[derive(Clone, Debug, Default)]
pub(crate) struct WorthUiRustAuthoredToArtifactInputLowerer;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthUiRustAuthoredInputLoweringDenial {
    InvalidModulePath,
    DuplicateModuleIdentity,
}

impl WorthUiRustAuthoredToArtifactInputLowerer {
    #[cfg(test)]
    pub(crate) fn lower(
        rust_authored_input: &WorthUiRustAuthoredArtifactInput,
    ) -> WorthUiArtifactInput {
        Self::try_lower(rust_authored_input)
            .expect("internal Rust-authored fixture should contain valid unique module paths")
    }

    pub(crate) fn try_lower(
        rust_authored_input: &WorthUiRustAuthoredArtifactInput,
    ) -> Result<WorthUiArtifactInput, WorthUiRustAuthoredInputLoweringDenial> {
        let mut modules = BTreeMap::new();
        let mut canonical_module_order = Vec::new();

        for module in rust_authored_input.modules() {
            let module_id =
                WorthUiSourceModuleId::from_relative_path(Path::new(module.relative_module_path()))
                    .map_err(|_| WorthUiRustAuthoredInputLoweringDenial::InvalidModulePath)?;
            let nodes = lower_rust_authored_module(module);
            canonical_module_order.push(module_id.clone());
            let previous_module = modules.insert(
                module_id.clone(),
                WorthUiArtifactInputModule::new(module_id, nodes),
            );
            if previous_module.is_some() {
                return Err(WorthUiRustAuthoredInputLoweringDenial::DuplicateModuleIdentity);
            }
        }

        Ok(WorthUiArtifactInputNormalizer::normalize(
            WorthUiArtifactInput::new(modules, canonical_module_order),
        ))
    }
}

fn lower_rust_authored_module(
    module: &WorthUiRustAuthoredArtifactInputModule,
) -> Vec<WorthUiArtifactInputNode> {
    module
        .declarations()
        .iter()
        .enumerate()
        .map(|(declaration_index, declaration)| {
            let provenance = WorthUiArtifactInputProvenance::rust_authored(
                module.relative_module_path(),
                declaration_index,
            );
            match declaration {
                WorthUiRustAuthoredDeclaration::Import { target_module_path } => {
                    WorthUiArtifactInputNode::Import(WorthUiArtifactInputImportNode::new(
                        WorthUiArtifactInputReference::new(target_module_path),
                        provenance,
                    ))
                }
                WorthUiRustAuthoredDeclaration::Component {
                    name_text,
                    authored_identity,
                    body_atoms,
                    appearance_role_attachment,
                } => {
                    let node = WorthUiArtifactInputBlockNode::new(
                        name_text,
                        authored_identity.clone(),
                        body_atoms.clone(),
                        provenance,
                    );
                    let node = match appearance_role_attachment.clone() {
                        Some(attachment) => node
                            .with_appearance_role_attachment(attachment)
                            .expect("a Rust-authored component has one attachment"),
                        None => node,
                    };
                    WorthUiArtifactInputNode::Component(node)
                }
                WorthUiRustAuthoredDeclaration::Surface {
                    name_text,
                    authored_identity,
                    body_atoms,
                } => WorthUiArtifactInputNode::Surface(WorthUiArtifactInputBlockNode::new(
                    name_text,
                    authored_identity.clone(),
                    body_atoms.clone(),
                    provenance,
                )),
                WorthUiRustAuthoredDeclaration::Binding {
                    name_text,
                    authored_identity,
                    body_atoms,
                } => WorthUiArtifactInputNode::Binding(WorthUiArtifactInputBlockNode::new(
                    name_text,
                    authored_identity.clone(),
                    body_atoms.clone(),
                    provenance,
                )),
                WorthUiRustAuthoredDeclaration::QueryScalar {
                    name_text,
                    body_atoms,
                } => WorthUiArtifactInputNode::QueryScalar(WorthUiArtifactInputBlockNode::new(
                    name_text,
                    None,
                    body_atoms.clone(),
                    provenance,
                )),
                WorthUiRustAuthoredDeclaration::QueryCollection {
                    name_text,
                    body_atoms,
                } => WorthUiArtifactInputNode::QueryCollection(WorthUiArtifactInputBlockNode::new(
                    name_text,
                    None,
                    body_atoms.clone(),
                    provenance,
                )),
                WorthUiRustAuthoredDeclaration::Token {
                    name_text,
                    authored_identity,
                    value_text,
                } => WorthUiArtifactInputNode::Token(WorthUiArtifactInputTokenNode::new(
                    name_text,
                    authored_identity.clone(),
                    value_text,
                    provenance,
                )),
                WorthUiRustAuthoredDeclaration::SemanticArtifact(declaration) => {
                    WorthUiArtifactInputNode::SemanticArtifact(
                        WorthUiArtifactInputSemanticArtifactNode::new(
                            declaration.clone().canonicalize(),
                            provenance,
                        ),
                    )
                }
                WorthUiRustAuthoredDeclaration::AppearanceRole(role) => {
                    WorthUiArtifactInputNode::AppearanceRole(
                        crate::WorthUiArtifactInputAppearanceRoleNode::new(
                            role.clone(),
                            provenance,
                        ),
                    )
                }
                WorthUiRustAuthoredDeclaration::Backdrop(declaration) => {
                    WorthUiArtifactInputNode::Backdrop(
                        crate::WorthUiArtifactInputBackdropNode::new(
                            declaration.clone(),
                            provenance,
                        ),
                    )
                }
            }
        })
        .collect()
}
