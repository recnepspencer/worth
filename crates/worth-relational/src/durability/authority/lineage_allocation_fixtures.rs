//! Value-only committed lineage events for the durability allocation courts.
//!
//! The canonical history mirrors commit finalization's emission order:
//! allocating events ascend by record target (entities before relations),
//! drawing ascending lineage ids from the one shared allocator.
//!
//! - commit 1 creates entity `e1` on lineage 1;
//! - commit 2 creates `e2` (lineage 2), restores a retired name from
//!   lineage 1 onto `e3` (lineage 3), and creates a relation (lineage 4);
//! - commit 3 only restores a retired name from lineage 2 onto `e5`
//!   (lineage 5).

use std::collections::BTreeMap;

use crate::history::data::{BranchId, CommitId, RelationalCommitReceipt};
use crate::identity::data::{EntityId, LineageId, PartitionId, VersionId};
use crate::lineage::data::{LineageEventKind, LineageEventRecord, LineageSubject};

use super::lineage_allocation::CommitAllocationView;

pub(super) fn entity(slot: u64) -> EntityId {
    EntityId::new(PartitionId::main(), slot, 1)
}

pub(super) fn receipt(commit: u64) -> RelationalCommitReceipt {
    RelationalCommitReceipt {
        commit_id: CommitId(commit),
        version_id: VersionId(commit),
        branch_id: BranchId("main".to_owned()),
        parents: (commit > 1)
            .then(|| CommitId(commit - 1))
            .into_iter()
            .collect(),
    }
}

/// One commit's lineage events and the entities its patch created.
#[derive(Clone)]
pub(super) struct CommitLineage {
    pub(super) receipt: RelationalCommitReceipt,
    pub(super) events: Vec<LineageEventRecord>,
    pub(super) created_entities: Vec<EntityId>,
}

impl CommitLineage {
    fn new(commit: u64, created_entities: Vec<EntityId>) -> Self {
        Self {
            receipt: receipt(commit),
            events: Vec::new(),
            created_entities,
        }
    }

    fn push(
        &mut self,
        kind: LineageEventKind,
        subject: LineageSubject,
        sources: &[u64],
        targets: &[u64],
    ) {
        let event_id = self.receipt.commit_id.0 * 10 + self.events.len() as u64;
        self.events.push(LineageEventRecord {
            event_id,
            commit: self.receipt.clone(),
            branch_id: self.receipt.branch_id.clone(),
            kind,
            sources: sources.iter().copied().map(LineageId).collect(),
            targets: targets.iter().copied().map(LineageId).collect(),
            subject,
            name_birth: None,
            kept_name: None,
            relation_id: None,
        });
    }

    pub(super) fn view(&self) -> CommitAllocationView<'_> {
        CommitAllocationView {
            commit: &self.receipt,
            events: &self.events,
            created_entities: self.created_entities.clone(),
        }
    }

    /// The event allocating `lineage`.
    pub(super) fn allocating(&mut self, lineage: u64) -> &mut LineageEventRecord {
        self.events
            .iter_mut()
            .find(|event| event.targets() == [LineageId(lineage)])
            .expect("the commit allocates the lineage")
    }
}

/// The three canonical commits described in the module documentation.
pub(super) fn restore_history() -> [CommitLineage; 3] {
    use LineageEventKind::{Create, Restore};
    use LineageSubject::{Entity, Relation};
    let mut first = CommitLineage::new(1, vec![entity(1)]);
    first.push(Create, Entity, &[], &[1]);
    let mut mixed = CommitLineage::new(2, vec![entity(3), entity(2)]);
    mixed.push(Create, Entity, &[], &[2]);
    mixed.push(Restore, Entity, &[1], &[3]);
    mixed.push(Create, Relation, &[], &[4]);
    let mut restore_only = CommitLineage::new(3, vec![entity(5)]);
    restore_only.push(Restore, Entity, &[2], &[5]);
    [first, mixed, restore_only]
}

/// The canonical entity lineage nodes of [`restore_history`].
pub(super) fn restore_history_nodes() -> BTreeMap<LineageId, EntityId> {
    [(1, 1), (2, 2), (3, 3), (5, 5)]
        .into_iter()
        .map(|(lineage, slot)| (LineageId(lineage), entity(slot)))
        .collect()
}
