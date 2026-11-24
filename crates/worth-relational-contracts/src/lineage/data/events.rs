use serde::{Deserialize, Serialize};

use crate::history::data::commit_receipt::RelationalCommitReceipt;
use crate::history::data::coordinates::BranchId;
use crate::identity::data::{EntityId, LineageId, RelationId};
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
    /// The exact entity an allocating event binds when that entity was not
    /// created by its commit: a history selection restores a record at its
    /// original identity onto a fresh lineage, which no `Created` patch
    /// names. Absent on every other event, so their bytes never change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) entity_id: Option<EntityId>,
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

    pub fn entity_id(&self) -> Option<EntityId> {
        self.entity_id
    }
}

/// Operations used by the Relational owners.
pub trait LineageSubjectOwnerPort: Sized {
    fn is_entity(&self) -> bool;
}

impl LineageSubjectOwnerPort for LineageSubject {
    fn is_entity(&self) -> bool {
        matches!(self, Self::Entity)
    }
}

pub trait LineageEventRecordFieldsPort {
    fn event_id_mut_for_test(&mut self) -> &mut u64;
    fn lineage_event_branch_mut_for_test(&mut self) -> &mut BranchId;
    fn set_event_kind_for_test(&mut self, kind: LineageEventKind);
    fn owner_sources(&self) -> &Vec<LineageId>;
    fn owner_targets(&self) -> &Vec<LineageId>;
    fn owner_event_id(&self) -> &u64;
    fn owner_commit_mut(&mut self) -> &mut RelationalCommitReceipt;
    fn owner_commit(&self) -> &RelationalCommitReceipt;
    fn owner_entity_id_mut(&mut self) -> &mut Option<EntityId>;
    fn owner_kept_name_mut(&mut self) -> &mut Option<PersistentNameIdentity>;
    fn owner_name_birth_mut(&mut self) -> &mut Option<PersistentNameBirthRecord>;
    fn owner_sources_mut(&mut self) -> &mut Vec<LineageId>;
    fn owner_subject_mut(&mut self) -> &mut LineageSubject;
    fn owner_targets_mut(&mut self) -> &mut Vec<LineageId>;
}

impl LineageEventRecordFieldsPort for LineageEventRecord {
    fn event_id_mut_for_test(&mut self) -> &mut u64 {
        &mut self.event_id
    }
    fn lineage_event_branch_mut_for_test(&mut self) -> &mut BranchId {
        &mut self.branch_id
    }
    fn set_event_kind_for_test(&mut self, kind: LineageEventKind) {
        self.kind = kind;
    }
    fn owner_sources(&self) -> &Vec<LineageId> {
        &self.sources
    }
    fn owner_targets(&self) -> &Vec<LineageId> {
        &self.targets
    }
    fn owner_event_id(&self) -> &u64 {
        &self.event_id
    }
    fn owner_commit_mut(&mut self) -> &mut RelationalCommitReceipt {
        &mut self.commit
    }
    fn owner_commit(&self) -> &RelationalCommitReceipt {
        &self.commit
    }
    fn owner_entity_id_mut(&mut self) -> &mut Option<EntityId> {
        &mut self.entity_id
    }
    fn owner_kept_name_mut(&mut self) -> &mut Option<PersistentNameIdentity> {
        &mut self.kept_name
    }
    fn owner_name_birth_mut(&mut self) -> &mut Option<PersistentNameBirthRecord> {
        &mut self.name_birth
    }
    fn owner_sources_mut(&mut self) -> &mut Vec<LineageId> {
        &mut self.sources
    }
    fn owner_subject_mut(&mut self) -> &mut LineageSubject {
        &mut self.subject
    }
    fn owner_targets_mut(&mut self) -> &mut Vec<LineageId> {
        &mut self.targets
    }
}

pub struct LineageEventRecordParts {
    pub event_id: u64,
    pub commit: RelationalCommitReceipt,
    pub branch_id: BranchId,
    pub kind: LineageEventKind,
    pub sources: Vec<LineageId>,
    pub targets: Vec<LineageId>,
    pub subject: LineageSubject,
    pub name_birth: Option<PersistentNameBirthRecord>,
    pub kept_name: Option<PersistentNameIdentity>,
    pub relation_id: Option<RelationId>,
    pub entity_id: Option<EntityId>,
}

pub fn assemble_lineage_event_record(parts: LineageEventRecordParts) -> LineageEventRecord {
    LineageEventRecord {
        event_id: parts.event_id,
        commit: parts.commit,
        branch_id: parts.branch_id,
        kind: parts.kind,
        sources: parts.sources,
        targets: parts.targets,
        subject: parts.subject,
        name_birth: parts.name_birth,
        kept_name: parts.kept_name,
        relation_id: parts.relation_id,
        entity_id: parts.entity_id,
    }
}
