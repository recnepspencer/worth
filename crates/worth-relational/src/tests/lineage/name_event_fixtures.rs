//! Canonical committed lineage events in exactly the shape Phase 2
//! finalization emits, for courts that need history at scales or in shapes
//! the real finalization driver does not cheaply reach.

use crate::facade::identity::{EntityId, LineageId, PartitionId};
use crate::facade::lineage::{
    LineageEventKind, LineageEventRecord, LineageSubject, PersistentNameIdentity,
    PersistentNameSubjectKind,
};
use crate::lineage::data::{PersistentNameBirthParts, PersistentNameBirthRecord};
use worth_relational_contracts::internal_port::lineage::data::events::assemble_lineage_event_record;
use worth_relational_contracts::internal_port::lineage::data::events::LineageEventRecordFieldsPort as _;
use worth_relational_contracts::internal_port::lineage::data::events::LineageEventRecordParts;
use worth_relational_contracts::internal_port::lineage::data::naming::PersistentNameBirthPartsOwnerPort as _;
use worth_relational_contracts::internal_port::lineage::data::naming::PersistentNameBirthRecordOwnerPort as _;
use worth_relational_contracts::internal_port::lineage::data::naming::PersistentNameIdentityOwnerPort as _;

use super::posture_fixtures::receipt_at;

/// A distinct minted-shape name: `tag` separates families, `ordinal` names.
pub(super) fn synthetic_name(tag: u8, ordinal: u64) -> PersistentNameIdentity {
    let mut bytes = [tag; 32];
    bytes[..8].copy_from_slice(&ordinal.to_be_bytes());
    PersistentNameIdentity::from_minted_digest(bytes)
}

/// A record on its own partition so it never aliases a driver's records.
pub(super) fn synthetic_entity(ordinal: u64) -> EntityId {
    EntityId::new(PartitionId::new(7), ordinal, 1)
}

fn event(
    commit: u64,
    event_id: u64,
    kind: LineageEventKind,
    sources: Vec<LineageId>,
    targets: Vec<LineageId>,
) -> LineageEventRecord {
    let receipt = receipt_at(commit, commit);
    assemble_lineage_event_record(LineageEventRecordParts {
        event_id,
        branch_id: receipt.branch_id.clone(),
        commit: receipt,
        kind,
        sources,
        targets,
        subject: LineageSubject::Entity,
        name_birth: None,
        kept_name: None,
        relation_id: None,
        entity_id: None,
    })
}

/// A named entity `Create` carrying its canonical birth record.
pub(super) fn named_create(
    commit: u64,
    event_id: u64,
    lineage: LineageId,
    name: PersistentNameIdentity,
) -> LineageEventRecord {
    let parts = PersistentNameBirthParts::new(
        "worth.cad.face.v1".to_owned(),
        format!("occ.synthetic.{event_id}"),
        "authored".to_owned(),
        Vec::new(),
        Vec::new(),
    );
    let mut create = event(
        commit,
        event_id,
        LineageEventKind::Create,
        vec![],
        vec![lineage],
    );
    (*create.owner_name_birth_mut()) = Some(PersistentNameBirthRecord::new(
        name,
        PersistentNameSubjectKind::Entity,
        parts,
    ));
    create
}

pub(super) fn retire(commit: u64, event_id: u64, lineage: LineageId) -> LineageEventRecord {
    event(
        commit,
        event_id,
        LineageEventKind::Retire,
        vec![lineage],
        vec![],
    )
}
