use super::work::MaintenanceWork;
use crate::identity::data::{EntityId, RelationId};
use crate::indexes::data::DerivedIndexMaintenanceDenialKind as Denial;
use crate::runtime::VisibilityProjectionView;
use crate::storage::data::RecordLifecycleState;
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
                    work.ordered::<EntityId, ()>(changed.entities.len(), 1, 0)?;
                    changed.entities.insert(id);
                }
                RecordRef::Relation(id) => {
                    work.ordered::<RelationId, ()>(changed.relations.len(), 1, 0)?;
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
        root.try_visit_partitions(|event| {
            let crate::branch::RelationalPartitionVisit::Partition(partition_id) = event else {
                return work.prepare(1, 0);
            };
            work.prepare(33, 0)?;
            let Some(partition) = root.partition_state(partition_id) else {
                return Ok(());
            };
            for slot in partition.entity_arena.live_bitset.iter_set_slots() {
                work.read()?;
                let Some(slot_view) = partition.entity_arena.get_slot(slot) else {
                    continue;
                };
                let id = EntityId::new(partition_id, slot as u64, slot_view.generation());
                work.lookup(root.schema_authority().registry().entity_kinds.len(), 1)?;
                if slot_view.lifecycle() == RecordLifecycleState::Live
                    && slot_view.kind_id().is_some_and(|kind| {
                        root.schema_authority()
                            .registry()
                            .entity_kinds
                            .contains_key(&kind)
                    })
                {
                    work.ordered::<EntityId, ()>(entities.len(), 1, 0)?;
                    entities.insert(id);
                }
            }
            for slot in partition.relation_arena.live_bitset.iter_set_slots() {
                work.read()?;
                let Some(slot_view) = partition.relation_arena.get_slot(slot) else {
                    continue;
                };
                let id = RelationId::new(partition_id, slot as u64, slot_view.generation());
                work.lookup(root.schema_authority().registry().relation_kinds.len(), 1)?;
                if matches!(
                    slot_view.lifecycle(),
                    RecordLifecycleState::Live | RecordLifecycleState::RetainedDanglingForAudit
                ) && slot_view.kind_id().is_some_and(|kind| {
                    root.schema_authority()
                        .registry()
                        .relation_kinds
                        .contains_key(&kind)
                }) && slot_view.extra().endpoints.is_some()
                {
                    work.ordered::<RelationId, ()>(relations.len(), 1, 0)?;
                    relations.insert(id);
                }
            }
            Ok(())
        })?;
        Ok(Self {
            entities,
            relations,
        })
    }
}
