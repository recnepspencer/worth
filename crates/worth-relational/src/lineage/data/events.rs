use serde::{Deserialize, Serialize};

use crate::history::data::{BranchId, RelationalCommitReceipt};
use crate::identity::data::LineageId;

/// What a committed lineage event did to entity lineages, as recorded in a
/// [`LineageEventRecord`].
///
/// Lineage resolution follows `Replace`, `Split`, and `Merge` events from
/// their sources to their targets; `Create` and `Retire` start or end a
/// lineage and are not followed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineageEventKind {
    /// A new lineage began; it has no sources and one target.
    Create,
    /// One lineage was continued by another lineage.
    Replace,
    /// One lineage was continued by several lineages.
    Split,
    /// Several lineages were continued by one lineage.
    Merge,
    /// A lineage ended; it has one source and no targets.
    Retire,
}

/// One lineage event published by a Relational commit: which commit and
/// branch produced it, what kind of event it was, and which lineages it
/// connects.
///
/// Lineage events are committed history. You read them from commit
/// inspection or a canonical commit envelope; they describe continuity and
/// grant nothing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageEventRecord {
    pub(crate) event_id: u64,
    pub(crate) commit: RelationalCommitReceipt,
    pub(crate) branch_id: BranchId,
    pub(crate) kind: LineageEventKind,
    pub(crate) sources: Vec<LineageId>,
    pub(crate) targets: Vec<LineageId>,
}

impl LineageEventRecord {
    pub fn event_id(&self) -> u64 {
        self.event_id
    }

    pub fn commit(&self) -> &RelationalCommitReceipt {
        &self.commit
    }

    pub fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub fn kind(&self) -> LineageEventKind {
        self.kind
    }

    pub fn sources(&self) -> &[LineageId] {
        &self.sources
    }

    pub fn targets(&self) -> &[LineageId] {
        &self.targets
    }
}
