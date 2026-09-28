use super::{
    WorthUiSealedSemanticPackage, WorthUiSemanticBlock, WorthUiSemanticDeclaration,
    WorthUiSemanticImport, WorthUiSemanticModule, WorthUiSemanticProjectionDeclaration,
    WorthUiSemanticProvenanceRef, WorthUiSemanticToken,
};
use crate::source::{
    WorthUiArtifactInputProvenance, WorthUiArtifactInputReference, WorthUiAuthoredStructuralBody,
    WorthUiProjectionRequirement, WorthUiSourceModuleId,
};

impl<'package> super::WorthUiSemanticDeclarationView<'package> {
    pub fn declaration(&self) -> &'package WorthUiSemanticDeclaration {
        self.declaration
    }

    pub fn provenance_ref(&self) -> super::WorthUiSemanticProvenanceRef {
        self.provenance_ref
    }

    pub fn provenance(&self) -> &'package WorthUiArtifactInputProvenance {
        self.provenance
    }
}

impl WorthUiSealedSemanticPackage {
    pub fn projection_requirements(&self) -> impl Iterator<Item = &WorthUiProjectionRequirement> {
        self.canonical_module_order.iter().flat_map(|module_id| {
            self.modules[module_id].declarations.iter().filter_map(
                |declaration| match declaration {
                    WorthUiSemanticDeclaration::Projection(projection) => {
                        Some(projection.requirement())
                    }
                    _ => None,
                },
            )
        })
    }

    pub fn service_declarations(
        &self,
    ) -> impl Iterator<
        Item = (
            &crate::WorthUiServiceDeclarationMeaning,
            &WorthUiArtifactInputProvenance,
        ),
    > {
        self.canonical_module_order.iter().flat_map(|module_id| {
            self.declaration_views(module_id)
                .into_iter()
                .flatten()
                .filter_map(|view| {
                    let service = match view.declaration() {
                        WorthUiSemanticDeclaration::SemanticArtifact(artifact) => {
                            artifact.declaration().service_declaration()
                        }
                        _ => None,
                    }?;
                    Some((service, view.provenance()))
                })
        })
    }
}

impl WorthUiSemanticModule {
    pub fn module_id(&self) -> &WorthUiSourceModuleId {
        &self.module_id
    }

    pub fn declarations(&self) -> &[WorthUiSemanticDeclaration] {
        &self.declarations
    }
}

impl WorthUiSemanticImport {
    pub fn target(&self) -> &WorthUiArtifactInputReference {
        &self.target
    }

    pub fn provenance_ref(&self) -> WorthUiSemanticProvenanceRef {
        self.provenance_ref
    }
}

impl WorthUiSemanticBlock {
    pub fn name_text(&self) -> &str {
        &self.name_text
    }

    pub fn authored_identity(&self) -> Option<&str> {
        self.authored_identity.as_deref()
    }

    pub fn structure(&self) -> &WorthUiAuthoredStructuralBody {
        &self.structure
    }

    pub fn appearance_role_attachment(
        &self,
    ) -> Option<&crate::UiAppearanceRoleAttachmentDeclaration> {
        self.appearance_role_attachment.as_ref()
    }

    pub fn provenance_ref(&self) -> WorthUiSemanticProvenanceRef {
        self.provenance_ref
    }
}

impl WorthUiSemanticToken {
    pub fn name_text(&self) -> &str {
        &self.name_text
    }

    pub fn authored_identity(&self) -> Option<&str> {
        self.authored_identity.as_deref()
    }

    pub fn value_text(&self) -> &str {
        &self.value_text
    }

    pub fn provenance_ref(&self) -> WorthUiSemanticProvenanceRef {
        self.provenance_ref
    }
}

impl WorthUiSemanticProjectionDeclaration {
    pub fn requirement(&self) -> &WorthUiProjectionRequirement {
        &self.requirement
    }

    pub fn provenance_ref(&self) -> WorthUiSemanticProvenanceRef {
        self.provenance_ref
    }
}

impl WorthUiSemanticDeclaration {
    pub fn provenance_ref(&self) -> WorthUiSemanticProvenanceRef {
        match self {
            Self::Import(declaration) => declaration.provenance_ref(),
            Self::Component(declaration)
            | Self::Surface(declaration)
            | Self::Binding(declaration) => declaration.provenance_ref(),
            Self::Projection(declaration) => declaration.provenance_ref(),
            Self::Token(declaration) => declaration.provenance_ref(),
            Self::SemanticArtifact(declaration) => declaration.provenance_ref(),
            Self::AppearanceRole(declaration) => declaration.provenance_ref(),
            Self::Backdrop(declaration) => declaration.provenance_ref(),
            Self::Layout(declaration) => declaration.provenance_ref(),
            Self::Expression(declaration) => declaration.provenance_ref(),
        }
    }
}
