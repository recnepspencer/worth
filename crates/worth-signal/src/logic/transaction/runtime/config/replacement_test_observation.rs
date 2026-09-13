use crate::data::comparator::VersionComparatorPolicy;
use crate::data::handle::NodeId;
use crate::data::node::NodeContract;
use crate::data::output::{
    ComputationFamily, ComputationKey, NodeEvaluationResult, StructuralMemoKey,
};
use crate::data::temporal::TemporalDuration;
use crate::data::tier::TierPolicy;

use super::SignalRuntimeConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SignalRuntimeConfigReplacementObservation<T: Copy + Ord> {
    pub(crate) node_tiers: Vec<(NodeId, Option<T>)>,
    pub(crate) tier_policies: Vec<TierPolicy<T>>,
    pub(crate) fallback_comparator: VersionComparatorPolicy,
    pub(crate) families: Vec<ComputationFamily>,
    pub(crate) keys: Vec<ComputationKey>,
    pub(crate) memo_keys: Vec<StructuralMemoKey>,
    pub(crate) computations: Vec<(ComputationFamily, NodeContract, T, VersionComparatorPolicy)>,
    pub(crate) keyed_nodes: Vec<(ComputationFamily, ComputationKey, NodeId)>,
    pub(crate) memo_cache: Vec<(
        ComputationFamily,
        ComputationKey,
        StructuralMemoKey,
        NodeEvaluationResult,
    )>,
    pub(crate) resource_runtime_deadline: Option<TemporalDuration>,
}

impl<T: Copy + Ord> SignalRuntimeConfig<T> {
    pub(crate) fn replacement_observation(
        &self,
        nodes: &[NodeId],
    ) -> SignalRuntimeConfigReplacementObservation<T> {
        SignalRuntimeConfigReplacementObservation {
            node_tiers: nodes
                .iter()
                .map(|node| (*node, self.node_meta.tier_for_node(*node)))
                .collect(),
            tier_policies: self.tier_policies.iter().cloned().collect(),
            fallback_comparator: self.fallback_comparator.clone(),
            families: self.key_registry.families.iter().cloned().collect(),
            keys: self.key_registry.keys.iter().cloned().collect(),
            memo_keys: self.key_registry.memo_keys.iter().cloned().collect(),
            computations: self
                .computations
                .iter()
                .map(|(family_id, registration)| {
                    (
                        self.key_registry.family(*family_id).clone(),
                        registration.contract.clone(),
                        registration.tier,
                        registration.comparator.clone(),
                    )
                })
                .collect(),
            keyed_nodes: self
                .keyed_nodes
                .iter()
                .map(|((family_id, key_id), node)| {
                    (
                        self.key_registry.family(*family_id).clone(),
                        self.key_registry.keys[key_id.index()].clone(),
                        *node,
                    )
                })
                .collect(),
            memo_cache: self
                .memo_cache
                .iter()
                .map(|((family_id, key_id, memo_key_id), result)| {
                    (
                        self.key_registry.family(*family_id).clone(),
                        self.key_registry.keys[key_id.index()].clone(),
                        self.key_registry.memo_key(*memo_key_id).clone(),
                        result.clone(),
                    )
                })
                .collect(),
            resource_runtime_deadline: self.resource_runtime_deadline,
        }
    }
}
