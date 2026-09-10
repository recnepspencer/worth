//! Arena-based signal graph with dependency storage.

use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use crate::data::aspect::SignalAspectLoweringOwner;
use crate::data::bitset::DenseBitset;
use crate::data::error::SignalError;
use crate::data::graph::storage::invalidation_causes::CanonicalCauseSetStore;
use crate::data::handle::NodeId;
use crate::data::node::{NodeColdData, NodeDefinitionData, NodeHotData, NodeWarmData};
use crate::logic::transaction::SignalObservationSessionState;
use crate::schema::data::SignalSchemaRegistry;

use super::super::compaction::CompactionState;
use super::super::storage::Slot;

mod branch_mutations;
mod capabilities;
#[cfg(test)]
mod cause_set_compaction_tests;
#[cfg(test)]
mod cause_sets_tests;
mod checkpoint;
mod construction;
mod counter_access;
#[cfg(test)]
mod deserialization_shape_tests;
#[cfg(test)]
mod direct_invalidation_basis_tests;
#[cfg(test)]
mod evaluation_partition_tests;
mod lineage_publication;
mod node_arena_wire;
mod observation_state;
mod performed_counter_state;
mod performed_work_state;
mod persistent_fork;
mod reconstitution;
#[cfg(test)]
mod reconstitution_tests;
mod reconstruction_counters;
#[cfg(test)]
#[path = "graph/replacement_test_observation.rs"]
mod replacement_test_observation;
mod retained_node_edit;
mod retained_node_mutation;
pub(crate) use retained_node_edit::{
    PreparedRetainedNodeEdit, RetainedNodeEditOutcome, RetainedNodeEditPreparation,
};
pub(in crate::data::graph) use retained_node_mutation::{
    map_accounting as map_node_edit_accounting, map_edit as map_node_edit,
    map_retention as map_node_edit_retention,
};
#[cfg(test)]
mod retained_node_edit_tests;
mod scheduling_state;
mod scratch_lease;
mod topology_state;
mod traversal_state;

pub(crate) use crate::logic::invalidation::causality::{
    PreparedDirectCauseNodes, PreparedDirectCausePublication, PreparedDirectCauseStores,
    PreparedRetainedDirectCauseStores,
};
pub(crate) use branch_mutations::{BranchMutationNodeImage, BranchMutationRecord};
pub use branch_mutations::{
    BranchStructuralDelta, DependencySnapshotStructuralDelta, DependencyTopologyDelta,
    RuntimeArtifactStructuralDelta,
};
pub(crate) use observation_state::{ObservationCaptureCleanup, RuntimeObservation};
pub(crate) use performed_counter_state::InvalidationPerformedCounterState;
pub(crate) use performed_work_state::{
    PerformedTargetSnapshot, PerformedWorkBuffer, PerformedWorkCaptureState,
    PreparedPerformedWorkCapture,
};
pub(crate) use persistent_fork::SignalGraphForkWork;
#[cfg(test)]
pub(crate) use persistent_fork::SignalGraphPersistentIdentity;
#[cfg(test)]
pub(crate) use persistent_fork::SignalGraphPersistentSharing;
pub use reconstitution::{SignalGraphReconstitution, SignalGraphReconstitutionReport};
pub(crate) use reconstruction_counters::ReconstructionCounters;
#[cfg(test)]
pub(crate) use replacement_test_observation::{
    SignalGraphCloneLocalObservation, SignalGraphRetainedObservation,
};
pub(crate) use topology_state::{EdgeTopology, RetainedTopologyIndexDenial};
pub(crate) use traversal_state::TraversalResources;

#[derive(Debug, Clone)]
pub(crate) struct NodeArena {
    pub(in crate::data::graph) definitions:
        crate::data::persistent_paged_vector::PersistentPagedVector<NodeDefinitionData>,
    pub(in crate::data::graph) nodes:
        crate::data::persistent_paged_vector::PersistentPagedVector<Slot>,
    pub(in crate::data::graph) hot:
        crate::data::persistent_paged_vector::PersistentPagedVector<Option<NodeHotData>>,
    pub(in crate::data::graph) warm:
        crate::data::persistent_paged_vector::PersistentPagedVector<NodeWarmData>,
    pub(in crate::data::graph) cold:
        crate::data::persistent_paged_vector::PersistentPagedVector<Option<Box<NodeColdData>>>,
    pub(in crate::data::graph) retained_node_ledger:
        Option<std::sync::Arc<crate::data::retained_storage::SignalConditionalRetentionLedger>>,
    pub(in crate::data::graph) retained_node_custody: Option<
        std::sync::Arc<crate::data::retained_storage::SignalConditionalRetentionReservation>,
    >,
    pub(in crate::data::graph) free_list: crate::data::persistent_vector::PersistentVector<u32>,
    pub(in crate::data::graph) free_slots: DenseBitset,
    pub(in crate::data::graph) active_nodes: u32,
    pub(in crate::data::graph) compaction: CompactionState,
    // Drop custody only after every retained payload owned by this root.
    pub(in crate::data::graph) retained_seed_custody: Option<
        std::sync::Arc<crate::data::retained_storage::SignalConditionalRetentionReservation>,
    >,
}

impl NodeArena {
    fn validate_deserialized_lane_alignment<E>(&self) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        let nodes = self.nodes.len();
        let definitions = self.definitions.len();
        let hot = self.hot.len();
        let warm = self.warm.len();
        let cold = self.cold.len();
        if nodes == definitions && nodes == hot && nodes == warm && nodes == cold {
            return Ok(());
        }
        Err(E::custom(format!(
            "signal graph arena lane lengths must match: nodes={nodes}, definitions={definitions}, hot={hot}, warm={warm}, cold={cold}"
        )))
    }

    fn fork_persistent(&mut self) -> Self {
        Self {
            definitions: self.definitions.fork_persistent(),
            nodes: self.nodes.fork_persistent(),
            hot: self.hot.fork_persistent(),
            warm: self.warm.fork_persistent(),
            cold: self.cold.fork_persistent(),
            retained_node_ledger: self.retained_node_ledger.clone(),
            retained_node_custody: self.retained_node_custody.clone(),
            retained_seed_custody: self.retained_seed_custody.clone(),
            free_list: self.free_list.fork_persistent(),
            free_slots: self.free_slots.fork_persistent(),
            active_nodes: self.active_nodes,
            compaction: self.compaction.clone(),
        }
    }

    fn operational_clone(&self) -> Self {
        Self {
            definitions: self.definitions.operational_clone(),
            nodes: self.nodes.operational_clone(),
            hot: self.hot.operational_clone(),
            warm: self.warm.operational_clone(),
            cold: self.cold.operational_clone(),
            // Materialized reconstruction owns new allocations; it cannot
            // retain the source roots' conditional storage reservation.
            retained_node_ledger: None,
            retained_node_custody: None,
            retained_seed_custody: None,
            free_list: self.free_list.operational_clone(),
            free_slots: self.free_slots.operational_clone(),
            active_nodes: self.active_nodes,
            compaction: self.compaction.clone(),
        }
    }

    #[cfg(test)]
    fn shares_storage_with(&self, other: &Self) -> bool {
        self.definitions.shares_storage_with(&other.definitions)
            && self.nodes.shares_storage_with(&other.nodes)
            && self.hot.shares_storage_with(&other.hot)
            && self.warm.shares_storage_with(&other.warm)
            && self.cold.shares_storage_with(&other.cold)
            && self.free_list.shares_storage_with(&other.free_list)
            && self.free_slots.shares_storage_with(&other.free_slots)
    }
}

/// The reactive signal graph.
///
/// An arena of `NodeEntry` values with graph-owned dependency, subscriber,
/// and snapshot storage.
#[derive(Debug, Serialize)]
pub struct SignalGraph {
    #[serde(skip, default)]
    pub(in crate::data::graph) lifecycle_token: std::sync::Arc<()>,
    #[serde(skip, default = "next_signal_graph_instance_id")]
    pub(in crate::data::graph) instance_id: u64,
    pub(in crate::data::graph) arena: NodeArena,
    pub(in crate::data::graph) topology: EdgeTopology,
    #[serde(
        default,
        serialize_with = "crate::data::graph::storage::invalidation_causes::serialize_canonical_cause_sets"
    )]
    pub(crate) cause_sets: CanonicalCauseSetStore,
    #[serde(skip, default)]
    pub(crate) cause_readmission_required: bool,
    pub(in crate::data::graph) traversal: TraversalResources,
    pub(in crate::data::graph) observation: RuntimeObservation,
    #[serde(skip, default)]
    pub(in crate::data::graph) schema_registry: std::sync::Arc<SignalSchemaRegistry>,
    #[serde(skip, default)]
    pub(crate) aspect_lowering_owner: Option<SignalAspectLoweringOwner>,
    #[serde(skip, default)]
    pub(crate) conditional_dependency_versions: crate::data::persistent_ord_map::PersistentOrdMap<
        NodeId,
        crate::data::conditional_execution::SignalConditionalVersionObservation,
    >,
    #[serde(skip, default)]
    pub(crate) conditional_dependency_versions_custody: Option<
        std::sync::Arc<crate::data::retained_storage::SignalConditionalRetentionReservation>,
    >,
    #[serde(skip, default)]
    pub(crate) authorization_policy_identities:
        crate::data::persistent_ord_set::PersistentOrdSet<[u8; 32]>,
    #[serde(skip, default)]
    pub(crate) invalidation_readiness_epoch: u64,
    #[serde(skip, default)]
    pub(crate) observation_sessions: SignalObservationSessionState,
    #[serde(skip, default)]
    pub(crate) observation_capture_cleanup: Option<std::sync::Arc<ObservationCaptureCleanup>>,
    #[serde(skip, default)]
    pub(crate) invalidation_performed_counters: InvalidationPerformedCounterState,
    #[serde(skip, default)]
    pub(crate) invalidation_performed_work: PerformedWorkCaptureState,
    #[serde(skip, default)]
    pub(crate) pending_repeated_invalidation_admissions:
        crate::data::persistent_ord_map::PersistentOrdMap<NodeId, u64>,
}

#[derive(Deserialize)]
struct SignalGraphSerde {
    arena: NodeArena,
    topology: EdgeTopology,
    #[serde(default)]
    cause_sets: CanonicalCauseSetStore,
    traversal: TraversalResources,
    observation: RuntimeObservation,
}

impl<'de> Deserialize<'de> for SignalGraph {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = SignalGraphSerde::deserialize(deserializer)?;
        wire.arena
            .validate_deserialized_lane_alignment::<D::Error>()?;
        let mut graph = Self {
            lifecycle_token: Default::default(),
            instance_id: next_signal_graph_instance_id(),
            arena: wire.arena,
            topology: wire.topology,
            cause_sets: wire.cause_sets,
            cause_readmission_required: false,
            traversal: wire.traversal,
            observation: wire.observation,
            schema_registry: std::sync::Arc::new(SignalSchemaRegistry::default()),
            aspect_lowering_owner: None,
            conditional_dependency_versions: Default::default(),
            conditional_dependency_versions_custody: None,
            authorization_policy_identities: crate::data::persistent_ord_set::PersistentOrdSet::new(
            ),
            invalidation_readiness_epoch: 0,
            observation_sessions: SignalObservationSessionState::default(),
            observation_capture_cleanup: None,
            invalidation_performed_counters: InvalidationPerformedCounterState::default(),
            invalidation_performed_work: PerformedWorkCaptureState::default(),
            pending_repeated_invalidation_admissions: Default::default(),
        };
        graph.rebind_observation_capture_state();
        Ok(graph)
    }
}

/// Weak liveness observation of one concrete Signal graph owner.
#[derive(Clone)]
pub struct SignalGraphLifecycleProbe(std::sync::Weak<()>);

impl SignalGraphLifecycleProbe {
    pub fn is_live(&self) -> bool {
        self.0.strong_count() != 0
    }
}

impl SignalGraph {
    pub fn lifecycle_probe(&self) -> SignalGraphLifecycleProbe {
        SignalGraphLifecycleProbe(std::sync::Arc::downgrade(&self.lifecycle_token))
    }
}

static NEXT_SIGNAL_GRAPH_INSTANCE_ID: AtomicU64 = AtomicU64::new(1);

fn next_signal_graph_instance_id() -> u64 {
    NEXT_SIGNAL_GRAPH_INSTANCE_ID.fetch_add(1, Ordering::Relaxed)
}

pub(in crate::data::graph) fn stale_error(id: NodeId, expected_generation: u32) -> SignalError {
    SignalError::stale_handle(id, expected_generation)
}
