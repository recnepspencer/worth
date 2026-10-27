//! Paid current-record walks consume Head views without opening history.
use super::super::read_prices::{EXAMINE_SLOT, RESERVE_RECORD};
use super::*;
impl ProjectionRecordReader<'_> {
    pub(super) fn paid_current_entities(
        &self,
        head: Option<crate::storage::residency::RelationalRegionHeadView<'_>>,
        registry: &crate::schema::data::RelationalSchemaRegistry,
        kind: Option<crate::identity::data::KindId>,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        let Some(head) = head else {
            return Ok(Vec::new());
        };
        let mut records = Vec::new();

        for slot in head.entity_arena().live_slots() {
            charge(EXAMINE_SLOT).map_err(crate::execution::denial::kernel_stop)?;
            if kind.is_some_and(|kind| {
                head.entity_arena()
                    .get_slot(slot)
                    .is_none_or(|record| record.kind_id() != Some(kind))
            }) {
                continue;
            }
            charge(RESERVE_RECORD).map_err(crate::execution::denial::kernel_stop)?;
            let clone_bytes =
                super::super::read_work::record_quote::entity_head(registry, head, slot)?;
            super::super::read_work::record_memory::prepare_push(
                memory,
                &mut records,
                clone_bytes,
            )?;
            if let Some(record) = head.entity_record(
                registry,
                crate::identity::data::EntityId::new(head.partition_id(), slot as u64, 0),
            ) {
                self.instrumentation.count(|counters| {
                    counters.visible_authoritative_entity_records_materialized += 1;
                });
                super::super::read_work::record_memory::settle_clone(
                    memory,
                    clone_bytes,
                    super::super::read_footprint::entity(&record),
                )?;
                records.push(record);
            } else {
                super::super::read_work::record_memory::release(memory, clone_bytes)?;
            }
        }

        Ok(records)
    }
    pub(super) fn paid_current_relations(
        &self,
        head: Option<crate::storage::residency::RelationalRegionHeadView<'_>>,
        registry: &crate::schema::data::RelationalSchemaRegistry,
        kind: Option<crate::identity::data::KindId>,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<RelationReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        let Some(head) = head else {
            return Ok(Vec::new());
        };
        let mut records = Vec::new();

        for slot in head.relation_arena().live_slots() {
            charge(EXAMINE_SLOT).map_err(crate::execution::denial::kernel_stop)?;
            if kind.is_some_and(|kind| {
                head.relation_arena()
                    .get_slot(slot)
                    .is_none_or(|record| record.kind_id() != Some(kind))
            }) {
                continue;
            }
            charge(RESERVE_RECORD).map_err(crate::execution::denial::kernel_stop)?;
            let clone_bytes =
                super::super::read_work::record_quote::relation_head(registry, head, slot)?;
            super::super::read_work::record_memory::prepare_push(
                memory,
                &mut records,
                clone_bytes,
            )?;
            if let Some(record) = head.relation_record(
                registry,
                crate::identity::data::RelationId::new(head.partition_id(), slot as u64, 0),
            ) {
                self.instrumentation.count(|counters| {
                    counters.visible_authoritative_relation_records_materialized += 1;
                });
                super::super::read_work::record_memory::settle_clone(
                    memory,
                    clone_bytes,
                    super::super::read_footprint::relation(&record),
                )?;
                records.push(record);
            } else {
                super::super::read_work::record_memory::release(memory, clone_bytes)?;
            }
        }

        super::truth_record_access::sort_authoritative_relation_records(&mut records);
        Ok(records)
    }
}
