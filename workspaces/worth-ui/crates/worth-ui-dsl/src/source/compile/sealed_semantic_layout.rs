use super::{
    WorthUiSealedSemanticPackage, WorthUiSemanticDeclaration, WorthUiSemanticProvenanceRef,
};

/// A sealed `layout` declaration: the grid a container restates for the
/// layout it registered, and where it was authored.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiSemanticLayoutDeclaration {
    declaration: crate::UiLayoutDeclaration,
    provenance_ref: WorthUiSemanticProvenanceRef,
}

impl WorthUiSemanticLayoutDeclaration {
    pub(super) fn new(
        declaration: crate::UiLayoutDeclaration,
        provenance_ref: WorthUiSemanticProvenanceRef,
    ) -> Self {
        Self {
            declaration,
            provenance_ref,
        }
    }

    pub fn declaration(&self) -> &crate::UiLayoutDeclaration {
        &self.declaration
    }

    pub fn provenance_ref(&self) -> WorthUiSemanticProvenanceRef {
        self.provenance_ref
    }
}

impl WorthUiSealedSemanticPackage {
    pub fn layout_declarations(&self) -> impl Iterator<Item = &WorthUiSemanticLayoutDeclaration> {
        self.canonical_module_order.iter().flat_map(|module_id| {
            self.modules[module_id].declarations.iter().filter_map(
                |declaration| match declaration {
                    WorthUiSemanticDeclaration::Layout(layout) => Some(layout),
                    _ => None,
                },
            )
        })
    }
}
