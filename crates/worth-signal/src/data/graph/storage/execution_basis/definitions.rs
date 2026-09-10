use std::sync::Arc;

use crate::data::aspect::SignalAspectLoweringOwner;
use crate::data::bitset::DenseBitset;
use crate::data::graph::SignalGraph;
use crate::data::node::NodeDefinitionData;
use crate::data::persistent_ord_set::PersistentOrdSet;
use crate::data::persistent_paged_vector::PersistentPagedVector;
use crate::data::persistent_vector::PersistentVector;
use crate::runtime_policy::InstalledSignalRuntimePolicy;
use crate::schema::data::SignalSchemaRegistry;

use super::super::Slot;

mod retained_charge;

/// The definition view paired with retained evaluation storage. Persistent
/// capture preserves old node generations without copying the whole graph.
/// This object is storage, not authority to execute the selected definition.
#[derive(Debug, Clone)]
pub(in crate::data::graph) struct SignalExecutionDefinitions {
    storage_lineage: u64,
    definitions: PersistentPagedVector<NodeDefinitionData>,
    nodes: PersistentPagedVector<Slot>,
    free_list: PersistentVector<u32>,
    free_slots: DenseBitset,
    active_nodes: u32,
    schema: Arc<SignalSchemaRegistry>,
    lowering_owner: Option<SignalAspectLoweringOwner>,
    authorization_policies: PersistentOrdSet<[u8; 32]>,
    installed_policy: InstalledSignalRuntimePolicy,
}

impl SignalExecutionDefinitions {
    #[cfg(test)]
    pub(super) fn contains_node(&self, node: crate::data::handle::NodeId) -> bool {
        self.nodes
            .get(node.index() as usize)
            .is_some_and(|slot| slot.is_occupied() && slot.generation == node.generation())
    }

    pub(in crate::data::graph) fn retain(
        graph: &mut SignalGraph,
        resources: &mut crate::data::retained_storage::SignalConditionalRetentionReservation,
    ) -> Self {
        Self {
            storage_lineage: graph.instance_id,
            definitions: graph.arena.definitions.fork_reserved(resources),
            nodes: graph.arena.nodes.fork_reserved(resources),
            free_list: graph.arena.free_list.fork_reserved(resources),
            free_slots: graph.arena.free_slots.fork_reserved(resources),
            active_nodes: graph.arena.active_nodes,
            schema: Arc::clone(&graph.schema_registry),
            lowering_owner: graph.aspect_lowering_owner.clone(),
            authorization_policies: graph
                .authorization_policy_identities
                .fork_reserved(resources),
            installed_policy: graph.observation.installed_policy,
        }
    }

    /// Persistent forks share this lineage. Only owner admission can establish
    /// branch/cell affinity; this check rejects unrelated storage families.
    pub(in crate::data::graph) fn matches_storage_lineage(&self, graph: &SignalGraph) -> bool {
        self.storage_lineage == graph.instance_id
    }

    pub(in crate::data::graph) fn installed_policy(&self) -> InstalledSignalRuntimePolicy {
        self.installed_policy
    }

    pub(in crate::data::graph) fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub(in crate::data::graph) fn exchange(&mut self, graph: &mut SignalGraph) {
        std::mem::swap(&mut self.definitions, &mut graph.arena.definitions);
        std::mem::swap(&mut self.nodes, &mut graph.arena.nodes);
        std::mem::swap(&mut self.free_list, &mut graph.arena.free_list);
        std::mem::swap(&mut self.free_slots, &mut graph.arena.free_slots);
        std::mem::swap(&mut self.active_nodes, &mut graph.arena.active_nodes);
        std::mem::swap(&mut self.schema, &mut graph.schema_registry);
        std::mem::swap(&mut self.lowering_owner, &mut graph.aspect_lowering_owner);
        std::mem::swap(
            &mut self.authorization_policies,
            &mut graph.authorization_policy_identities,
        );
        std::mem::swap(
            &mut self.installed_policy,
            &mut graph.observation.installed_policy,
        );
    }
}
