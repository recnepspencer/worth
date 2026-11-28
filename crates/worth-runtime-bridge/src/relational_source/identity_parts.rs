use worth_relational::facade::identity::{EntityId, PartitionId, RelationId};
use worth_relational::facade::transactions::RecordRef;

use crate::input::envelope::{BridgeCommittedPatchItem, BridgeCommittedPatchTarget};
use crate::snapshot::{SnapshotReadContract, SnapshotReadRequest};

pub use crate::identity::truth::RelationalBridgeSnapshotIdentityParts;

const RELATIONAL_RECORD_ENTITY_PREFIX: &str = "relational-record:entity:";
const RELATIONAL_RECORD_RELATION_PREFIX: &str = "relational-record:relation:";

/// Whether a Relational record is an entity or a relation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelationalBridgeRecordIdentityKind {
    /// An entity record.
    Entity,
    /// A relation record.
    Relation,
}

/// Exact native record coordinates carried by Bridge correspondence and reads.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelationalBridgeRecordIdentityParts(RecordCoordinate);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum RecordCoordinate {
    Entity(EntityId),
    Relation(RelationId),
}

impl RelationalBridgeRecordIdentityParts {
    pub const fn from_entity(entity: EntityId) -> Self {
        Self(RecordCoordinate::Entity(entity))
    }

    pub const fn from_relation(relation: RelationId) -> Self {
        Self(RecordCoordinate::Relation(relation))
    }

    pub const fn entity(partition_id: u32, local_slot: u64, generation: u32) -> Self {
        Self::from_entity(EntityId::new(
            PartitionId(partition_id),
            local_slot,
            generation,
        ))
    }

    pub const fn relation(partition_id: u32, local_slot: u64, generation: u32) -> Self {
        Self::from_relation(RelationId::new(
            PartitionId(partition_id),
            local_slot,
            generation,
        ))
    }

    pub const fn new(
        kind: RelationalBridgeRecordIdentityKind,
        partition: u32,
        slot: u64,
        generation: u32,
    ) -> Self {
        match kind {
            RelationalBridgeRecordIdentityKind::Entity => Self::entity(partition, slot, generation),
            RelationalBridgeRecordIdentityKind::Relation => {
                Self::relation(partition, slot, generation)
            }
        }
    }

    pub const fn kind(self) -> RelationalBridgeRecordIdentityKind {
        match self.0 {
            RecordCoordinate::Entity(_) => RelationalBridgeRecordIdentityKind::Entity,
            RecordCoordinate::Relation(_) => RelationalBridgeRecordIdentityKind::Relation,
        }
    }

    pub const fn partition_id(self) -> u32 {
        match self.0 {
            RecordCoordinate::Entity(id) => id.partition_id.0,
            RecordCoordinate::Relation(id) => id.partition_id.0,
        }
    }

    pub const fn local_slot(self) -> u64 {
        match self.0 {
            RecordCoordinate::Entity(id) => id.local_slot.0,
            RecordCoordinate::Relation(id) => id.local_slot.0,
        }
    }

    pub const fn generation(self) -> u32 {
        match self.0 {
            RecordCoordinate::Entity(id) => id.generation.0,
            RecordCoordinate::Relation(id) => id.generation.0,
        }
    }

    pub const fn record_ref(self) -> RecordRef {
        match self.0 {
            RecordCoordinate::Entity(id) => RecordRef::Entity(id),
            RecordCoordinate::Relation(id) => RecordRef::Relation(id),
        }
    }

    pub(crate) fn bridge_entity_identity(self) -> String {
        let prefix = match self.0 {
            RecordCoordinate::Entity(_) => RELATIONAL_RECORD_ENTITY_PREFIX,
            RecordCoordinate::Relation(_) => RELATIONAL_RECORD_RELATION_PREFIX,
        };
        format!(
            "{prefix}{}:{}:{}",
            self.partition_id(),
            self.local_slot(),
            self.generation(),
        )
    }

    /// The Bridge record identity string, for reports and diagnostics.
    pub fn terminal_projection_for_reporting(self) -> String {
        self.bridge_entity_identity()
    }
}

impl From<&RecordRef> for RelationalBridgeRecordIdentityParts {
    fn from(record: &RecordRef) -> Self {
        match *record {
            RecordRef::Entity(entity) => Self::from_entity(entity),
            RecordRef::Relation(relation) => Self::from_relation(relation),
        }
    }
}

impl std::fmt::Debug for RelationalBridgeRecordIdentityParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RelationalBridgeRecordIdentityParts")
            .field(&"<opaque>")
            .finish()
    }
}

impl BridgeCommittedPatchItem {
    /// A committed-patch item for a Relational record, with no semantic change
    /// attached.
    pub fn with_relational_record_target(
        record_identity: RelationalBridgeRecordIdentityParts,
        target: BridgeCommittedPatchTarget,
    ) -> Self {
        Self::from_relational_record_parts(record_identity, target, None)
    }

    /// A committed-patch item for a Relational record, carrying the semantic
    /// aspect change Relational reported for it.
    pub(crate) fn with_relational_semantic_change(
        record_identity: RelationalBridgeRecordIdentityParts,
        target: BridgeCommittedPatchTarget,
        semantic_change: crate::input::envelope::BridgeSemanticAspectChange,
    ) -> Self {
        Self::from_relational_record_parts(record_identity, target, Some(semantic_change))
    }
}

impl SnapshotReadRequest {
    /// A snapshot read request for one Relational record.
    pub fn for_relational_record(
        record_identity: RelationalBridgeRecordIdentityParts,
        target: SnapshotReadContract,
    ) -> Self {
        Self::for_record(
            crate::identity::record::BridgeRecordIdentity::Relational(record_identity),
            target,
        )
    }
}

pub(crate) fn fixture_position(label: impl Into<String>) -> u64 {
    let label = label.into();
    if let Some(position) = fixture_suffix_position(&label) {
        return position;
    }
    label.bytes().fold(17_u64, |acc, byte| {
        acc.wrapping_mul(131).wrapping_add(u64::from(byte))
    })
}

fn fixture_suffix_position(label: &str) -> Option<u64> {
    if label.split(['-', ':']).count() != 2 {
        return None;
    }
    let suffix = label.rsplit(['-', ':']).next()?;
    match suffix {
        "a" => Some(1),
        "b" => Some(2),
        "c" => Some(3),
        "d" => Some(4),
        "e" => Some(5),
        "f" => Some(6),
        _ => suffix.parse::<u64>().ok(),
    }
}
#[cfg(test)]
#[path = "identity_parts_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
