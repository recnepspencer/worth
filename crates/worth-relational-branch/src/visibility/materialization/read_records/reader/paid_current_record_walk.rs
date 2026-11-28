//! Paid current-record walks consume Head views without opening history.
use super::super::read_work::record_memory;
use super::super::read_prices::{EXAMINE_SLOT, RESERVE_RECORD};
use super::projection_record_reader::ProjectionRecordReader;
use super::record_class::RecordClassOps;
use crate::execution::RelationalExecutionDenialCause as Stop;
use crate::identity::data::KindId;
use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::residency::RelationalRegionHeadView;
impl ProjectionRecordReader<'_> {
    pub(super) fn paid_current_records<C: RecordClassOps>(
        &self,
        head: Option<RelationalRegionHeadView<'_>>,
        registry: &RelationalSchemaRegistry,
        kind: Option<KindId>,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<C::Record>, Stop> {
        let Some(head) = head else {
            return Ok(Vec::new());
        };
        let mut records = Vec::new();
        let arena = C::head_arena(head);
        for slot in arena.live_slots() {
            charge(EXAMINE_SLOT).map_err(crate::execution::denial::kernel_stop)?;
            if kind.is_some_and(|kind| {
                arena
                    .get_slot(slot)
                    .is_none_or(|record| record.kind_id() != Some(kind))
            }) {
                continue;
            }
            charge(RESERVE_RECORD).map_err(crate::execution::denial::kernel_stop)?;
            let clone_bytes = C::quote_head(registry, head, slot)?;
            record_memory::prepare_push(memory, &mut records, clone_bytes)?;
            if let Some(record) = C::read_head(registry, head, slot) {
                C::count_materialization(self.instrumentation);
                record_memory::settle_clone(memory, clone_bytes, C::record_bytes(&record))?;
                records.push(record);
            } else {
                record_memory::release(memory, clone_bytes)?;
            }
        }
        C::order(&mut records);
        Ok(records)
    }
}
