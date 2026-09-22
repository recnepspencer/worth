use super::work::MaintenanceWork;
use crate::identity::data::{EntityId, RelationId};
use crate::indexes::data::DerivedIndexMaintenanceDenialKind as Denial;
use crate::runtime::VisibilityProjectionView;
use crate::transactions::data::RecordRef;
use std::collections::BTreeSet;

pub(super) struct ChangedRecords {
    pub(super) entities: BTreeSet<EntityId>,
    pub(super) relations: BTreeSet<RelationId>,
}

impl ChangedRecords {
    pub(super) fn patch(
        root: &crate::branch::RelationalBranchRoot,
        work: &mut MaintenanceWork,
    ) -> Result<Self, Denial> {
        let patches = &root
            .canonical_envelope()
            .ok_or(Denial::CommitMismatch)?
            .patch
            .authoritative_record_patches;
        work.charge(patches.len())?;
        work.counts.patch_records += patches.len();
        let mut changed = Self {
            entities: BTreeSet::new(),
            relations: BTreeSet::new(),
        };
        for patch in patches {
            match patch.target {
                RecordRef::Entity(id) => {
                    changed.entities.insert(id);
                }
                RecordRef::Relation(id) => {
                    changed.relations.insert(id);
                }
            }
        }
        Ok(changed)
    }

    pub(super) fn cold(
        view: &VisibilityProjectionView<'_>,
        work: &mut MaintenanceWork,
    ) -> Result<Self, Denial> {
        let root = view.selected_root().ok_or(Denial::CommitMismatch)?;
        let slots = root
            .entity_slot_count()
            .saturating_add(root.relation_slot_count())
            .saturating_add(root.region_count());
        if work.budget.maximum_cold_record_slots == 0
            || slots
                > work
                    .budget
                    .maximum_cold_record_slots
                    .saturating_sub(work.counts.cold_record_slots)
        {
            return Err(Denial::ColdReconstructionRequired);
        }
        work.charge(slots)?;
        work.counts.cold_record_slots += slots;
        let mut entities = BTreeSet::new();
        let mut relations = BTreeSet::new();
        for partition_id in root.partition_ids() {
            let Some(partition) = root.partition_state(partition_id) else {
                continue;
            };
            for slot in partition.entity_arena.live_bitset.iter_set_slots() {
                work.read()?;
                let Some(slot_view) = partition.entity_arena.get_slot(slot) else {
                    continue;
                };
                let id = EntityId::new(partition_id, slot as u64, slot_view.generation());
                if let Some(record) = view.authoritative_entity_record(id) {
                    entities.insert(record.entity_id);
                }
            }
            for slot in partition.relation_arena.live_bitset.iter_set_slots() {
                work.read()?;
                let Some(slot_view) = partition.relation_arena.get_slot(slot) else {
                    continue;
                };
                let id = RelationId::new(partition_id, slot as u64, slot_view.generation());
                if let Some(record) = view.authoritative_relation_record(id) {
                    relations.insert(record.relation_id);
                }
            }
        }
        Ok(Self {
            entities,
            relations,
        })
    }
}
