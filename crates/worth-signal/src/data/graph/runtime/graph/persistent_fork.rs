use std::sync::Arc;

use super::{
    InvalidationPerformedCounterState, ObservationCaptureCleanup, PerformedWorkCaptureState,
    SignalGraph, TraversalResources,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SignalGraphForkWork {
    copied_mutable_graph_nodes: u64,
}

impl SignalGraphForkWork {
    const fn shared_without_node_copy() -> Self {
        Self {
            copied_mutable_graph_nodes: 0,
        }
    }

    pub(crate) const fn copied_mutable_graph_nodes(self) -> u64 {
        self.copied_mutable_graph_nodes
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SignalGraphPersistentSharing {
    pub(crate) arena_root_shared: bool,
    pub(crate) topology_root_shared: bool,
    pub(crate) cause_root_shared: bool,
    pub(crate) schema_root_shared: bool,
    pub(crate) observation_root_shared: bool,
    pub(crate) keyed_roots_shared: bool,
}

#[cfg(test)]
pub(crate) struct SignalGraphPersistentIdentity {
    arena: super::NodeArena,
    topology: super::EdgeTopology,
    cause_sets: crate::data::graph::storage::invalidation_causes::CanonicalCauseSetStore,
    schema_registry: Arc<crate::schema::data::SignalSchemaRegistry>,
    partition_interner: crate::data::output::PartitionInterner,
    conditional_dependency_versions: crate::data::persistent_ord_map::PersistentOrdMap<
        crate::data::handle::NodeId,
        crate::data::conditional_execution::SignalConditionalVersionObservation,
    >,
    authorization_policy_identities: crate::data::persistent_ord_set::PersistentOrdSet<[u8; 32]>,
}

#[cfg(test)]
impl SignalGraphPersistentIdentity {
    pub(crate) fn sharing_with(&self, other: &Self) -> SignalGraphPersistentSharing {
        SignalGraphPersistentSharing {
            arena_root_shared: self.arena.shares_storage_with(&other.arena),
            topology_root_shared: self.topology.shares_storage_with(&other.topology),
            cause_root_shared: self.cause_sets.shares_storage_with(&other.cause_sets),
            schema_root_shared: Arc::ptr_eq(&self.schema_registry, &other.schema_registry),
            observation_root_shared: self
                .partition_interner
                .shares_storage_with(&other.partition_interner),
            keyed_roots_shared: self
                .conditional_dependency_versions
                .ptr_eq(&other.conditional_dependency_versions)
                && self
                    .authorization_policy_identities
                    .ptr_eq(&other.authorization_policy_identities),
        }
    }
}

impl SignalGraph {
    /// Fork one owner-managed branch while retaining the installed external
    /// lowering owner that gives copied definition custody its meaning.
    /// Branch-local pending admissions remain reset by `fork_persistent`.
    pub(crate) fn fork_owner_branch(&mut self) -> (Self, SignalGraphForkWork) {
        let lowering_owner = self.aspect_lowering_owner.clone();
        let (mut successor, work) = self.fork_persistent();
        successor.aspect_lowering_owner = lowering_owner;
        (successor, work)
    }

    /// Build an isolated, persistent candidate for a same-cell conditional
    /// mutation. Unlike a branch fork, this candidate retains definition
    /// claimant and pending-admission state because it represents the next
    /// basis of the same graph lineage.
    pub(crate) fn fork_conditional_successor(&mut self) -> (Self, SignalGraphForkWork) {
        let lowering_owner = self.aspect_lowering_owner.clone();
        let repeated_admissions = self
            .pending_repeated_invalidation_admissions
            .fork_persistent();
        let (mut successor, work) = self.fork_persistent();
        successor.aspect_lowering_owner = lowering_owner;
        successor.pending_repeated_invalidation_admissions = repeated_admissions;
        (successor, work)
    }

    pub(crate) fn fork_persistent(&mut self) -> (Self, SignalGraphForkWork) {
        let observation_sessions: crate::logic::transaction::SignalObservationSessionState =
            Default::default();
        observation_sessions.set_default_surface_mask(
            self.installed_runtime_policy()
                .observation_capture_plan()
                .default_surface_mask(),
        );
        let invalidation_performed_counters = InvalidationPerformedCounterState::with_capture_gate(
            observation_sessions.capture_gate(),
        );
        let invalidation_performed_work =
            PerformedWorkCaptureState::with_capture_gate(observation_sessions.capture_gate());
        let observation_capture_cleanup = Arc::new(ObservationCaptureCleanup::new(
            invalidation_performed_counters.shared_values(),
            invalidation_performed_work.shared_bindings(),
            observation_sessions.shared_completed_execution_boundaries(),
            observation_sessions.shared_last_completion(),
        ));
        (
            Self {
                lifecycle_token: Default::default(),
                instance_id: self.instance_id,
                arena: self.arena.fork_persistent(),
                topology: self.topology.fork_persistent(),
                cause_sets: self.cause_sets.fork_persistent(),
                cause_readmission_required: self.cause_readmission_required,
                traversal: TraversalResources::default(),
                observation: self.observation.fork_branch_local(),
                schema_registry: Arc::clone(&self.schema_registry),
                aspect_lowering_owner: None,
                conditional_dependency_versions: self
                    .conditional_dependency_versions
                    .fork_persistent(),
                conditional_dependency_versions_custody: self
                    .conditional_dependency_versions_custody
                    .clone(),
                authorization_policy_identities: self
                    .authorization_policy_identities
                    .fork_persistent(),
                invalidation_readiness_epoch: self.invalidation_readiness_epoch,
                observation_sessions,
                observation_capture_cleanup: Some(observation_capture_cleanup),
                invalidation_performed_counters,
                invalidation_performed_work,
                pending_repeated_invalidation_admissions: Default::default(),
            },
            SignalGraphForkWork::shared_without_node_copy(),
        )
    }

    #[cfg(test)]
    pub(crate) fn persistent_identity(&self) -> SignalGraphPersistentIdentity {
        SignalGraphPersistentIdentity {
            arena: self.arena.clone(),
            topology: self.topology.fork_storage_identity(),
            cause_sets: self.cause_sets.fork_storage_identity(),
            schema_registry: Arc::clone(&self.schema_registry),
            partition_interner: self.observation.fork_storage_identity(),
            conditional_dependency_versions: self
                .conditional_dependency_versions
                .fork_storage_identity(),
            authorization_policy_identities: self
                .authorization_policy_identities
                .fork_storage_identity(),
        }
    }

    #[cfg(test)]
    pub(crate) fn hot_page_identities(&self) -> Vec<usize> {
        self.arena.hot.page_identities()
    }
}
