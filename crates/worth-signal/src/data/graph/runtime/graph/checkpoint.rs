use std::collections::BTreeMap;

use crate::data::bitset::DenseBitset;
use crate::data::dependency::{
    CanonicalDependencies, CommittedSnapshotUpdate, DependencyEdge, DependencySnapshot,
    DependencySnapshotShapeStore, DependencySnapshotStore,
};
use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::data::node::NodeEntry;
use crate::data::output::PartitionInterner;
use crate::data::proof::{PendingSnapshotBatch, PendingSnapshotCommit, SnapshotBatchCommit};
use crate::data::telemetry::RuntimeTelemetry;
use crate::schema::data::SignalSchemaRegistry;
use crate::state::{
    SignalCheckpointArena, SignalCheckpointAuthority, SignalCheckpointImage, SignalCheckpointSlot,
    SignalCheckpointTopology,
};

use crate::data::graph::compaction::CompactionState;
use crate::data::graph::storage::Slot;

use super::{
    EdgeTopology, NodeArena, ReconstructionCounters, RuntimeObservation, SignalGraph,
    TraversalResources,
};

impl SignalGraph {
    pub(crate) fn capture_checkpoint_authority(&self) -> SignalCheckpointAuthority {
        let mut graph = self.clone_stateful();
        graph
            .compact_cause_set_storage()
            .expect("checkpoint capture must compact valid canonical cause storage");
        for index in 0..graph.arena.nodes.len() {
            if !graph.arena.nodes[index].is_occupied() {
                continue;
            }
            if let Some(hot) = graph.arena.hot[index].as_mut() {
                hot.dep_snapshot_id = crate::data::dependency::DependencySnapshotId::EMPTY;
            }
            graph.arena.cold[index] = None;
        }
        graph.observation.telemetry = RuntimeTelemetry::default();
        graph.observation.reconstruction_counters = ReconstructionCounters::default();
        graph.observation.branch_mutation_view.clear();
        graph.observation.branch_mutation_records.clear();
        graph.topology.dependency_snapshots = DependencySnapshotStore::default();
        graph.topology.dependency_snapshot_shapes = DependencySnapshotShapeStore::default();
        graph.observation.diagnostics = self.observation.diagnostics.authority_carrier_clone();
        let installed_policy = graph.observation.installed_policy();
        SignalCheckpointAuthority {
            arena: SignalCheckpointArena {
                slots: graph
                    .arena
                    .nodes
                    .iter()
                    .enumerate()
                    .map(|(index, slot)| SignalCheckpointSlot {
                        node: slot.is_occupied().then(|| {
                            NodeEntry::from_storage_parts(
                                graph.arena.definitions[index].clone(),
                                graph.arena.hot[index]
                                    .clone()
                                    .expect("occupied slot must retain hot lane"),
                                graph.arena.warm[index].clone(),
                                graph.arena.cold[index].clone(),
                            )
                            .to_checkpoint_image()
                        }),
                        generation: slot.generation,
                        retired: slot.retired,
                    })
                    .collect(),
                free_list: graph.arena.free_list.iter().copied().collect(),
                active_nodes: graph.arena.active_nodes,
            },
            topology: SignalCheckpointTopology {
                dependency_edges: graph.topology.dependency_edges,
                subscriber_edges: graph.topology.subscriber_edges,
            },
            cause_sets: graph.cause_sets,
            diagnostics: graph.observation.diagnostics,
            installed_policy,
        }
    }

    pub(crate) fn restore_from_checkpoint_authority(
        authority: &SignalCheckpointAuthority,
    ) -> Result<Self, SignalError> {
        let mut free_slots = DenseBitset::default();
        for index in &authority.arena.free_list {
            free_slots.mark(*index as usize);
        }
        let instance_id = super::next_signal_graph_instance_id();
        let mut cause_sets = authority.cause_sets.clone();
        cause_sets.readmit_graph_instance(instance_id);
        let cause_readmission_required = cause_sets.has_occupied_sets();
        let observation_sessions: crate::logic::transaction::SignalObservationSessionState =
            Default::default();
        observation_sessions.set_default_surface_mask(
            authority
                .installed_policy
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
        let mut graph = Self {
            lifecycle_token: Default::default(),
            instance_id,
            arena: restore_node_arena(&authority.arena, free_slots),
            topology: EdgeTopology {
                dependency_snapshots: DependencySnapshotStore::default(),
                dependency_snapshot_shapes: DependencySnapshotShapeStore::default(),
                dependency_snapshot_storage_custody: None,
                dependency_edges: authority.topology.dependency_edges.clone(),
                subscriber_edges: authority.topology.subscriber_edges.clone(),
                reverse_subscriptions: Default::default(),
                pending_revalidation_waiters: Default::default(),
                pending_revalidation_storage_custody: None,
            },
            cause_sets,
            cause_readmission_required,
            traversal: TraversalResources::default(),
            observation: RuntimeObservation {
                telemetry: RuntimeTelemetry::default(),
                reconstruction_counters: ReconstructionCounters::default(),
                partition_interner: PartitionInterner::default(),
                branch_mutation_view: Default::default(),
                branch_mutation_records: Default::default(),
                diagnostics: authority.diagnostics.clone(),
                installed_policy: authority.installed_policy,
            },
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
        };
        let installed_policy = graph.observation.installed_policy;
        graph
            .observation
            .diagnostics
            .set_installed_policy(installed_policy);
        graph.rebuild_checkpoint_topology()?;
        Ok(graph)
    }

    pub(crate) fn restore_from_checkpoint_image(
        image: &SignalCheckpointImage,
    ) -> Result<Self, SignalError> {
        let mut graph = Self::restore_from_checkpoint_authority(&image.authority)?;
        let batch = graph.derive_dependency_snapshot_restore_batch_from_checkpoint_batch(
            &image.authority,
            &image.dependency_snapshot_batch,
        )?;
        graph.apply_classified_snapshot_batch_commit(batch.classify())?;
        graph.readmit_checkpoint_causes()?;
        graph
            .observation
            .reconstruction_counters
            .record_checkpoint_reconstruction();
        Ok(graph)
    }

    fn rebuild_checkpoint_topology(&mut self) -> Result<(), SignalError> {
        let dependency_sets = self
            .live_node_ids()
            .into_iter()
            .map(|node| {
                self.raw_dependencies_of(node)
                    .map(|edges| (node, edges.to_vec()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.topology.dependency_edges = Default::default();
        self.observation.partition_interner = PartitionInterner::default();
        let mut repaired_nodes = Vec::new();

        for (node, edges) in dependency_sets {
            let original_len = edges.len();
            let retained = edges
                .into_iter()
                .filter(|edge| self.is_alive(edge.source()))
                .collect::<Vec<_>>();
            let rebuilt = CanonicalDependencies::new(retained.iter().map(|edge| {
                match edge.scope_ref().cloned() {
                    Some(scope) => {
                        let interned = self
                            .observation
                            .partition_interner
                            .intern_subscription(&scope);
                        DependencyEdge::with_scope(edge.source(), edge.aspect(), scope, interned)
                    }
                    None => DependencyEdge::new(edge.source(), edge.aspect()),
                }
            }));
            let id = self
                .topology
                .dependency_edges
                .insert_from_slice(rebuilt.as_slice());
            self.set_dependencies_id_direct(node, id)?;
            if retained.len() != original_len {
                self.release_pending_causes(node)?;
                self.get_entry_mut(node)?.advance_dependency_revision();
                repaired_nodes.push(node);
            }
        }
        self.topology.subscriber_edges = Default::default();
        self.rebuild_subscriber_index_from_dependencies()?;
        self.rebuild_reverse_subscription_index_from_dependencies()?;
        self.rebuild_pending_revalidation_waiters()?;
        for node in repaired_nodes {
            self.transition_node_structural_revalidation(node)?;
        }
        self.cause_readmission_required = self.cause_sets.has_occupied_sets();
        if !self.cause_readmission_required {
            self.cause_sets.complete_readmission();
        }
        self.clear_branch_mutation_nodes();
        Ok(())
    }

    pub(crate) fn checkpoint_authority_arena_capacity(
        authority: &SignalCheckpointAuthority,
    ) -> usize {
        authority.arena.slots.len()
    }

    pub(crate) fn checkpoint_authority_live_node_id_at(
        authority: &SignalCheckpointAuthority,
        index: usize,
    ) -> Option<NodeId> {
        let slot = authority.arena.slots.get(index)?;
        slot.node.as_ref()?;
        Some(NodeId::new(index as u32, slot.generation))
    }

    pub(crate) fn capture_checkpoint_dependency_snapshot_batch(&self) -> SnapshotBatchCommit {
        let entries = self
            .live_node_ids()
            .into_iter()
            .filter_map(|node| {
                let snapshot = self
                    .get_dep_snapshot(node)
                    .expect("live node must have readable dependency snapshot")
                    .clone();
                (!snapshot.entries().is_empty()).then_some((node, snapshot))
            })
            .collect::<Vec<_>>();
        SnapshotBatchCommit::from_pairs(entries)
    }

    pub(crate) fn derive_dependency_snapshot_restore_batch_from_checkpoint_batch(
        &self,
        authority: &SignalCheckpointAuthority,
        checkpoint_batch: &SnapshotBatchCommit,
    ) -> Result<SnapshotBatchCommit, SignalError> {
        let target_snapshots = checkpoint_batch
            .pending()
            .as_slice()
            .iter()
            .map(|entry| {
                let target = entry
                    .update
                    .clone()
                    .apply_to(&DependencySnapshot::empty())
                    .into_snapshot();
                (entry.node, target)
            })
            .collect::<BTreeMap<_, _>>();
        let mut entries = Vec::new();
        for index in 0..Self::checkpoint_authority_arena_capacity(authority) {
            let Some(node) = Self::checkpoint_authority_live_node_id_at(authority, index) else {
                continue;
            };
            if !self.is_alive(node) {
                continue;
            }
            let previous = self.get_dep_snapshot(node)?.clone();
            let next = target_snapshots
                .get(&node)
                .cloned()
                .unwrap_or_else(DependencySnapshot::empty);
            let (_, previous_snapshot_id) = self.node_dependency_ids(node)?;
            let mut shape_store = self.topology.dependency_snapshot_shapes.clone();
            let previous_shape_handle = previous.shape().intern(&mut shape_store);
            let (update, delta) = CommittedSnapshotUpdate::between(
                node,
                previous_snapshot_id,
                previous_shape_handle,
                &previous,
                next,
            );
            if delta.changed() {
                entries.push(PendingSnapshotCommit {
                    node,
                    update,
                    delta,
                });
            }
        }
        Ok(SnapshotBatchCommit::new(PendingSnapshotBatch::new(entries)))
    }
}

// Decode each node image once; definition and evaluation lanes share one input.
fn restore_node_arena(authority: &SignalCheckpointArena, free_slots: DenseBitset) -> NodeArena {
    let mut arena = NodeArena {
        definitions: Default::default(),
        nodes: Default::default(),
        hot: Default::default(),
        warm: Default::default(),
        cold: Default::default(),
        free_list: authority.free_list.iter().copied().collect(),
        free_slots,
        active_nodes: authority.active_nodes,
        compaction: CompactionState::default(),
        retained_node_ledger: None,
        retained_node_custody: None,
        retained_seed_custody: None,
    };
    for slot in &authority.slots {
        arena.nodes.push_back(Slot {
            generation: slot.generation,
            retired: slot.retired,
            occupied: slot.node.is_some(),
        });
        let (definition, hot, warm, cold) = match &slot.node {
            Some(image) => {
                let (definition, hot, warm, cold) =
                    NodeEntry::from_checkpoint_image(image.clone()).into_storage_parts();
                (definition, Some(hot), warm, cold)
            }
            None => (Default::default(), None, Default::default(), None),
        };
        arena.definitions.push_back(definition);
        arena.hot.push_back(hot);
        arena.warm.push_back(warm);
        arena.cold.push_back(cold);
    }
    arena
}
