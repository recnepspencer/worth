//! One validated posture's canonical lineage event before its id is assigned.

use crate::history::data::RelationalCommitReceipt;
use crate::identity::data::{EntityId, LineageId, RelationId};
use crate::lineage::data::{
    LineageDecisionKind, LineageEventKind, LineageEventRecord, LineageSubject,
    PersistentNameBirthRecord, PersistentNameIdentity,
};

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
        LineageEventRecord {
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
        }
    }
}
