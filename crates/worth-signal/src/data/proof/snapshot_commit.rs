use serde::{Deserialize, Serialize};

use crate::data::dependency::{
    CommittedSnapshotUpdate, DependencySnapshot, ReplacementSnapshotUpdate,
    SharedDependencySnapshot, SnapshotDeltaRecord, VersionOnlySnapshotUpdate,
};
use crate::data::handle::NodeId;

use super::locality::{node_sort_key, DedupedNodeBatch};
use super::SummaryForm;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingSnapshotCommit {
    pub node: NodeId,
    pub update: CommittedSnapshotUpdate,
    pub delta: SnapshotDeltaRecord,
}

impl PendingSnapshotCommit {
    pub fn is_stable_shape(&self) -> bool {
        matches!(self.update, CommittedSnapshotUpdate::VersionOnly(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingStableShapeSnapshotCommit {
    node: NodeId,
    update: VersionOnlySnapshotUpdate,
    delta: SnapshotDeltaRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingReplacementSnapshotCommit {
    node: NodeId,
    update: ReplacementSnapshotUpdate,
    delta: SnapshotDeltaRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PendingSnapshotBatch {
    entries: Vec<PendingSnapshotCommit>,
}

impl PendingSnapshotBatch {
    pub fn new(entries: impl IntoIterator<Item = PendingSnapshotCommit>) -> Self {
        let mut entries = entries.into_iter().collect::<Vec<_>>();
        if entries.len() > 1 {
            entries.sort_unstable_by_key(|entry| node_sort_key(&entry.node));
            entries.dedup_by(|left, right| left.node == right.node);
        }
        Self { entries }
    }

    pub fn from_pairs(entries: impl IntoIterator<Item = (NodeId, DependencySnapshot)>) -> Self {
        let entries = entries
            .into_iter()
            .map(|(node, snapshot)| {
                let snapshot = SharedDependencySnapshot::new(snapshot.canonicalize_unordered());
                PendingSnapshotCommit {
                    node,
                    delta: SnapshotDeltaRecord::between(
                        node,
                        &DependencySnapshot::empty(),
                        &snapshot,
                    ),
                    update: CommittedSnapshotUpdate::Replace(
                        ReplacementSnapshotUpdate::from_snapshot(snapshot.into_snapshot()),
                    ),
                }
            })
            .collect::<Vec<_>>();
        Self::new(entries)
    }

    pub(crate) fn from_unique_pending_snapshots_in_stage_order(
        entries: impl IntoIterator<Item = crate::logic::evaluation::PendingDependencySnapshot>,
    ) -> Self {
        let entries = entries
            .into_iter()
            .map(|pending| PendingSnapshotCommit {
                node: pending.node,
                update: pending.update,
                delta: pending.delta,
            })
            .collect::<Vec<_>>();
        debug_assert!(pending_snapshot_nodes_are_unique(entries.as_slice()));
        Self { entries }
    }

    pub fn as_slice(&self) -> &[PendingSnapshotCommit] {
        &self.entries
    }

    pub fn into_vec(self) -> Vec<PendingSnapshotCommit> {
        self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn is_stable_shape_only(&self) -> bool {
        !self.entries.is_empty()
            && self
                .entries
                .iter()
                .all(PendingSnapshotCommit::is_stable_shape)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SnapshotBatchCommit {
    pending: PendingSnapshotBatch,
    target_nodes: DedupedNodeBatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StableShapeSnapshotBatchCommit {
    pending: Vec<PendingStableShapeSnapshotCommit>,
    target_nodes: DedupedNodeBatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MixedSnapshotBatchCommit {
    stable_shape: Vec<PendingStableShapeSnapshotCommit>,
    replacements: Vec<PendingReplacementSnapshotCommit>,
    target_nodes: DedupedNodeBatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClassifiedSnapshotBatchCommit {
    StableShape(StableShapeSnapshotBatchCommit),
    Mixed(MixedSnapshotBatchCommit),
}

impl SnapshotBatchCommit {
    pub fn new(pending: PendingSnapshotBatch) -> Self {
        let target_nodes = DedupedNodeBatch::new(pending.as_slice().iter().map(|entry| entry.node));
        Self {
            pending,
            target_nodes,
        }
    }

    pub fn from_pairs(entries: impl IntoIterator<Item = (NodeId, DependencySnapshot)>) -> Self {
        Self::new(PendingSnapshotBatch::from_pairs(entries))
    }

    pub(crate) fn from_unique_pending_snapshots_in_stage_order(
        entries: impl IntoIterator<Item = crate::logic::evaluation::PendingDependencySnapshot>,
    ) -> Self {
        Self::new(PendingSnapshotBatch::from_unique_pending_snapshots_in_stage_order(entries))
    }

    pub fn pending(&self) -> &PendingSnapshotBatch {
        &self.pending
    }

    pub fn target_nodes(&self) -> &DedupedNodeBatch {
        &self.target_nodes
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn classify(self) -> ClassifiedSnapshotBatchCommit {
        if self.pending.is_stable_shape_only() {
            let pending = self
                .pending
                .into_vec()
                .into_iter()
                .map(|entry| match entry.update {
                    CommittedSnapshotUpdate::VersionOnly(update) => {
                        PendingStableShapeSnapshotCommit {
                            node: entry.node,
                            update,
                            delta: entry.delta,
                        }
                    }
                    CommittedSnapshotUpdate::Replace(_) => {
                        unreachable!("stable-shape classification must exclude replacement entries")
                    }
                })
                .collect::<Vec<_>>();
            ClassifiedSnapshotBatchCommit::StableShape(StableShapeSnapshotBatchCommit {
                pending,
                target_nodes: self.target_nodes,
            })
        } else {
            let mut stable_shape = Vec::new();
            let mut replacements = Vec::new();
            for entry in self.pending.into_vec() {
                match entry.update {
                    CommittedSnapshotUpdate::VersionOnly(update) => {
                        stable_shape.push(PendingStableShapeSnapshotCommit {
                            node: entry.node,
                            update,
                            delta: entry.delta,
                        });
                    }
                    CommittedSnapshotUpdate::Replace(update) => {
                        replacements.push(PendingReplacementSnapshotCommit {
                            node: entry.node,
                            update,
                            delta: entry.delta,
                        });
                    }
                }
            }
            ClassifiedSnapshotBatchCommit::Mixed(MixedSnapshotBatchCommit {
                stable_shape,
                replacements,
                target_nodes: self.target_nodes,
            })
        }
    }
}

impl StableShapeSnapshotBatchCommit {
    pub fn node(&self, index: usize) -> Option<NodeId> {
        self.pending
            .get(index)
            .map(PendingStableShapeSnapshotCommit::node)
    }

    pub fn pending(&self) -> &[PendingStableShapeSnapshotCommit] {
        self.pending.as_slice()
    }

    pub fn target_nodes(&self) -> &DedupedNodeBatch {
        &self.target_nodes
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

impl MixedSnapshotBatchCommit {
    pub fn stable_shape(&self) -> &[PendingStableShapeSnapshotCommit] {
        self.stable_shape.as_slice()
    }

    pub fn replacements(&self) -> &[PendingReplacementSnapshotCommit] {
        self.replacements.as_slice()
    }

    pub fn target_nodes(&self) -> &DedupedNodeBatch {
        &self.target_nodes
    }

    pub fn is_empty(&self) -> bool {
        self.stable_shape.is_empty() && self.replacements.is_empty()
    }
}

impl ClassifiedSnapshotBatchCommit {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::StableShape(commit) => commit.is_empty(),
            Self::Mixed(commit) => commit.is_empty(),
        }
    }

    pub fn target_nodes(&self) -> &DedupedNodeBatch {
        match self {
            Self::StableShape(commit) => commit.target_nodes(),
            Self::Mixed(commit) => commit.target_nodes(),
        }
    }
}

impl PendingStableShapeSnapshotCommit {
    pub fn node(&self) -> NodeId {
        self.node
    }

    pub fn update(&self) -> &VersionOnlySnapshotUpdate {
        &self.update
    }

    pub fn delta(&self) -> SnapshotDeltaRecord {
        self.delta
    }
}

impl PendingReplacementSnapshotCommit {
    pub fn node(&self) -> NodeId {
        self.node
    }

    pub fn update(&self) -> &ReplacementSnapshotUpdate {
        &self.update
    }

    pub fn delta(&self) -> SnapshotDeltaRecord {
        self.delta
    }
}

impl SummaryForm for PendingSnapshotBatch {}
impl SummaryForm for SnapshotBatchCommit {}
impl SummaryForm for StableShapeSnapshotBatchCommit {}
impl SummaryForm for MixedSnapshotBatchCommit {}

fn pending_snapshot_nodes_are_unique(entries: &[PendingSnapshotCommit]) -> bool {
    let mut seen = std::collections::HashSet::with_capacity(entries.len());
    entries.iter().all(|entry| seen.insert(entry.node))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::aspect::Aspect;

    #[test]
    fn snapshot_batch_commit_classifies_stable_shape_batches() {
        let node = NodeId::new(0, 0);
        let mut snapshot = DependencySnapshot::empty();
        snapshot.record(NodeId::new(1, 0), Aspect::new(0), 3, None);
        let mut shape_store = crate::data::dependency::DependencySnapshotShapeStore::default();
        let basis = crate::data::dependency::StableShapeSnapshotBasis::prove(
            &crate::data::dependency::DependencyInputScan::stable_shape(
                node,
                crate::data::dependency::DependencySnapshotId::EMPTY,
                1,
                1,
                vec![5],
            ),
            snapshot.shape().intern(&mut shape_store),
        )
        .expect("proof should exist");
        let update = crate::data::dependency::CommittedSnapshotUpdate::VersionOnly(
            crate::data::dependency::VersionOnlySnapshotUpdate::from_basis_and_versions(
                basis.clone(),
                crate::data::dependency::VersionVector::from_scan(
                    &basis,
                    &crate::data::dependency::DependencyInputScan::stable_shape(
                        node,
                        crate::data::dependency::DependencySnapshotId::EMPTY,
                        1,
                        1,
                        vec![5],
                    ),
                ),
            ),
        );
        let batch = SnapshotBatchCommit::new(PendingSnapshotBatch::new([PendingSnapshotCommit {
            node,
            update,
            delta: SnapshotDeltaRecord::for_version_update(node, &snapshot, &[5]),
        }]));

        assert!(matches!(
            batch.classify(),
            ClassifiedSnapshotBatchCommit::StableShape(_)
        ));
    }
}
