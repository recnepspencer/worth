//! Capture and restore runtime declarations while leaving interning and memoization cold.
use super::SignalRuntimeConfig;
use crate::data::graph::SignalGraph;
use crate::data::output::{ComputationFamily, ComputationKey};
use crate::data::tier::TierPolicy;
use crate::durability::authored_image::{
    ComputationDefinitionView, KeyedNodeDefinitionView, RuntimeDefinitionView,
};
use crate::durability::{
    SignalAuthoredImageDenial as Denial, SignalDurableTierVocabulary, SignalRuntimeAuthoredImage,
};
use worth_execution::ExecutionRequest;
#[cfg(test)]
mod tests;

impl<T: Copy + Ord> SignalRuntimeConfig<T> {
    /// Capture semantic registrations, not runtime string IDs or cached results.
    pub fn capture_authored_image(
        &self,
        graph: &SignalGraph,
        vocabulary: &SignalDurableTierVocabulary<T>,
        branch: worth_foundational::facade::DurableSignalBranchIdentity,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalRuntimeAuthoredImage, Denial> {
        self.validate_authored_bindings(graph)?;
        self.capture_authored_nodes(graph.live_node_ids(), vocabulary, branch, execution)
    }
    fn capture_authored_nodes(
        &self,
        nodes: impl IntoIterator<Item = crate::data::handle::NodeId>,
        vocabulary: &SignalDurableTierVocabulary<T>,
        branch: worth_foundational::facade::DurableSignalBranchIdentity,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalRuntimeAuthoredImage, Denial> {
        let view = self.authored_definitions(nodes, vocabulary)?;
        SignalRuntimeAuthoredImage::capture(execution, branch, &view)
    }
    fn authored_definitions(
        &self,
        nodes: impl IntoIterator<Item = crate::data::handle::NodeId>,
        vocabulary: &SignalDurableTierVocabulary<T>,
    ) -> Result<RuntimeDefinitionView<'_, T>, Denial> {
        let mut policies = Vec::new();
        for policy in self.tier_policies.iter() {
            policies.push((
                vocabulary
                    .name(policy.tier)
                    .map_err(Denial::TierVocabulary)?,
                policy,
            ));
        }
        policies.sort_by_key(|(name, _)| *name);
        let mut computations = Vec::new();
        for (id, registration) in self.computations.iter() {
            computations.push(ComputationDefinitionView {
                family: self.key_registry.family(*id).as_str(),
                contract: &registration.contract,
                tier: vocabulary
                    .name(registration.tier)
                    .map_err(Denial::TierVocabulary)?,
                comparator: &registration.comparator,
            });
        }
        computations.sort_by_key(|row| row.family);
        let mut node_tiers = Vec::new();
        for node in nodes {
            if let Some(tier) = self.node_meta.tier_for_node(node) {
                node_tiers.push((node, vocabulary.name(tier).map_err(Denial::TierVocabulary)?));
            }
        }
        let mut keyed_nodes = Vec::new();
        for ((family, key), node) in self.keyed_nodes.iter() {
            keyed_nodes.push(KeyedNodeDefinitionView {
                family: self.key_registry.family(*family).as_str(),
                key: self.key_registry.keys[key.index()].as_str(),
                node: *node,
            });
        }
        keyed_nodes.sort_by_key(|row| (row.family, row.key));
        Ok(RuntimeDefinitionView {
            fallback: &self.fallback_comparator,
            resource_deadline: self.resource_runtime_deadline,
            policies,
            computations,
            node_tiers,
            keyed_nodes,
        })
    }
    /// Reconstruct configuration declarations with empty memoization and fresh
    /// local string identities. This does not admit a live owner branch.
    pub fn readmit_authored_image(
        execution: &ExecutionRequest<'_, '_>,
        image: &SignalRuntimeAuthoredImage,
        graph: &SignalGraph,
        vocabulary: &SignalDurableTierVocabulary<T>,
    ) -> Result<Self, Denial> {
        let config = Self::readmit_authored_nodes(
            execution,
            image,
            vocabulary,
            graph.runtime_instance_id(),
            |node| graph.is_alive(node),
        )?;
        config.validate_authored_bindings(graph)?;
        Ok(config)
    }
    fn validate_authored_bindings(&self, graph: &SignalGraph) -> Result<(), Denial> {
        graph.validate_authored_comparator(&self.fallback_comparator)?;
        for policy in self.tier_policies.iter() {
            graph.validate_authored_comparator(&policy.default_comparator)?;
        }
        for registration in self.computations.values() {
            graph.validate_authored_comparator(&registration.comparator)?;
        }
        Ok(())
    }
    fn readmit_authored_nodes(
        execution: &ExecutionRequest<'_, '_>,
        image: &SignalRuntimeAuthoredImage,
        vocabulary: &SignalDurableTierVocabulary<T>,
        graph_instance: u64,
        node_is_alive: impl Fn(crate::data::handle::NodeId) -> bool,
    ) -> Result<Self, Denial> {
        crate::durability::authored_image::admit_work(execution, image.as_bytes().len())?;
        let rows = image.rows(graph_instance)?;
        Self::readmit_authored_rows(rows, vocabulary, node_is_alive)
    }
    fn readmit_authored_rows(
        rows: crate::durability::authored_image::RuntimeDefinitionRows,
        vocabulary: &SignalDurableTierVocabulary<T>,
        node_is_alive: impl Fn(crate::data::handle::NodeId) -> bool,
    ) -> Result<Self, Denial> {
        let mut config = Self::new();
        config.fallback_comparator = rows.fallback;
        config.resource_runtime_deadline = rows.resource_deadline;
        for policy in rows.policies {
            let tier = vocabulary
                .resolve(&policy.tier)
                .map_err(Denial::TierVocabulary)?;
            config.set_tier_policy(
                TierPolicy::new(tier, policy.dependency, policy.propagation, policy.trigger)
                    .with_default_comparator(policy.comparator),
            );
        }
        for registration in rows.computations {
            let tier = vocabulary
                .resolve(&registration.tier)
                .map_err(Denial::TierVocabulary)?;
            config
                .define_computation(
                    ComputationFamily::new(registration.family),
                    registration.contract,
                    tier,
                    registration.comparator,
                )
                .map_err(|cause| Denial::Topology(Box::new(cause)))?;
        }
        for (node, name) in rows.node_tiers {
            if !node_is_alive(node) {
                return Err(Denial::Malformed);
            }
            let tier = vocabulary.resolve(&name).map_err(Denial::TierVocabulary)?;
            config.node_meta.ensure_capacity(node.index() as usize + 1);
            config.node_meta.set_tier(node, tier);
        }
        for binding in rows.keyed_nodes {
            let family = config
                .key_registry
                .intern_family(&ComputationFamily::new(binding.family));
            let key = config
                .key_registry
                .intern_key(&ComputationKey::new(binding.key));
            // A retired keyed binding stays retired. It cannot fabricate a lost
            // definition or stand in for that node's authored arena slot.
            config.keyed_nodes.insert((family, key), binding.node);
        }
        Ok(config)
    }
}
