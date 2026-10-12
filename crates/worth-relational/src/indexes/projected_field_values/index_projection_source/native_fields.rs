//! Exact field indexes borrow native values; historical projections keep their read backing.
use super::*;
use crate::storage::overlay::PartitionAccess;
use crate::visibility::materialization::read_records::{
    authoritative_state_query_locus_value, entity_query_locus_value, relation_query_locus_value,
};
use worth_foundational::facade::{AspectFieldLocator, AspectValue};

impl IndexProjectionSource<'_> {
    pub(in crate::indexes) fn try_for_each_entity_field<E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind: KindId,
        field: &AspectFieldLocator,
        mut before: impl FnMut(u64) -> Result<(), E>,
        mut visit: impl FnMut(EntityId, Option<&AspectValue>) -> Result<(), E>,
    ) -> Result<(), E> {
        if !self.projection.is_exact_basis() {
            return self.try_for_each_entity(execution, kind, before, |record| {
                visit(record.entity_id, entity_query_locus_value(record, field))
            });
        }
        let Some(root) = self.projection.selected_root() else {
            return Ok(());
        };
        root.try_for_each_partition_id(|partition_id| {
            let Some(partition) = root.get_partition(partition_id) else {
                return Ok(());
            };
            for slot in partition.entity_arena.live_bitset.set_slots() {
                // Preserve each candidate's checkpoint. The pinned native value
                // needs no read-record allocation, including unrelated kinds.
                before(0)?;
                let Some(record) = partition.entity_arena.get_slot(slot) else {
                    continue;
                };
                if record.kind_id() != Some(kind) {
                    continue;
                }
                visit(
                    EntityId::new(partition_id, slot as u64, record.generation()),
                    authoritative_state_query_locus_value(
                        record.extra().authoritative_aspect_state.as_ref(),
                        field,
                    ),
                )?;
            }
            Ok(())
        })
    }

    pub(in crate::indexes) fn try_for_each_relation_field<E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind: KindId,
        field: &AspectFieldLocator,
        mut before: impl FnMut(u64) -> Result<(), E>,
        mut visit: impl FnMut(RelationId, Option<&AspectValue>) -> Result<(), E>,
    ) -> Result<(), E> {
        if !self.projection.is_exact_basis() {
            return self.try_for_each_relation(execution, kind, before, |record| {
                visit(
                    record.relation_id,
                    relation_query_locus_value(record, field),
                )
            });
        }
        let Some(root) = self.projection.selected_root() else {
            return Ok(());
        };
        root.try_for_each_partition_id(|partition_id| {
            let Some(partition) = root.get_partition(partition_id) else {
                return Ok(());
            };
            for slot in partition.relation_arena.live_bitset.set_slots() {
                before(0)?;
                let Some(record) = partition.relation_arena.get_slot(slot) else {
                    continue;
                };
                if record.kind_id() != Some(kind) {
                    continue;
                }
                visit(
                    RelationId::new(partition_id, slot as u64, record.generation()),
                    authoritative_state_query_locus_value(
                        record.extra().authoritative_aspect_state.as_ref(),
                        field,
                    ),
                )?;
            }
            Ok(())
        })
    }
}
