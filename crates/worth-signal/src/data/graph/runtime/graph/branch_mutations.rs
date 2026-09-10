mod publication_work;
use serde::{Deserialize, Serialize};

use crate::data::core_profile::StableHashValue;
use crate::data::dependency::{DependencyEdge, SnapshotDeltaRecord};
use crate::data::handle::NodeId;
use crate::data::reuse::ReuseBasis;
use crate::diagnostics::lineage::LineageArtifactId;

use super::SignalGraph;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub(crate) struct BranchMutationRecord {
    pub introduced: bool,
    pub state_changed: bool,
    pub dependencies_changed: bool,
    pub dependency_snapshot_changed: bool,
    pub runtime_artifact_changed: bool,
    pub retained_artifact_changed: bool,
    pub causality_changed: bool,
    #[serde(default)]
    pub structural_deltas: Vec<BranchStructuralDelta>,
}

#[derive(Debug, Clone)]
pub(crate) struct BranchMutationNodeImage {
    view: Option<BranchMutationRecord>,
    pending: Option<BranchMutationRecord>,
}

impl BranchMutationRecord {
    pub(crate) fn merge_relevant(&self) -> bool {
        self.introduced
            || self.state_changed
            || self.dependencies_changed
            || self.dependency_snapshot_changed
    }

    fn mark_introduced(&mut self) {
        self.introduced = true;
        self.state_changed = true;
        self.structural_deltas
            .push(BranchStructuralDelta::NodeIntroduced);
    }

    fn mark_state_changed(&mut self) {
        self.state_changed = true;
        self.structural_deltas
            .push(BranchStructuralDelta::NodeStateChanged);
    }

    fn mark_dependencies_changed(&mut self, delta: DependencyTopologyDelta) {
        self.dependencies_changed = true;
        if let Some(BranchStructuralDelta::DependencyTopologyChanged(existing)) = self
            .structural_deltas
            .iter_mut()
            .find(|delta| matches!(delta, BranchStructuralDelta::DependencyTopologyChanged(_)))
        {
            merge_dependency_topology_delta(existing, delta);
        } else {
            self.structural_deltas
                .push(BranchStructuralDelta::DependencyTopologyChanged(delta));
        }
    }

    fn mark_dependency_snapshot_changed(&mut self, delta: DependencySnapshotStructuralDelta) {
        self.dependency_snapshot_changed = true;
        self.structural_deltas
            .push(BranchStructuralDelta::DependencySnapshotChanged(delta));
    }

    fn mark_runtime_artifact_changed(&mut self, delta: RuntimeArtifactStructuralDelta) {
        self.runtime_artifact_changed = true;
        self.structural_deltas
            .push(BranchStructuralDelta::RuntimeArtifactChanged(delta));
    }

    fn mark_retained_artifact_changed(&mut self) {
        self.retained_artifact_changed = true;
        self.structural_deltas
            .push(BranchStructuralDelta::RetainedArtifactChanged);
    }

    fn mark_causality_changed(&mut self) {
        self.causality_changed = true;
        self.structural_deltas
            .push(BranchStructuralDelta::CausalityChanged);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BranchStructuralDelta {
    NodeIntroduced,
    NodeStateChanged,
    DependencyTopologyChanged(DependencyTopologyDelta),
    DependencySnapshotChanged(DependencySnapshotStructuralDelta),
    RuntimeArtifactChanged(RuntimeArtifactStructuralDelta),
    RetainedArtifactChanged,
    CausalityChanged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DependencyTopologyDelta {
    pub added_edges: Vec<DependencyEdge>,
    pub removed_edges: Vec<DependencyEdge>,
}

fn merge_dependency_topology_delta(
    existing: &mut DependencyTopologyDelta,
    delta: DependencyTopologyDelta,
) {
    for added in delta.added_edges {
        if let Some(index) = existing
            .removed_edges
            .iter()
            .position(|edge| edge == &added)
        {
            existing.removed_edges.remove(index);
        } else if !existing.added_edges.iter().any(|edge| edge == &added) {
            existing.added_edges.push(added);
        }
    }

    for removed in delta.removed_edges {
        if let Some(index) = existing
            .added_edges
            .iter()
            .position(|edge| edge == &removed)
        {
            existing.added_edges.remove(index);
        } else if !existing.removed_edges.iter().any(|edge| edge == &removed) {
            existing.removed_edges.push(removed);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencySnapshotStructuralDelta {
    pub previous_entry_count: u32,
    pub next_entry_count: u32,
    pub changed_entry_count: u32,
}

impl DependencySnapshotStructuralDelta {
    pub(crate) fn from_snapshot_delta(delta: SnapshotDeltaRecord) -> Self {
        Self {
            previous_entry_count: delta.previous_entry_count,
            next_entry_count: delta.next_entry_count,
            changed_entry_count: delta.changed_entry_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeArtifactStructuralDelta {
    pub previous_artifact_id: Option<LineageArtifactId>,
    pub next_artifact_id: Option<LineageArtifactId>,
    pub previous_output_hash: Option<StableHashValue>,
    pub next_output_hash: Option<StableHashValue>,
    pub previous_reuse_basis: Option<ReuseBasis>,
    pub next_reuse_basis: Option<ReuseBasis>,
}

impl SignalGraph {
    pub(crate) fn branch_mutation_node_image(&self, node: NodeId) -> BranchMutationNodeImage {
        BranchMutationNodeImage {
            view: self.observation.branch_mutation_view.get(&node).cloned(),
            pending: self.observation.branch_mutation_records.get(&node).cloned(),
        }
    }

    pub(crate) fn restore_branch_mutation_node_image(
        &mut self,
        node: NodeId,
        image: BranchMutationNodeImage,
    ) {
        restore_optional_record(&mut self.observation.branch_mutation_view, node, image.view);
        restore_optional_record(
            &mut self.observation.branch_mutation_records,
            node,
            image.pending,
        );
    }

    fn record_branch_mutation(
        &mut self,
        node: NodeId,
        mut update: impl FnMut(&mut BranchMutationRecord),
    ) {
        update(
            self.observation
                .branch_mutation_view
                .entry(node)
                .or_default(),
        );
        update(
            self.observation
                .branch_mutation_records
                .entry(node)
                .or_default(),
        );
    }

    pub(crate) fn record_branch_mutation_introduced(&mut self, node: NodeId) {
        self.record_branch_mutation(node, BranchMutationRecord::mark_introduced);
    }

    pub(crate) fn record_branch_mutation_state(&mut self, node: NodeId) {
        self.record_branch_mutation(node, BranchMutationRecord::mark_state_changed);
    }

    pub(crate) fn record_branch_mutation_dependencies(
        &mut self,
        node: NodeId,
        delta: DependencyTopologyDelta,
    ) {
        self.record_branch_mutation(node, |record| {
            record.mark_dependencies_changed(delta.clone())
        });
    }

    pub(crate) fn record_branch_mutation_snapshot(
        &mut self,
        node: NodeId,
        delta: DependencySnapshotStructuralDelta,
    ) {
        self.record_branch_mutation(node, |record| {
            record.mark_dependency_snapshot_changed(delta.clone())
        });
    }

    pub(crate) fn record_branch_mutation_runtime_artifact(
        &mut self,
        node: NodeId,
        delta: RuntimeArtifactStructuralDelta,
    ) {
        self.record_branch_mutation(node, |record| {
            record.mark_runtime_artifact_changed(delta.clone())
        });
    }

    pub(crate) fn record_branch_mutation_retained_artifact(&mut self, node: NodeId) {
        self.record_branch_mutation(node, BranchMutationRecord::mark_retained_artifact_changed);
    }

    pub(crate) fn record_branch_mutation_causality(&mut self, node: NodeId) {
        self.record_branch_mutation(node, BranchMutationRecord::mark_causality_changed);
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn branch_mutation_records(&self) -> Vec<(NodeId, BranchMutationRecord)> {
        self.observation
            .branch_mutation_view
            .iter()
            .map(|(node, record)| (*node, record.clone()))
            .collect()
    }

    pub(crate) fn pending_branch_mutation_records(&self) -> Vec<(NodeId, BranchMutationRecord)> {
        self.observation
            .branch_mutation_records
            .iter()
            .map(|(node, record)| (*node, record.clone()))
            .collect()
    }

    pub(crate) fn clear_branch_mutation_nodes(&mut self) {
        self.observation.branch_mutation_records.clear();
    }
}

fn restore_optional_record(
    records: &mut crate::data::persistent_ord_map::PersistentOrdMap<NodeId, BranchMutationRecord>,
    node: NodeId,
    record: Option<BranchMutationRecord>,
) {
    match record {
        Some(record) => {
            records.insert(node, record);
        }
        None => {
            records.remove(&node);
        }
    }
}
