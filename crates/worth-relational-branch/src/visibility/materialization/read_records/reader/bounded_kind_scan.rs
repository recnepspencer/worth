//! Meter bounded kind scans and preclaim materialized records on the caller's ticket.
use super::super::projection::exact_basis_reads::ExactProjectionReader;
use super::super::{
    read_prices::{EXAMINE_SLOT, RESERVE_RECORD},
    read_work::record_memory,
};
use super::{
    kind_scan_visibility::KindScanVisibility, projection_record_reader::ProjectionRecordReader,
    record_class::RecordClassOps,
};
use crate::execution::RelationalExecutionDenialCause;
use crate::identity::data::{KindId, PartitionId, RecordId};
use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::{
    data::RelationalStorageUnavailable, overlay::PartitionAccess, substrate::RecordKind,
};
use worth_execution::{ExecutionResultMemory, MapKernelStop};
use worth_relational_contracts::internal_port::execution::request_or::RequestOr;

pub(super) trait BoundedRecordClass: RecordClassOps {
    type Denial;
    const COUNT_MATERIALIZED: bool;

    fn limit(examined: usize, reserved: usize) -> Self::Denial;
    fn unreadable(partition_id: PartitionId, slot: usize) -> Self::Denial;
}

pub(super) struct KindScan<'charge, 'memory, C: BoundedRecordClass> {
    charge: Option<&'charge mut dyn FnMut(u64) -> Result<(), MapKernelStop>>,
    memory: Option<&'charge mut ExecutionResultMemory<'memory>>,
    maximum: Option<usize>,
    require_materialization: bool,
    slots_examined: usize,
    records_reserved: usize,
    work_units: usize,
    records: Vec<C::Record>,
}

impl<'charge, 'memory, C: BoundedRecordClass> KindScan<'charge, 'memory, C> {
    pub(super) fn new(
        maximum: Option<usize>,
        require_materialization: bool,
        charge: Option<&'charge mut dyn FnMut(u64) -> Result<(), MapKernelStop>>,
        memory: Option<&'charge mut ExecutionResultMemory<'memory>>,
    ) -> Self {
        Self {
            charge,
            memory,
            maximum,
            require_materialization,
            slots_examined: 0,
            records_reserved: 0,
            work_units: 0,
            records: Vec::new(),
        }
    }

    fn spend(&mut self, units: u64) -> Result<(), RequestOr<C::Denial>> {
        if self.maximum == Some(self.work_units) {
            return Err(RequestOr::Inner(C::limit(
                self.slots_examined,
                self.records_reserved,
            )));
        }
        if let Some(charge) = &mut self.charge {
            charge(units).map_err(RequestOr::from_kernel_stop)?;
        }
        self.work_units += 1;
        Ok(())
    }

    pub(super) fn examine_slot(&mut self) -> Result<(), RequestOr<C::Denial>> {
        self.spend(EXAMINE_SLOT)?;
        self.slots_examined += 1;
        Ok(())
    }

    pub(super) fn reserve_with(
        &mut self,
        partition_id: PartitionId,
        slot: usize,
        clone_bytes: u64,
        materialize: impl FnOnce() -> Option<C::Record>,
    ) -> Result<(), RequestOr<C::Denial>> {
        self.spend(RESERVE_RECORD)?;
        self.records_reserved += 1;
        if let Some(memory) = self.memory.as_mut() {
            record_memory::prepare_push(memory, &mut self.records, clone_bytes)
                .map_err(RequestOr::Request)?;
        }
        if let Some(record) = materialize() {
            if let Some(memory) = self.memory.as_mut() {
                record_memory::settle_clone(memory, clone_bytes, C::record_bytes(&record))
                    .map_err(RequestOr::Request)?;
            }
            self.records.push(record);
        } else if self.require_materialization {
            return Err(RequestOr::Inner(C::unreadable(partition_id, slot)));
        }
        Ok(())
    }

    pub(super) fn finish(mut self) -> (Vec<C::Record>, usize, usize) {
        C::order(&mut self.records);
        (self.records, self.slots_examined, self.records_reserved)
    }

    fn result_memory(&mut self) -> &mut ExecutionResultMemory<'memory> {
        self.memory
            .as_deref_mut()
            .expect("state scan retains its initiating result loan")
    }
}

impl ProjectionRecordReader<'_> {
    pub(super) fn scan_kind_from_directory<C: BoundedRecordClass>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &ExactProjectionReader<'_>,
        kind: KindId,
        ids: impl Iterator<
            Item = Result<RecordId<<C::Kind as RecordKind>::Domain>, RelationalStorageUnavailable>,
        >,
        scan: &mut KindScan<'_, '_, C>,
    ) -> Result<(), RequestOr<C::Denial>> {
        let registry = source.registry();
        // Release the previous borrow before demanding the next region so this
        // scan keeps at most one Head lease.
        let mut window = None;
        for id in ids {
            let id = id.map_err(|cause| {
                RequestOr::Request(RelationalExecutionDenialCause::StorageUnavailable(cause))
            })?;
            scan.examine_slot()?;
            let partition = id.partition_id;
            if window.as_ref().map(|(region, _)| *region) != Some(partition) {
                drop(window.take());
                let lease = source
                    .borrow_head(execution, partition)
                    .map_err(RequestOr::Request)?;
                window = Some((partition, lease));
            }
            let head = window
                .as_ref()
                .map(|(_, lease)| lease.head_view())
                .expect("the window holds the current partition");
            if C::head_arena(head)
                .get(&id)
                .is_none_or(|record| record.kind_id() != Some(kind))
            {
                return Err(RequestOr::Request(source.membership_failure(partition)));
            }
            let slot = id.slot_index();
            let quoted = C::quote_head(registry, head, slot).map_err(RequestOr::Request)?;
            scan.reserve_with(partition, slot, quoted, || {
                C::count_materialization(self.instrumentation);
                C::read_exact_head(registry, head, id)
            })?;
        }
        Ok(())
    }

    pub(super) fn scan_kind_in_state<C: BoundedRecordClass>(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &RelationalSchemaRegistry,
        kind: KindId,
        visibility: KindScanVisibility,
        scan: &mut KindScan<'_, '_, C>,
    ) -> Result<(), RequestOr<C::Denial>> {
        let (partitions, held) = record_memory::partition_ids(state, scan.result_memory())
            .map_err(RequestOr::Request)?;
        for partition in partitions {
            self.scan_kind_in_partition(state, registry, partition, kind, visibility, scan)?;
        }
        record_memory::release(scan.result_memory(), held).map_err(RequestOr::Request)
    }

    fn scan_kind_in_partition<C: BoundedRecordClass>(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &RelationalSchemaRegistry,
        partition_id: PartitionId,
        kind: KindId,
        visibility: KindScanVisibility,
        scan: &mut KindScan<'_, '_, C>,
    ) -> Result<(), RequestOr<C::Denial>> {
        let KindScanVisibility::AtVersion(version) = visibility else {
            let Some(head) = state.get_current_partition(partition_id) else {
                return Ok(());
            };
            let arena = C::head_arena(head);
            for slot in arena.live_slots() {
                scan.examine_slot()?;
                if arena
                    .get_slot(slot)
                    .is_none_or(|record| record.kind_id() != Some(kind))
                {
                    continue;
                }
                let quoted = C::quote_head(registry, head, slot).map_err(RequestOr::Request)?;
                scan.reserve_with(partition_id, slot, quoted, || {
                    if C::COUNT_MATERIALIZED {
                        C::count_materialization(self.instrumentation);
                    }
                    C::read_head(registry, head, slot)
                })?;
            }
            return Ok(());
        };

        let Some(partition) = state.get_partition(partition_id) else {
            return Ok(());
        };
        let (slots, held) =
            record_memory::occupied_slots(C::arena(partition), scan.result_memory())
                .map_err(RequestOr::Request)?;
        for slot in slots {
            scan.examine_slot()?;
            C::count_historical_slot(self.instrumentation);
            if !C::visible(partition, slot, Some(kind), version, self.current_version) {
                continue;
            }
            let quoted = C::quote_history(registry, partition, slot, visibility)
                .map_err(RequestOr::Request)?;
            scan.reserve_with(partition_id, slot, quoted, || {
                if C::COUNT_MATERIALIZED {
                    C::count_materialization(self.instrumentation);
                }
                C::read_history(registry, partition, partition_id, slot, version)
            })?;
        }
        record_memory::release(scan.result_memory(), held).map_err(RequestOr::Request)
    }
}
