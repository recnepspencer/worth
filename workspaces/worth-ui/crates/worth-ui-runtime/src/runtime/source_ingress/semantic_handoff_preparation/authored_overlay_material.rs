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
    backdrop_declarations_by_surface:
        std::collections::BTreeMap<UiSemanticSurfaceDeclarationIdentity, Box<[usize]>>,
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
        let mut backdrop_declarations_by_surface = std::collections::BTreeMap::<_, Vec<_>>::new();
        let mut portal_anchor_bindings = Vec::new();
        let overlay_declaration_bindings = package.overlay_declaration_bindings();
        for module_id in package.module_ids() {
            for view in package.declaration_views(module_id).into_iter().flatten() {
                match view.declaration() {
                    WorthUiSemanticDeclaration::Backdrop(declaration) => {
                        backdrop_declarations_by_surface
                            .entry(declaration.declaration().surface())
                            .or_default()
                            .push(backdrop_declarations.len());
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
            backdrop_declarations_by_surface: boxed_index(backdrop_declarations_by_surface),
            overlay_relation_graph: package.overlay_relation_graph().cloned(),
            overlay_declaration_bindings: package.overlay_declaration_bindings().clone(),
            portal_anchor_bindings: portal_anchor_bindings.into_boxed_slice(),
        }
    }

    pub fn backdrop_declarations(&self) -> &[WorthUiAuthoredBackdropDeclaration] {
        &self.backdrop_declarations
    }

    pub(crate) fn backdrop_declarations_for_surface(
        &self,
        surface: UiSemanticSurfaceDeclarationIdentity,
    ) -> impl Iterator<Item = &worth_ui_dsl::UiBackdropDeclaration> {
        self.backdrop_declarations_by_surface
            .get(&surface)
            .into_iter()
            .flat_map(|indices| indices.iter())
            .map(|index| {
                self.backdrop_declarations[*index]
                    .declaration()
                    .declaration()
            })
    }

    pub(crate) fn backdrop_appearance_role_identities(
        &self,
    ) -> impl Iterator<Item = worth_ui_dsl::UiAppearanceRoleIdentity> + '_ {
        self.backdrop_declarations
            .iter()
            .map(|authored| authored.declaration().declaration().role().clone())
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

fn boxed_index<K: Ord, V>(
    values: std::collections::BTreeMap<K, Vec<V>>,
) -> std::collections::BTreeMap<K, Box<[V]>> {
    values
        .into_iter()
        .map(|(key, values)| (key, values.into_boxed_slice()))
        .collect()
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

    pub(crate) fn policy(&self) -> crate::declaration::UiPortalPolicy {
        super::authored_service_policy::portal_policy(&self.portal)
    }

    pub fn provenance(&self) -> &WorthUiArtifactInputProvenance {
        &self.provenance
    }
}
