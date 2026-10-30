use std::collections::{BTreeMap, BTreeSet};
use worth_ui_dsl::WorthUiSourceModuleId;

#[cfg(test)]
use crate::source::{
    WorthUiArtifactEquivalenceBasis, WorthUiArtifactEquivalenceComparator, WorthUiArtifactNode,
};
use crate::source::{WorthUiArtifactHandle, WorthUiArtifactModule};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorthUiArtifact {
    modules: BTreeMap<WorthUiSourceModuleId, WorthUiArtifactModule>,
    canonical_module_order: Vec<WorthUiSourceModuleId>,
    node_identity_index: BTreeMap<String, (WorthUiSourceModuleId, usize)>,
    authored_provenance_index: BTreeMap<u64, Box<[Box<str>]>>,
    query_binding_ids: BTreeSet<String>,
}

impl WorthUiArtifact {
    pub(crate) fn new(
        modules: BTreeMap<WorthUiSourceModuleId, WorthUiArtifactModule>,
        canonical_module_order: Vec<WorthUiSourceModuleId>,
    ) -> Self {
        let mut node_identity_index = BTreeMap::new();
        let mut authored_provenance_index = BTreeMap::<u64, Vec<Box<str>>>::new();
        let mut query_binding_ids = BTreeSet::new();
        for module_id in &canonical_module_order {
            let Some(module) = modules.get(module_id) else {
                continue;
            };
            for (node_index, node) in module.nodes().iter().enumerate() {
                node_identity_index.insert(
                    node.identity_seed().basis().to_owned(),
                    (module_id.clone(), node_index),
                );
                authored_provenance_index
                    .entry(node.authored_provenance_digest())
                    .or_default()
                    .push(node.authored_declaration_identity().into());
                if let crate::source::WorthUiArtifactNode::Binding(binding) = node {
                    let binding_id = binding
                        .view_binding_reference()
                        .view_binding()
                        .id()
                        .as_str()
                        .to_owned();
                    node_identity_index.insert(binding_id.clone(), (module_id.clone(), node_index));
                    query_binding_ids.insert(binding_id);
                }
                if let crate::source::WorthUiArtifactNode::Surface(surface) = node
                    && let Some(binding) = surface.semantics().view_binding()
                {
                    query_binding_ids.insert(binding.view_binding().id().as_str().to_owned());
                }
            }
        }
        Self {
            modules,
            canonical_module_order,
            node_identity_index,
            authored_provenance_index: authored_provenance_index
                .into_iter()
                .map(|(digest, identities)| (digest, identities.into_boxed_slice()))
                .collect(),
            query_binding_ids,
        }
    }

    pub(crate) fn module(
        &self,
        module_id: &WorthUiSourceModuleId,
    ) -> Option<&WorthUiArtifactModule> {
        self.modules.get(module_id)
    }

    pub(crate) fn module_ids(&self) -> &[WorthUiSourceModuleId] {
        &self.canonical_module_order
    }

    pub(crate) fn identity_handles(&self) -> impl Iterator<Item = (&str, &WorthUiArtifactHandle)> {
        self.canonical_module_order
            .iter()
            .filter_map(|module_id| self.module(module_id))
            .flat_map(|module| {
                module
                    .nodes()
                    .iter()
                    .map(|node| (node.identity_seed().basis(), node.handle()))
            })
    }

    #[cfg(test)]
    pub(crate) fn node(&self, handle: &WorthUiArtifactHandle) -> Option<&WorthUiArtifactNode> {
        self.module(handle.module_id())
            .and_then(|module| module.node(handle.node_index()))
    }

    pub(crate) fn node_for_identity_basis(
        &self,
        identity_basis: &str,
    ) -> Option<&crate::source::WorthUiArtifactNode> {
        let (module_id, node_index) = self.node_identity_index.get(identity_basis)?;
        self.module(module_id)?.nodes().get(*node_index)
    }

    pub(crate) fn identity_bases_for_authored_provenance(
        &self,
        provenance_digest: u64,
    ) -> &[Box<str>] {
        self.authored_provenance_index
            .get(&provenance_digest)
            .map(Box::as_ref)
            .unwrap_or_default()
    }

    pub(crate) fn authored_provenance_entries(
        &self,
    ) -> impl Iterator<Item = (u64, String, Option<&str>, Option<&str>)> {
        self.canonical_module_order
            .iter()
            .filter_map(|module_id| self.module(module_id))
            .flat_map(|module| {
                module.nodes().iter().map(|node| {
                    (
                        node.authored_provenance_digest(),
                        node.authored_declaration_identity(),
                        node.component_capability_identity(),
                        node.theme_token_capability_identity(),
                    )
                })
            })
    }

    pub(crate) fn query_binding_ids(&self) -> &BTreeSet<String> {
        &self.query_binding_ids
    }

    pub(crate) fn authored_provenance_digests(&self) -> Vec<u64> {
        let mut digests = Vec::new();
        for module_id in &self.canonical_module_order {
            let Some(module) = self.module(module_id) else {
                continue;
            };
            for node in module.nodes() {
                digests.push(node.authored_provenance_digest());
            }
        }
        digests.sort_unstable();
        digests.dedup();
        digests
    }

    #[cfg(test)]
    pub(crate) fn equivalent_shape(&self, other: &Self) -> bool {
        WorthUiArtifactEquivalenceComparator::compare(
            self,
            other,
            WorthUiArtifactEquivalenceBasis::semantic(),
        )
        .is_equivalent()
    }
}
