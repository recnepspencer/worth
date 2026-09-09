use crate::data::bitset::DenseBitset;
use crate::data::graph::compaction::CompactionState;
use crate::schema::data::SignalSchemaRegistry;

use super::{EdgeTopology, NodeArena, RuntimeObservation, SignalGraph, TraversalResources};

impl Clone for SignalGraph {
    fn clone(&self) -> Self {
        let instance_id = super::next_signal_graph_instance_id();
        let mut cause_sets = self.cause_sets.operational_clone();
        cause_sets.readmit_graph_instance(instance_id);
        let observation_sessions: crate::logic::transaction::SignalObservationSessionState =
            Default::default();
        observation_sessions.set_default_surface_mask(
            self.installed_runtime_policy()
                .observation_capture_plan()
                .default_surface_mask(),
        );
        let invalidation_performed_counters =
            super::InvalidationPerformedCounterState::with_capture_gate(
                observation_sessions.capture_gate(),
            );
        let invalidation_performed_work = super::PerformedWorkCaptureState::with_capture_gate(
            observation_sessions.capture_gate(),
        );
        let observation_capture_cleanup =
            std::sync::Arc::new(super::ObservationCaptureCleanup::new(
                invalidation_performed_counters.shared_values(),
                invalidation_performed_work.shared_bindings(),
                observation_sessions.shared_completed_execution_boundaries(),
                observation_sessions.shared_last_completion(),
            ));
        Self {
            lifecycle_token: Default::default(),
            instance_id,
            arena: self.arena.operational_clone(),
            topology: self.topology.operational_clone(),
            cause_sets,
            cause_readmission_required: self.cause_readmission_required,
            traversal: self.traversal.clone(),
            observation: self.observation.operational_clone(),
            schema_registry: std::sync::Arc::new((*self.schema_registry).clone()),
            aspect_lowering_owner: None,
            conditional_dependency_versions: self
                .conditional_dependency_versions
                .operational_clone(),
            conditional_dependency_versions_custody: None,
            authorization_policy_identities: self
                .authorization_policy_identities
                .operational_clone(),
            invalidation_readiness_epoch: 0,
            invalidation_performed_counters,
            invalidation_performed_work,
            observation_sessions,
            observation_capture_cleanup: Some(observation_capture_cleanup),
            pending_repeated_invalidation_admissions: Default::default(),
        }
    }
}

impl Default for SignalGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalGraph {
    pub(super) const PARALLELISM_NODE_THRESHOLD: usize = 1_000;
    pub(super) const GC_PRESSURE_TOMBSTONE_RATIO: f32 = 0.30;

    pub fn new() -> Self {
        let observation_sessions: crate::logic::transaction::SignalObservationSessionState =
            Default::default();
        let default_observation_mask =
            crate::runtime_policy::InstalledSignalRuntimePolicy::default()
                .observation_capture_plan()
                .default_surface_mask();
        observation_sessions.set_default_surface_mask(default_observation_mask);
        let invalidation_performed_counters =
            super::InvalidationPerformedCounterState::with_capture_gate(
                observation_sessions.capture_gate(),
            );
        let invalidation_performed_work = super::PerformedWorkCaptureState::with_capture_gate(
            observation_sessions.capture_gate(),
        );
        let observation_capture_cleanup =
            std::sync::Arc::new(super::ObservationCaptureCleanup::new(
                invalidation_performed_counters.shared_values(),
                invalidation_performed_work.shared_bindings(),
                observation_sessions.shared_completed_execution_boundaries(),
                observation_sessions.shared_last_completion(),
            ));
        Self {
            lifecycle_token: Default::default(),
            instance_id: super::next_signal_graph_instance_id(),
            arena: NodeArena {
                definitions: Default::default(),
                nodes: crate::data::persistent_paged_vector::PersistentPagedVector::new(),
                hot: crate::data::persistent_paged_vector::PersistentPagedVector::new(),
                warm: crate::data::persistent_paged_vector::PersistentPagedVector::new(),
                cold: crate::data::persistent_paged_vector::PersistentPagedVector::new(),
                free_list: crate::data::persistent_vector::PersistentVector::new(),
                free_slots: DenseBitset::default(),
                active_nodes: 0,
                compaction: CompactionState::default(),
                retained_node_ledger: None,
                retained_node_custody: None,
                retained_seed_custody: None,
            },
            topology: EdgeTopology::default(),
            cause_sets: Default::default(),
            cause_readmission_required: false,
            traversal: TraversalResources::default(),
            observation: RuntimeObservation::default(),
            schema_registry: std::sync::Arc::new(SignalSchemaRegistry::default()),
            aspect_lowering_owner: None,
            conditional_dependency_versions: Default::default(),
            conditional_dependency_versions_custody: None,
            authorization_policy_identities: crate::data::persistent_ord_set::PersistentOrdSet::new(
            ),
            invalidation_readiness_epoch: 0,
            invalidation_performed_counters,
            invalidation_performed_work,
            observation_sessions,
            observation_capture_cleanup: Some(observation_capture_cleanup),
            pending_repeated_invalidation_admissions: Default::default(),
        }
    }

    pub(crate) const fn runtime_instance_id(&self) -> u64 {
        self.instance_id
    }

    pub fn with_gc_threshold(gc_threshold: u32) -> Self {
        let mut graph = Self::new();
        graph.arena.compaction = CompactionState::new(gc_threshold);
        graph
    }

    pub(crate) fn clone_stateful(&self) -> Self {
        self.clone()
    }

    pub fn with_schema_registry(mut self, schema_registry: SignalSchemaRegistry) -> Self {
        self.schema_registry = std::sync::Arc::new(schema_registry);
        self
    }

    pub fn set_schema_registry(&mut self, schema_registry: SignalSchemaRegistry) {
        self.schema_registry = std::sync::Arc::new(schema_registry);
    }

    pub fn schema_registry(&self) -> &SignalSchemaRegistry {
        &self.schema_registry
    }
}
