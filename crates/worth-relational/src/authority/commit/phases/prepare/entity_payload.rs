//! Entity payload custody required by the lowered transaction's exact footprint.
use crate::storage::overlay::PartitionCloneMode;
use crate::transactions::data::{EntityMutationIntent, MergedCommitPlan, MutationIntent};
use std::collections::{BTreeMap, BTreeSet};

const AOSOA_SPARSE_ENTITY_SLOT_LIMIT: usize = 1024;
const AOSOA_SPARSE_PARTITION_LIMIT: usize = 8;

pub(super) enum EntityOverlayPayload {
    ShapeOnly,
    Selected(BTreeMap<crate::identity::data::PartitionId, BTreeSet<usize>>),
    Complete,
}

impl EntityOverlayPayload {
    pub(super) fn selected_slots(
        &self,
    ) -> Option<&BTreeMap<crate::identity::data::PartitionId, BTreeSet<usize>>> {
        match self {
            Self::Selected(slots) => Some(slots),
            Self::ShapeOnly | Self::Complete => None,
        }
    }

    pub(super) fn requires_complete_payload(&self) -> bool {
        matches!(self, Self::Complete)
    }
}

pub(super) fn sparse_entity_slots_for_plan(
    clone_mode: PartitionCloneMode,
    merged_plan: &MergedCommitPlan,
    footprint: Option<&crate::mvcc::RelationalTransactionFootprint>,
) -> EntityOverlayPayload {
    if !matches!(
        clone_mode,
        PartitionCloneMode::EntityOnly | PartitionCloneMode::GraphSparseEntities
    ) {
        return EntityOverlayPayload::Complete;
    }

    let mut slots_by_partition = BTreeMap::new();
    for intent in &merged_plan.merged_intents {
        match intent {
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(spec)) => {
                slots_by_partition
                    .entry(spec.entity_id.partition_id)
                    .or_insert_with(BTreeSet::new)
                    .insert(spec.entity_id.slot_index());
            }
            // A revalidation demand writes no field, but it marks the
            // record's slot touched, and a touched slot must be materialized
            // in the working state. Naming its slot here keeps the sparse
            // clone both correct and narrow: without this arm the plan falls
            // back to cloning every slot in the partition to cover one record
            // the demand never changed.
            MutationIntent::Entity(EntityMutationIntent::Revalidate(spec)) => {
                slots_by_partition
                    .entry(spec.entity_id.partition_id)
                    .or_insert_with(BTreeSet::new)
                    .insert(spec.entity_id.slot_index());
            }
            MutationIntent::Create(_)
                if matches!(clone_mode, PartitionCloneMode::GraphSparseEntities) => {}
            _ => return EntityOverlayPayload::Complete,
        }
    }

    if let Some(footprint) = footprint {
        for read in footprint.reads() {
            if let crate::mvcc::RelationalTransactionReadLocus::Existing(
                crate::transactions::data::RecordRef::Entity(entity),
            ) = read
            {
                slots_by_partition
                    .entry(entity.partition_id)
                    .or_insert_with(BTreeSet::new)
                    .insert(entity.slot_index());
            }
        }
    }

    let total_slots: usize = slots_by_partition.values().map(BTreeSet::len).sum();
    if total_slots == 0 {
        EntityOverlayPayload::ShapeOnly
    } else if total_slots <= AOSOA_SPARSE_ENTITY_SLOT_LIMIT
        && slots_by_partition.len() <= AOSOA_SPARSE_PARTITION_LIMIT
    {
        EntityOverlayPayload::Selected(slots_by_partition)
    } else {
        EntityOverlayPayload::Complete
    }
}
