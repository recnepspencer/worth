use worth_ui_dsl::{
    UiOverlayRelationGraph, UiPortalDeclarationId, UiSemanticSurfaceDeclarationIdentity,
    WorthUiArtifactInputProvenance, WorthUiPortalDeclaration,
    WorthUiSealedOverlayDeclarationBindings, WorthUiSealedSemanticPackage,
    WorthUiSemanticBackdropDeclaration, WorthUiSemanticDeclaration,
};

/// Immutable compiler-owned overlay truth carried across the authored/runtime
/// handoff. Empty collections and an absent graph mean that the sealed source
/// package admitted no corresponding declarations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiAuthoredOverlayMaterial {
    backdrop_declarations: Box<[WorthUiAuthoredBackdropDeclaration]>,
    overlay_relation_graph: Option<UiOverlayRelationGraph>,
    overlay_declaration_bindings: WorthUiSealedOverlayDeclarationBindings,
    portal_anchor_bindings: Box<[WorthUiAuthoredPortalAnchorBinding]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiAuthoredBackdropDeclaration {
    declaration: WorthUiSemanticBackdropDeclaration,
    provenance: WorthUiArtifactInputProvenance,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiAuthoredPortalAnchorBinding {
    portal_declaration_id: UiPortalDeclarationId,
    surface_declaration_id: Option<UiSemanticSurfaceDeclarationIdentity>,
    portal: WorthUiPortalDeclaration,
    provenance: WorthUiArtifactInputProvenance,
}

impl WorthUiAuthoredOverlayMaterial {
    pub(super) fn from_package(package: &WorthUiSealedSemanticPackage) -> Self {
        let mut backdrop_declarations = Vec::new();
        let mut portal_anchor_bindings = Vec::new();
        let overlay_declaration_bindings = package.overlay_declaration_bindings();
        for module_id in package.module_ids() {
            for view in package.declaration_views(module_id).into_iter().flatten() {
                match view.declaration() {
                    WorthUiSemanticDeclaration::Backdrop(declaration) => {
                        backdrop_declarations.push(WorthUiAuthoredBackdropDeclaration {
                            declaration: declaration.clone(),
                            provenance: view.provenance().clone(),
                        });
                    }
                    WorthUiSemanticDeclaration::SemanticArtifact(artifact) => {
                        if let Some(worth_ui_dsl::WorthUiServiceDeclarationMeaning::Portal(
                            portal,
                        )) = artifact.declaration().service_declaration()
                        {
                            let portal_declaration_id = overlay_declaration_bindings
                                .portal_named(portal.identity())
                                .expect("sealed portal has a compiler-issued overlay identity");
                            let surface_declaration_id = portal.surface().map(|surface| {
                                overlay_declaration_bindings
                                    .surface_named(surface)
                                    .expect("validated Portal surface is compiler-issued")
                            });
                            portal_anchor_bindings.push(WorthUiAuthoredPortalAnchorBinding {
                                portal_declaration_id,
                                surface_declaration_id,
                                portal: portal.clone(),
                                provenance: view.provenance().clone(),
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
        Self {
            backdrop_declarations: backdrop_declarations.into_boxed_slice(),
            overlay_relation_graph: package.overlay_relation_graph().cloned(),
            overlay_declaration_bindings: package.overlay_declaration_bindings().clone(),
            portal_anchor_bindings: portal_anchor_bindings.into_boxed_slice(),
        }
    }

    pub fn backdrop_declarations(&self) -> &[WorthUiAuthoredBackdropDeclaration] {
        &self.backdrop_declarations
    }

    pub fn overlay_relation_graph(&self) -> Option<&UiOverlayRelationGraph> {
        self.overlay_relation_graph.as_ref()
    }

    pub fn overlay_declaration_bindings(&self) -> &WorthUiSealedOverlayDeclarationBindings {
        &self.overlay_declaration_bindings
    }

    pub fn portal_anchor_bindings(&self) -> &[WorthUiAuthoredPortalAnchorBinding] {
        &self.portal_anchor_bindings
    }

    pub fn portal_anchor_binding(
        &self,
        declaration: UiPortalDeclarationId,
    ) -> Option<&WorthUiAuthoredPortalAnchorBinding> {
        self.portal_anchor_bindings
            .iter()
            .find(|binding| binding.portal_declaration_id == declaration)
    }
}

impl WorthUiAuthoredBackdropDeclaration {
    pub fn declaration(&self) -> &WorthUiSemanticBackdropDeclaration {
        &self.declaration
    }

    pub fn provenance(&self) -> &WorthUiArtifactInputProvenance {
        &self.provenance
    }
}

impl WorthUiAuthoredPortalAnchorBinding {
    pub const fn portal_declaration_id(&self) -> UiPortalDeclarationId {
        self.portal_declaration_id
    }

    pub const fn surface_declaration_id(&self) -> Option<UiSemanticSurfaceDeclarationIdentity> {
        self.surface_declaration_id
    }

    pub fn portal(&self) -> &WorthUiPortalDeclaration {
        &self.portal
    }

    pub fn anchor(&self) -> &str {
        self.portal.anchor()
    }

    pub fn provenance(&self) -> &WorthUiArtifactInputProvenance {
        &self.provenance
    }
}
