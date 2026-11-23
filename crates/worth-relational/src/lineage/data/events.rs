use serde::{Deserialize, Serialize};

use crate::history::data::{BranchId, RelationalCommitReceipt};
use crate::identity::data::{LineageId, RelationId};
use crate::lineage::data::{PersistentNameBirthRecord, PersistentNameIdentity};

/// Subject class whose continuity a [`LineageEventRecord`] records.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub enum LineageSubject {
    #[default]
    Entity,
    Relation,
}

impl LineageSubject {
    pub(crate) fn is_entity(&self) -> bool {
        matches!(self, Self::Entity)
    }
}

/// What a committed lineage event did to entity or relation lineages, as
/// recorded in a [`LineageEventRecord`].
///
/// Lineage resolution follows `Replace`, `Split`, `Merge`, and `Restore`
/// events from their sources to their targets; `Create` and `Retire` start or
/// end a lineage and are not followed.
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
    /// Undo, redo, history checkout, or authored restore re-activating a
    /// previously retired persistent name onto a fresh generational
    /// `RecordId` (`EntityId` or `RelationId`) and fresh `LineageId`.
    Restore,
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
    #[serde(default, skip_serializing_if = "LineageSubject::is_entity")]
    pub(crate) subject: LineageSubject,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "super::event_serde::verified_birth"
    )]
    pub(crate) name_birth: Option<PersistentNameBirthRecord>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "super::event_serde::persisted_kept_name"
    )]
    pub(crate) kept_name: Option<PersistentNameIdentity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) relation_id: Option<RelationId>,
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

    pub fn subject(&self) -> LineageSubject {
        self.subject
    }

    pub fn name_birth(&self) -> Option<&PersistentNameBirthRecord> {
        self.name_birth.as_ref()
    }

    pub fn kept_name(&self) -> Option<PersistentNameIdentity> {
        self.kept_name
    }

    pub fn relation_id(&self) -> Option<RelationId> {
        self.relation_id
    }
}
