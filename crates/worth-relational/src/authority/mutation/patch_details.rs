use crate::identity::data::{EntityId, RelationId};
use crate::publication::patch::data::PatchDetail;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EntityPatchDetailKind {
    Created,
    Updated,
    Deleted,
    MaterializationSuspended,
    Rematerialized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RelationPatchDetailKind {
    Created,
    Updated,
    Deleted,
    RetainedForAudit,
    MaterializationSuspended,
    Rematerialized,
}

pub(super) fn patch_detail_for_entity(
    kind: EntityPatchDetailKind,
    entity_id: EntityId,
) -> PatchDetail {
    PatchDetail::DenseBitset(vec![
        entity_patch_kind_code(kind),
        entity_id.partition_value_u64(),
        entity_id.local_slot_value(),
        entity_id.generation_value() as u64,
    ])
}

pub(super) fn patch_detail_for_relation(
    kind: RelationPatchDetailKind,
    relation_id: RelationId,
    source: EntityId,
    target: EntityId,
) -> PatchDetail {
    PatchDetail::DenseBitset(vec![
        relation_patch_kind_code(kind),
        relation_id.partition_value_u64(),
        relation_id.local_slot_value(),
        relation_id.generation_value() as u64,
        source.partition_value_u64(),
        source.local_slot_value(),
        target.partition_value_u64(),
        target.local_slot_value(),
    ])
}

fn entity_patch_kind_code(kind: EntityPatchDetailKind) -> u64 {
    match kind {
        EntityPatchDetailKind::Created => 1,
        EntityPatchDetailKind::Updated => 2,
        EntityPatchDetailKind::Deleted => 3,
        EntityPatchDetailKind::MaterializationSuspended => 8,
        EntityPatchDetailKind::Rematerialized => 9,
    }
}

fn relation_patch_kind_code(kind: RelationPatchDetailKind) -> u64 {
    match kind {
        RelationPatchDetailKind::Created => 4,
        RelationPatchDetailKind::Updated => 5,
        RelationPatchDetailKind::Deleted => 6,
        RelationPatchDetailKind::RetainedForAudit => 7,
        RelationPatchDetailKind::MaterializationSuspended => 10,
        RelationPatchDetailKind::Rematerialized => 11,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        patch_detail_for_entity, patch_detail_for_relation, EntityPatchDetailKind,
        RelationPatchDetailKind,
    };
    use crate::identity::data::{EntityId, PartitionId, RelationId};
    use crate::publication::patch::data::PatchDetail;

    #[test]
    fn dense_entity_patch_details_use_named_identity_accessors() {
        let entity_id = EntityId::new(PartitionId::new(3), 11, 7);

        let detail = patch_detail_for_entity(EntityPatchDetailKind::Updated, entity_id);

        assert_eq!(detail, PatchDetail::DenseBitset(vec![2, 3, 11, 7]),);
    }

    #[test]
    fn dense_relation_patch_details_use_named_identity_accessors() {
        let relation_id = RelationId::new(PartitionId::new(9), 5, 2);
        let source = EntityId::new(PartitionId::new(3), 11, 7);
        let target = EntityId::new(PartitionId::new(4), 12, 8);

        let detail = patch_detail_for_relation(
            RelationPatchDetailKind::Created,
            relation_id,
            source,
            target,
        );

        assert_eq!(
            detail,
            PatchDetail::DenseBitset(vec![4, 9, 5, 2, 3, 11, 4, 12]),
        );
    }
}
