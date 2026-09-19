use std::collections::BTreeMap;

use crate::capability::CapabilitySnapshot;
use crate::source::{
    WorthUiResolutionDiagnostic, WorthUiResolutionReport, WorthUiResolvedArtifactInput,
    WorthUiResolvedArtifactInputBindingNode, WorthUiResolvedArtifactInputComponentNode,
    WorthUiResolvedArtifactInputModule, WorthUiResolvedArtifactInputNode,
    WorthUiResolvedArtifactInputSurfaceNode, WorthUiResolvedArtifactInputThemeTokenNode,
    WorthUiResolvedThemeTokenBindingTarget, WorthUiRuntimeSemanticImport,
};
use worth_ui_dsl::{
    WorthUiSealedSemanticPackage, WorthUiSemanticDeclaration, WorthUiSourceModuleId,
};

use super::worth_ui_snapshot_resolution_context::WorthUiSnapshotResolutionContext;

#[derive(Clone, Debug, Default)]
pub(crate) struct WorthUiArtifactInputResolver;

impl WorthUiArtifactInputResolver {
    pub(crate) fn resolve(
        sealed_package: &WorthUiSealedSemanticPackage,
        snapshot: &CapabilitySnapshot,
    ) -> Result<WorthUiResolvedArtifactInput, WorthUiResolutionReport> {
        let mut resolution_context = WorthUiSnapshotResolutionContext::new(snapshot);
        let mut modules = BTreeMap::new();
        let mut diagnostics = Vec::<WorthUiResolutionDiagnostic>::new();

        for module_id in sealed_package.module_ids() {
            let declarations = sealed_package
                .declaration_views(module_id)
                .expect("sealed semantic package should contain every canonical module");
            let mut resolved_nodes = Vec::new();

            for declaration in declarations {
                match resolve_node(
                    module_id,
                    declaration.declaration(),
                    declaration.provenance(),
                    &mut resolution_context,
                ) {
                    Ok(Some(resolved_node)) => resolved_nodes.push(resolved_node),
                    Ok(None) => {}
                    Err(diagnostic) => diagnostics.push(diagnostic),
                }
            }

            modules.insert(
                module_id.clone(),
                WorthUiResolvedArtifactInputModule::new(module_id.clone(), resolved_nodes),
            );
        }

        let metrics = resolution_context.finish_metrics();
        if !diagnostics.is_empty() {
            return Err(WorthUiResolutionReport::new(diagnostics, metrics));
        }

        Ok(WorthUiResolvedArtifactInput::new(
            modules,
            sealed_package.module_ids().to_vec(),
        ))
    }
}

fn resolve_node(
    module_id: &WorthUiSourceModuleId,
    node: &WorthUiSemanticDeclaration,
    provenance: &worth_ui_dsl::WorthUiArtifactInputProvenance,
    resolution_context: &mut WorthUiSnapshotResolutionContext<'_>,
) -> Result<Option<WorthUiResolvedArtifactInputNode>, WorthUiResolutionDiagnostic> {
    match node {
        WorthUiSemanticDeclaration::Import(import_node) => {
            Ok(Some(WorthUiResolvedArtifactInputNode::Import(
                WorthUiRuntimeSemanticImport::new(import_node.target().clone(), provenance.clone()),
            )))
        }
        WorthUiSemanticDeclaration::Component(component_node) => {
            let (component, descriptor) = resolution_context.resolve_component(
                module_id,
                component_node.name_text(),
                provenance,
            )?;
            Ok(Some(WorthUiResolvedArtifactInputNode::Component(
                WorthUiResolvedArtifactInputComponentNode::new(
                    component,
                    descriptor,
                    component_node.authored_identity().map(str::to_owned),
                    component_node.structure().clone(),
                    provenance.clone(),
                ),
            )))
        }
        WorthUiSemanticDeclaration::Surface(surface_node) => {
            let (surface, descriptor) = resolution_context.resolve_surface(
                module_id,
                surface_node.name_text(),
                provenance,
            )?;
            Ok(Some(WorthUiResolvedArtifactInputNode::Surface(
                WorthUiResolvedArtifactInputSurfaceNode::new(
                    surface,
                    descriptor,
                    surface_node.authored_identity().map(str::to_owned),
                    surface_node.structure().clone(),
                    provenance.clone(),
                ),
            )))
        }
        WorthUiSemanticDeclaration::Binding(binding_node) => {
            let (view_binding, entry) = resolution_context.resolve_view_binding(
                module_id,
                binding_node.name_text(),
                provenance,
            )?;
            Ok(Some(WorthUiResolvedArtifactInputNode::Binding(
                WorthUiResolvedArtifactInputBindingNode::new(
                    view_binding,
                    entry,
                    binding_node.authored_identity().map(str::to_owned),
                    binding_node.structure().clone(),
                    provenance.clone(),
                ),
            )))
        }
        WorthUiSemanticDeclaration::Token(token_node) => {
            let (theme_token, entry) = resolution_context.resolve_theme_token(
                module_id,
                token_node.name_text(),
                provenance,
            )?;
            let binding_target = WorthUiResolvedThemeTokenBindingTarget::from_ingress(
                token_node.value_text(),
                entry.resolved_target_id(),
                provenance,
            );
            Ok(Some(WorthUiResolvedArtifactInputNode::Token(
                WorthUiResolvedArtifactInputThemeTokenNode::new(
                    theme_token,
                    entry,
                    binding_target,
                    token_node.authored_identity().map(str::to_owned),
                    provenance.clone(),
                ),
            )))
        }
        WorthUiSemanticDeclaration::Projection(_)
        | WorthUiSemanticDeclaration::SemanticArtifact(_)
        | WorthUiSemanticDeclaration::AppearanceRole(_)
        | WorthUiSemanticDeclaration::Backdrop(_) => Ok(None),
    }
}
