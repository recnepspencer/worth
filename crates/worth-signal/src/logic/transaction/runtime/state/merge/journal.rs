use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::data::graph::{BranchMutationRecord, BranchStructuralDelta};
use crate::data::handle::NodeId;

use super::core::MergeBoundaryWitness;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralMergeCandidateRecord {
    pub node: NodeId,
    pub introduced: bool,
    pub state_changed: bool,
    pub dependencies_changed: bool,
    pub dependency_snapshot_changed: bool,
    pub runtime_artifact_changed: bool,
    pub retained_artifact_changed: bool,
    pub causality_changed: bool,
    pub structural_deltas: Vec<BranchStructuralDelta>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BranchMutationJournalSlice {
    pub records: Vec<StructuralMergeCandidateRecord>,
}

impl BranchMutationJournalSlice {
    pub fn candidate_nodes(&self) -> Vec<NodeId> {
        self.records.iter().map(|record| record.node).collect()
    }

    pub fn contains_node(&self, node: NodeId) -> bool {
        self.records.iter().any(|record| record.node == node)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralMergeJournalSlice {
    pub boundary_witness: MergeBoundaryWitness,
    pub records: Vec<StructuralMergeCandidateRecord>,
}

impl StructuralMergeJournalSlice {
    pub fn from_branch_journal(
        boundary_witness: MergeBoundaryWitness,
        journal: BranchMutationJournalSlice,
    ) -> Self {
        Self {
            boundary_witness,
            records: journal.records,
        }
    }

    pub fn candidate_nodes(&self) -> Vec<NodeId> {
        self.records.iter().map(|record| record.node).collect()
    }

    pub fn contains_node(&self, node: NodeId) -> bool {
        self.records.iter().any(|record| record.node == node)
    }

    pub fn breadth(&self) -> u64 {
        self.records.len() as u64
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BranchMutationLedger {
    pub(crate) pending: BTreeMap<NodeId, BranchMutationRecord>,
    pub(crate) baseline_snapshot_id: Option<crate::state::SignalSnapshotId>,
    pub(crate) boundary_established: bool,
}

impl BranchMutationLedger {
    pub fn with_baseline_snapshot(
        mut self,
        snapshot_id: Option<crate::state::SignalSnapshotId>,
    ) -> Self {
        self.baseline_snapshot_id = snapshot_id;
        self.boundary_established = true;
        self
    }

    pub(crate) fn absorb_records(
        &mut self,
        records: impl IntoIterator<Item = (NodeId, BranchMutationRecord)>,
    ) {
        for (node, record) in records {
            self.pending.entry(node).or_default().absorb(record);
        }
    }

    pub fn structural_merge_journal(&self) -> BranchMutationJournalSlice {
        BranchMutationJournalSlice {
            records: self
                .pending
                .iter()
                .filter(|(_, record)| record.merge_relevant())
                .map(|(node, record)| StructuralMergeCandidateRecord {
                    node: *node,
                    introduced: record.introduced,
                    state_changed: record.state_changed,
                    dependencies_changed: record.dependencies_changed,
                    dependency_snapshot_changed: record.dependency_snapshot_changed,
                    runtime_artifact_changed: record.runtime_artifact_changed,
                    retained_artifact_changed: record.retained_artifact_changed,
                    causality_changed: record.causality_changed,
                    structural_deltas: record.structural_deltas.clone(),
                })
                .collect(),
        }
    }

    pub fn clear_all(&mut self, baseline_snapshot_id: Option<crate::state::SignalSnapshotId>) {
        self.pending.clear();
        self.baseline_snapshot_id = baseline_snapshot_id;
        self.boundary_established = true;
    }

    pub fn clear_merged_nodes(
        &mut self,
        merged_nodes: impl IntoIterator<Item = NodeId>,
        baseline_snapshot_id: Option<crate::state::SignalSnapshotId>,
    ) {
        for node in merged_nodes {
            self.pending.remove(&node);
        }
        self.baseline_snapshot_id = baseline_snapshot_id;
        self.boundary_established = true;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MergeNodeMap {
    pub source_to_target: BTreeMap<NodeId, NodeId>,
}

impl MergeNodeMap {
    pub fn insert(&mut self, source: NodeId, target: NodeId) {
        self.source_to_target.insert(source, target);
    }

    pub fn resolve(&self, source: NodeId) -> Option<NodeId> {
        self.source_to_target.get(&source).copied()
    }
}
