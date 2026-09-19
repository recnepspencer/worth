use super::{WorthUiSealedSemanticPackage, WorthUiSemanticDeclaration};
use crate::source::{WorthUiArtifactInputProvenance, WorthUiProjectionRequirement};

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
