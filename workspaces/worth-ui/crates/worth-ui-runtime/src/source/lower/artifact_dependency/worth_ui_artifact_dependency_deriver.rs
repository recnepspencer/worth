use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use worth_ui_dsl::WorthUiSourceModuleId;

use crate::capability::SurfaceId;
use crate::source::{
    WorthUiArtifact, WorthUiArtifactDependencyEdge, WorthUiArtifactDependencyEdgeKind,
    WorthUiArtifactDependencyGraph, WorthUiArtifactDependencyMetrics,
    WorthUiArtifactDependencyReport, WorthUiArtifactDependencyTarget, WorthUiArtifactHandle,
    WorthUiArtifactImpactMetadata, WorthUiArtifactNode, WorthUiArtifactSubtreeDigest,
    WorthUiIncrementalInvalidationBasis, WorthUiMosaicRegionFacts, WorthUiMosaicStructureFacts,
    WorthUiRuntimeDependencyHook,
};

use super::worth_ui_artifact_dependency_index_builder::{
    build_dependency_index, WorthUiArtifactDependencyIndex,
};
use super::worth_ui_runtime_dependency_hook_deriver::hooks_for_view_binding;
use super::worth_ui_subtree_digest_basis::subtree_digest;

#[derive(Clone, Debug, Default)]
pub(crate) struct WorthUiArtifactDependencyDeriver;

impl WorthUiArtifactDependencyDeriver {
    #[cfg(test)]
    pub(crate) fn derive(artifact: &WorthUiArtifact) -> WorthUiIncrementalInvalidationBasis {
        Self::derive_with_report(artifact).basis().clone()
    }

    pub(crate) fn derive_with_report(
        artifact: &WorthUiArtifact,
    ) -> WorthUiArtifactDependencyReport {
        let index = build_dependency_index(artifact);
        let mut context = DependencyDerivationContext::new(index);
        context.derive_from_artifact(artifact);
        let basis = context.finish();
        WorthUiArtifactDependencyReport::new(basis, context.metrics)
    }
}

struct DependencyDerivationContext {
    index: WorthUiArtifactDependencyIndex,
    edges: Vec<WorthUiArtifactDependencyEdge>,
    module_dependencies: BTreeMap<WorthUiSourceModuleId, Vec<WorthUiSourceModuleId>>,
    module_impacts: BTreeMap<WorthUiSourceModuleId, BTreeSet<WorthUiArtifactHandle>>,
    subtree_impacts: BTreeMap<WorthUiArtifactHandle, BTreeSet<WorthUiArtifactHandle>>,
    subtree_digests: BTreeMap<WorthUiArtifactHandle, WorthUiArtifactSubtreeDigest>,
    runtime_hooks: BTreeMap<WorthUiArtifactHandle, Vec<WorthUiRuntimeDependencyHook>>,
    metrics: WorthUiArtifactDependencyMetrics,
}

impl DependencyDerivationContext {
    fn new(index: WorthUiArtifactDependencyIndex) -> Self {
        Self {
            index,
            edges: Vec::new(),
            module_dependencies: BTreeMap::new(),
            module_impacts: BTreeMap::new(),
            subtree_impacts: BTreeMap::new(),
            subtree_digests: BTreeMap::new(),
            runtime_hooks: BTreeMap::new(),
            metrics: WorthUiArtifactDependencyMetrics::default(),
        }
    }

    fn derive_from_artifact(&mut self, artifact: &WorthUiArtifact) {
        for module_id in artifact.module_ids() {
            let module = artifact.module(module_id).expect("artifact module");
            for node in module.nodes() {
                self.metrics.record_node_indexed();
                self.record_module_membership(module_id, node.handle());
                self.record_node_dependencies(node);
            }
        }
    }

    fn finish(&self) -> WorthUiIncrementalInvalidationBasis {
        let graph = WorthUiArtifactDependencyGraph::new(
            self.edges.clone(),
            self.module_dependencies.clone(),
            self.subtree_digests.clone(),
            self.runtime_hooks.clone(),
        );
        let impact_metadata = WorthUiArtifactImpactMetadata::new(
            set_map_to_vec_map(&self.module_impacts),
            set_map_to_vec_map(&self.subtree_impacts),
            self.index.handles().len(),
        );
        WorthUiIncrementalInvalidationBasis::new(graph, impact_metadata)
    }

    fn record_module_membership(
        &mut self,
        module_id: &WorthUiSourceModuleId,
        handle: &WorthUiArtifactHandle,
    ) {
        self.module_impacts
            .entry(module_id.clone())
            .or_default()
            .insert(handle.clone());
        self.subtree_impacts
            .entry(handle.clone())
            .or_default()
            .insert(handle.clone());
    }

    fn record_node_dependencies(&mut self, node: &WorthUiArtifactNode) {
        let hooks = runtime_hooks_for_node(node);
        self.record_runtime_hooks(node.handle(), hooks);
        self.record_subtree_digest(node);
        match node {
            WorthUiArtifactNode::Import(import) => self.record_module_import(import),
            WorthUiArtifactNode::Component(component) => {
                self.record_structure_edges(component.handle(), component.structure())
            }
            WorthUiArtifactNode::Surface(surface) => {
                self.record_structure_edges(surface.handle(), surface.structure())
            }
            WorthUiArtifactNode::Binding(binding) => {
                self.record_structure_edges(binding.handle(), binding.structure())
            }
            WorthUiArtifactNode::Token(_) => {}
        }
    }

    fn record_module_import(&mut self, import: &crate::source::WorthUiArtifactImportNode) {
        let Some(target_module) = module_id_from_import(import.target().authored_text()) else {
            return;
        };
        self.module_dependencies
            .entry(import.handle().module_id().clone())
            .or_default()
            .push(target_module.clone());
        self.module_impacts
            .entry(target_module.clone())
            .or_default()
            .insert(import.handle().clone());
        self.push_edge(
            import.handle().clone(),
            WorthUiArtifactDependencyTarget::Module(target_module),
            WorthUiArtifactDependencyEdgeKind::ModuleImport,
        );
    }

    fn record_structure_edges(
        &mut self,
        source: &WorthUiArtifactHandle,
        structure: &WorthUiMosaicStructureFacts,
    ) {
        for surface_id in mounted_surface_ids(structure) {
            let Some(target) = self.index.surface_handle(&surface_id).cloned() else {
                continue;
            };
            self.subtree_impacts
                .entry(target.clone())
                .or_default()
                .insert(source.clone());
            self.push_edge(
                source.clone(),
                WorthUiArtifactDependencyTarget::Artifact(target),
                WorthUiArtifactDependencyEdgeKind::MosaicMount,
            );
        }
    }

    fn record_runtime_hooks(
        &mut self,
        handle: &WorthUiArtifactHandle,
        hooks: Vec<WorthUiRuntimeDependencyHook>,
    ) {
        for hook in &hooks {
            self.push_edge(
                handle.clone(),
                WorthUiArtifactDependencyTarget::RuntimeHook(hook.clone()),
                WorthUiArtifactDependencyEdgeKind::RuntimeHook,
            );
            self.metrics.record_runtime_hook();
        }
        if !hooks.is_empty() {
            self.runtime_hooks.insert(handle.clone(), hooks);
        }
    }

    fn record_subtree_digest(&mut self, node: &WorthUiArtifactNode) {
        let hooks = self
            .runtime_hooks
            .get(node.handle())
            .map_or_else(|| &[] as &[WorthUiRuntimeDependencyHook], Vec::as_slice);
        self.subtree_digests
            .insert(node.handle().clone(), subtree_digest(node, hooks));
        self.metrics.record_subtree_digest();
    }

    fn push_edge(
        &mut self,
        source: WorthUiArtifactHandle,
        target: WorthUiArtifactDependencyTarget,
        kind: WorthUiArtifactDependencyEdgeKind,
    ) {
        self.edges
            .push(WorthUiArtifactDependencyEdge::new(source, target, kind));
        self.metrics.record_dependency_edge();
    }
}

fn runtime_hooks_for_node(node: &WorthUiArtifactNode) -> Vec<WorthUiRuntimeDependencyHook> {
    match node {
        WorthUiArtifactNode::Surface(node) => node
            .semantics()
            .view_binding()
            .map(hooks_for_view_binding)
            .unwrap_or_default(),
        WorthUiArtifactNode::Binding(node) => hooks_for_view_binding(node.view_binding_reference()),
        WorthUiArtifactNode::Import(_)
        | WorthUiArtifactNode::Component(_)
        | WorthUiArtifactNode::Token(_) => Vec::new(),
    }
}

fn mounted_surface_ids(structure: &WorthUiMosaicStructureFacts) -> Vec<SurfaceId> {
    let mut surface_ids = Vec::new();
    for region in structure.root_regions() {
        collect_region_mounts(region, &mut surface_ids);
    }
    surface_ids
}

fn collect_region_mounts(region: &WorthUiMosaicRegionFacts, surface_ids: &mut Vec<SurfaceId>) {
    for mount in region.mounts() {
        surface_ids.push(mount.surface().id().clone());
    }
    for child_region in region.child_regions() {
        collect_region_mounts(child_region, surface_ids);
    }
}

fn module_id_from_import(authored_text: &str) -> Option<WorthUiSourceModuleId> {
    WorthUiSourceModuleId::from_relative_path(Path::new(authored_text)).ok()
}

fn set_map_to_vec_map<K: Ord + Clone, V: Ord + Clone>(
    map: &BTreeMap<K, BTreeSet<V>>,
) -> BTreeMap<K, Vec<V>> {
    map.iter()
        .map(|(key, values)| (key.clone(), values.iter().cloned().collect()))
        .collect()
}
