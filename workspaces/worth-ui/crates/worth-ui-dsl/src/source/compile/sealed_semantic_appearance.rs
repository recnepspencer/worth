use super::{
    WorthUiSealedSemanticPackage, WorthUiSemanticDeclaration, WorthUiSemanticProvenanceRef,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiSemanticAppearanceRoleDeclaration {
    role: crate::UiAppearanceRoleDeclaration,
    provenance_ref: WorthUiSemanticProvenanceRef,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiSemanticBackdropDeclaration {
    declaration: crate::UiBackdropDeclaration,
    provenance_ref: WorthUiSemanticProvenanceRef,
}

impl WorthUiSemanticAppearanceRoleDeclaration {
    pub(super) fn new(
        role: crate::UiAppearanceRoleDeclaration,
        provenance_ref: WorthUiSemanticProvenanceRef,
    ) -> Self {
        Self {
            role,
            provenance_ref,
        }
    }

    pub fn role(&self) -> &crate::UiAppearanceRoleDeclaration {
        &self.role
    }

    pub fn provenance_ref(&self) -> WorthUiSemanticProvenanceRef {
        self.provenance_ref
    }
}

impl WorthUiSemanticBackdropDeclaration {
    pub(super) fn new(
        declaration: crate::UiBackdropDeclaration,
        provenance_ref: WorthUiSemanticProvenanceRef,
    ) -> Self {
        Self {
            declaration,
            provenance_ref,
        }
    }

    pub fn declaration(&self) -> &crate::UiBackdropDeclaration {
        &self.declaration
    }

    pub fn provenance_ref(&self) -> WorthUiSemanticProvenanceRef {
        self.provenance_ref
    }
}

impl WorthUiSealedSemanticPackage {
    pub fn overlay_relation_graph(&self) -> Option<&crate::UiOverlayRelationGraph> {
        self.overlay_relation_graph.as_ref()
    }

    pub fn appearance_role_declarations(
        &self,
    ) -> impl Iterator<Item = &WorthUiSemanticAppearanceRoleDeclaration> {
        self.canonical_module_order.iter().flat_map(|module_id| {
            self.modules[module_id].declarations.iter().filter_map(
                |declaration| match declaration {
                    WorthUiSemanticDeclaration::AppearanceRole(role) => Some(role),
                    _ => None,
                },
            )
        })
    }

    pub fn backdrop_declarations(
        &self,
    ) -> impl Iterator<Item = &WorthUiSemanticBackdropDeclaration> {
        self.canonical_module_order.iter().flat_map(|module_id| {
            self.modules[module_id].declarations.iter().filter_map(
                |declaration| match declaration {
                    WorthUiSemanticDeclaration::Backdrop(backdrop) => Some(backdrop),
                    _ => None,
                },
            )
        })
    }
}
