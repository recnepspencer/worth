use crate::data::comparator::VersionComparatorPolicy;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::{NodeContract, NodeEvaluationConfig};
use crate::data::node_meta::NodeMetaStore;
use crate::data::output::{
    ComputationFamily, ComputationKey, NodeEvaluationResult, StructuralMemoKey,
};
use crate::data::temporal::TemporalDuration;
use crate::data::tier::TierPolicy;
use crate::data::tier_policy_table::TierPolicyTable;

use super::super::key_registry::{RuntimeKeyRegistry, RuntimeStringId};

#[cfg(test)]
#[path = "config/replacement_test_observation.rs"]
mod replacement_test_observation;
#[cfg(test)]
pub(crate) use replacement_test_observation::SignalRuntimeConfigReplacementObservation;

#[derive(Debug, Clone)]
pub struct SignalRuntimeConfig<T: Copy + Ord> {
    node_meta: NodeMetaStore<T>,
    tier_policies: TierPolicyTable<T>,
    fallback_comparator: VersionComparatorPolicy,
    pub(super) key_registry: RuntimeKeyRegistry,
    computations: crate::data::persistent_ord_map::PersistentOrdMap<
        RuntimeStringId,
        ComputationRegistration<T>,
    >,
    keyed_nodes: crate::data::persistent_ord_map::PersistentOrdMap<
        (RuntimeStringId, RuntimeStringId),
        NodeId,
    >,
    memo_cache: crate::data::persistent_ord_map::PersistentOrdMap<
        (RuntimeStringId, RuntimeStringId, RuntimeStringId),
        NodeEvaluationResult,
    >,
    resource_runtime_deadline: Option<TemporalDuration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ComputationRegistration<T: Copy + Ord> {
    contract: NodeContract,
    tier: T,
    comparator: VersionComparatorPolicy,
}

impl<T: Copy + Ord> Default for SignalRuntimeConfig<T> {
    fn default() -> Self {
        Self {
            node_meta: NodeMetaStore::default(),
            tier_policies: TierPolicyTable::default(),
            fallback_comparator: VersionComparatorPolicy::Exact,
            key_registry: RuntimeKeyRegistry::default(),
            computations: Default::default(),
            keyed_nodes: Default::default(),
            memo_cache: Default::default(),
            resource_runtime_deadline: None,
        }
    }
}

impl<T: Copy + Ord> SignalRuntimeConfig<T> {
    pub(crate) fn fork_persistent(&mut self) -> Self {
        Self {
            node_meta: self.node_meta.fork_persistent(),
            tier_policies: self.tier_policies.fork_persistent(),
            fallback_comparator: self.fallback_comparator.clone(),
            key_registry: self.key_registry.fork_persistent(),
            computations: self.computations.fork_persistent(),
            keyed_nodes: self.keyed_nodes.fork_persistent(),
            memo_cache: self.memo_cache.fork_persistent(),
            resource_runtime_deadline: self.resource_runtime_deadline,
        }
    }

    #[cfg(test)]
    pub(crate) fn fork_storage_identity(&self) -> Self {
        Self {
            node_meta: self.node_meta.clone(),
            tier_policies: self.tier_policies.clone(),
            fallback_comparator: self.fallback_comparator.clone(),
            key_registry: self.key_registry.fork_storage_identity(),
            computations: self.computations.fork_storage_identity(),
            keyed_nodes: self.keyed_nodes.fork_storage_identity(),
            memo_cache: self.memo_cache.fork_storage_identity(),
            resource_runtime_deadline: self.resource_runtime_deadline,
        }
    }

    #[cfg(test)]
    pub(crate) fn shares_fork_storage_with(&self, other: &Self) -> bool {
        self.node_meta.shares_storage_with(&other.node_meta)
            && self.tier_policies.shares_storage_with(&other.tier_policies)
            && self.key_registry.shares_storage_with(&other.key_registry)
            && self.computations.ptr_eq(&other.computations)
            && self.keyed_nodes.ptr_eq(&other.keyed_nodes)
            && self.memo_cache.ptr_eq(&other.memo_cache)
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub(super) fn sync_graph_capacity(&mut self, graph: &SignalGraph) {
        self.node_meta.ensure_capacity(graph.arena_capacity());
    }

    pub(super) fn prune_stale_node_meta(&mut self, graph: &SignalGraph) {
        self.node_meta.prune_slots(|index, generation| {
            graph
                .live_node_id_at(index)
                .is_some_and(|node| node.generation() == generation)
        });
    }

    pub fn set_node_tier(&mut self, graph: &SignalGraph, node: NodeId, tier: T) {
        self.sync_graph_capacity(graph);
        self.node_meta.set_tier(node, tier);
    }

    pub fn set_tier_policy(&mut self, policy: TierPolicy<T>) {
        self.tier_policies.set(policy);
    }

    pub fn set_fallback_comparator(&mut self, policy: VersionComparatorPolicy) {
        self.fallback_comparator = policy;
    }

    pub fn node_meta(&self) -> &NodeMetaStore<T> {
        &self.node_meta
    }

    pub fn tier_policies(&self) -> &TierPolicyTable<T> {
        &self.tier_policies
    }

    pub fn fallback_comparator(&self) -> &VersionComparatorPolicy {
        &self.fallback_comparator
    }

    pub fn set_resource_runtime_deadline(&mut self, deadline: TemporalDuration) {
        self.resource_runtime_deadline = Some(deadline);
    }

    pub fn clear_resource_runtime_deadline(&mut self) {
        self.resource_runtime_deadline = None;
    }

    pub fn resource_runtime_deadline(&self) -> Option<TemporalDuration> {
        self.resource_runtime_deadline
    }

    pub fn define_computation(
        &mut self,
        family: impl Into<ComputationFamily>,
        contract: NodeContract,
        tier: T,
        comparator: VersionComparatorPolicy,
    ) -> Result<ComputationFamily, crate::data::error::SignalError> {
        let family = family.into();
        let family_id = self.key_registry.intern_family(&family);
        let registration = ComputationRegistration {
            contract,
            tier,
            comparator,
        };
        if let Some(existing) = self.computations.get(&family_id) {
            if existing != &registration {
                return Err(crate::data::error::SignalError::invalid_input(format!(
                    "computation family '{family:?}' already defined with a different spec",
                )));
            }
            return Ok(family);
        }
        self.computations.insert(family_id, registration);
        Ok(family)
    }

    pub(super) fn resolve_defined_node(
        &mut self,
        graph: &mut SignalGraph,
        family: &ComputationFamily,
        key: impl Into<ComputationKey>,
    ) -> NodeId {
        let key = key.into();
        let registry_key = (
            self.key_registry.intern_family(family),
            self.key_registry.intern_key(&key),
        );
        if let Some(node) = self.keyed_nodes.get(&registry_key).copied() {
            return node;
        }
        let node = self.create_keyed_node(graph, registry_key.0);
        self.sync_graph_capacity(graph);
        self.keyed_nodes.insert(registry_key, node);
        node
    }

    pub(super) fn resolve_defined_node_with_created(
        &mut self,
        graph: &mut SignalGraph,
        family: &ComputationFamily,
        key: impl Into<ComputationKey>,
    ) -> (NodeId, bool) {
        let key = key.into();
        let registry_key = (
            self.key_registry.intern_family(family),
            self.key_registry.intern_key(&key),
        );
        if let Some(node) = self.keyed_nodes.get(&registry_key).copied() {
            return (node, false);
        }
        let node = self.create_keyed_node(graph, registry_key.0);
        self.sync_graph_capacity(graph);
        self.keyed_nodes.insert(registry_key, node);
        (node, true)
    }

    fn create_keyed_node(&mut self, graph: &mut SignalGraph, family_id: RuntimeStringId) -> NodeId {
        let node = match self.computations.get(&family_id) {
            Some(registration) => {
                let mut config = NodeEvaluationConfig::default();
                config.contract = registration.contract.clone();
                config.comparator = Some(registration.comparator.clone());
                graph.create_node_with_config(config)
            }
            None => graph.node().build(),
        };
        self.sync_graph_capacity(graph);
        if let Some(registration) = self.computations.get(&family_id) {
            self.node_meta.set_tier(node, registration.tier);
        }
        node
    }

    pub(super) fn lookup_memoized_result(
        &self,
        family: &ComputationFamily,
        key: &ComputationKey,
        memo_key: &StructuralMemoKey,
    ) -> Option<NodeEvaluationResult> {
        let family_id = self.key_registry.family_lookup.get(family).copied()?;
        let key_id = self.key_registry.key_lookup.get(key).copied()?;
        let memo_key_id = self.key_registry.memo_key_lookup.get(memo_key).copied()?;
        self.memo_cache
            .get(&(family_id, key_id, memo_key_id))
            .cloned()
    }

    pub(super) fn store_memoized_result(
        &mut self,
        family: &ComputationFamily,
        key: &ComputationKey,
        memo_key: &StructuralMemoKey,
        result: NodeEvaluationResult,
    ) {
        let family_id = self.key_registry.intern_family(family);
        let key_id = self.key_registry.intern_key(key);
        let memo_key_id = self.key_registry.intern_memo_key(memo_key);
        self.memo_cache
            .insert((family_id, key_id, memo_key_id), result);
    }

    #[cfg(test)]
    pub(crate) fn test_registry_counts(&self) -> (usize, usize, usize, usize, usize) {
        (
            self.key_registry.families.len(),
            self.key_registry.keys.len(),
            self.key_registry.memo_keys.len(),
            self.keyed_nodes.len(),
            self.memo_cache.len(),
        )
    }
}
