use std::collections::{BTreeMap, BTreeSet};

use crate::source::{
    WorthUiArtifactInputModule, WorthUiArtifactInputNode, WorthUiDslCompileReport,
    WorthUiSealedOverlayDeclarationBindings, WorthUiSourceModuleId,
};

use super::overlay_identity_resolution::{
    first_node_provenance, insert_name, portal_surface_identity, resolver_report, ResolutionDenial,
};
use super::sealed_overlay_declaration_bindings::allocate;

pub(crate) fn resolve_rust_authored_overlay_declaration_bindings(
    modules: &BTreeMap<WorthUiSourceModuleId, WorthUiArtifactInputModule>,
) -> Result<WorthUiSealedOverlayDeclarationBindings, WorthUiDslCompileReport> {
    let mut portal_names = BTreeSet::new();
    let mut surface_names = BTreeSet::new();
    for module in modules.values() {
        for node in module.nodes() {
            match node {
                WorthUiArtifactInputNode::Surface(surface) => {
                    insert_name(&mut surface_names, "surface", surface.name_text()).map_err(
                        |denial| resolver_report(denial, first_node_provenance(modules)),
                    )?;
                }
                WorthUiArtifactInputNode::SemanticArtifact(artifact) => {
                    if let Some(crate::WorthUiServiceDeclarationMeaning::Portal(portal)) =
                        artifact.declaration().service_declaration()
                    {
                        insert_name(&mut portal_names, "portal", portal.identity()).map_err(
                            |denial| resolver_report(denial, first_node_provenance(modules)),
                        )?;
                    }
                }
                _ => {}
            }
        }
    }
    let portals = allocate(portal_names, crate::UiPortalDeclarationId::new).map_err(|_| {
        resolver_report(
            ResolutionDenial::Capacity("portal"),
            first_node_provenance(modules),
        )
    })?;
    let surfaces = allocate(
        surface_names,
        crate::UiSemanticSurfaceDeclarationIdentity::new,
    )
    .map_err(|_| {
        resolver_report(
            ResolutionDenial::Capacity("surface"),
            first_node_provenance(modules),
        )
    })?;
    let bindings = WorthUiSealedOverlayDeclarationBindings::from_parts(
        BTreeMap::new(),
        portals,
        surfaces,
        BTreeMap::new(),
    );
    for module in modules.values() {
        for node in module.nodes() {
            if let WorthUiArtifactInputNode::SemanticArtifact(artifact) = node {
                if let Some(crate::WorthUiServiceDeclarationMeaning::Portal(portal)) =
                    artifact.declaration().service_declaration()
                {
                    portal_surface_identity(portal, &bindings)
                        .map_err(|denial| resolver_report(denial, Some(artifact.provenance())))?;
                }
            }
        }
    }
    Ok(bindings)
}
