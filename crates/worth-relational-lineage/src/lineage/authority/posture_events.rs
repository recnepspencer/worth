//! One validated posture's canonical lineage event before its id is assigned.

use crate::contracts::history::data::commit_receipt::RelationalCommitReceipt;
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::lineage::data::artifacts::decision_records::LineageDecisionKind;
use crate::contracts::lineage::data::events::{
    assemble_lineage_event_record, LineageEventKind, LineageEventRecord, LineageEventRecordParts,
    LineageSubject,
};
use crate::contracts::lineage::data::naming::{PersistentNameBirthRecord, PersistentNameIdentity};

pub(super) struct AssignedPostureEvent {
    pub(super) kind: LineageEventKind,
    pub(super) decision_kind: LineageDecisionKind,
    pub(super) subject: LineageSubject,
    pub(super) sources: Vec<LineageId>,
    pub(super) targets: Vec<LineageId>,
    pub(super) entity_node: Option<(EntityId, LineageId, bool)>,
    pub(super) relation_lineage: Option<(RelationId, LineageId)>,
    pub(super) name_birth: Option<PersistentNameBirthRecord>,
    pub(super) kept_name: Option<PersistentNameIdentity>,
}

impl AssignedPostureEvent {
    /// The canonical record this event commits as `event_id`; `entity_id`
    /// names an entity the commit did not create (a selection restore).
    pub(super) fn into_record(
        self,
        event_id: u64,
        commit: &RelationalCommitReceipt,
        entity_id: Option<EntityId>,
    ) -> LineageEventRecord {
        assemble_lineage_event_record(LineageEventRecordParts {
            event_id,
            commit: commit.clone(),
            branch_id: commit.branch_id.clone(),
            kind: self.kind,
            sources: self.sources,
            targets: self.targets,
            subject: self.subject,
            name_birth: self.name_birth,
            kept_name: self.kept_name,
            relation_id: self.relation_lineage.map(|(relation_id, _)| relation_id),
            entity_id,
        })
    }
}
