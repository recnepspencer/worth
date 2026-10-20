//! Bounded enumeration of the live entities of one kind.
//!
//! A kind scan is the one truth read whose cost is set by how much state
//! already exists rather than by how much the caller named. A caller that must
//! visit every entity of a kind — validating existing state against a rule that
//! is only now starting to govern it, for instance — therefore has to be able
//! to declare in advance how much of that it is willing to pay for, and be
//! refused with a typed answer instead of a stall when the branch holds more.
//!
//! The bound is applied while the arena is walked, never after materializing
//! the answer, so an over-large scope costs the declared budget and stops.

use super::super::native_scan_stop::NativeScanStop;
use super::*;

/// The live entities of one kind, read under a declared work bound.
#[derive(Debug)]
pub struct BoundedEntityKindTruthRead {
    records: Vec<EntityReadRecord>,
    entity_slots_examined: usize,
}

/// A kind scan that reached its declared bound before it had seen the whole
/// kind. The counts describe what the refused read had already spent, so a
/// caller can report the scope it could not afford rather than a bare refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntityKindTruthReadLimitExceeded {
    entity_slots_examined: usize,
    entity_records_reserved: usize,
}

impl<'runtime> VisibilityReadContext<'runtime> {
    /// Reads the live entities of one kind across every partition, spending at
    /// most `maximum_work_units` units of work.
    ///
    /// One unit is charged for each occupied entity slot examined — including a
    /// slot skipped for belonging to another kind, which the scan still had to
    /// look at — and one further unit for each record materialized into the
    /// answer. The bound is tested before each unit is spent, so a maximum of
    /// zero refuses without examining anything.
    pub fn bounded_visible_entities_of_kind(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
        maximum_work_units: usize,
    ) -> Result<
        Result<BoundedEntityKindTruthRead, EntityKindTruthReadLimitExceeded>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        crate::visibility::materialization::read_records::read_work::scan_with_memory(
            execution,
            |context, memory| {
                let state = self.runtime.storage_access().current_edition();
                ProjectionRecordReader::new(
                    self.runtime.current_version_id(),
                    &self.runtime.services.instrumentation,
                )
                .bounded_entities_of_kind_in_state(
                    &state,
                    &self.runtime.config.schema.registry,
                    kind_id,
                    KindScanVisibility::for_version(self.runtime, version_id),
                    maximum_work_units,
                    &mut |units| context.checkpoint(units),
                    memory,
                )
            },
            |result| super::super::read_footprint::entities(&result.records),
        )
    }
}

/// The running cost of one kind scan and the records it has reserved so far.
pub(super) struct EntityKindScan<'charge, 'memory> {
    memory: Option<&'charge mut worth_execution::ExecutionResultMemory<'memory>>,
    charge: Option<&'charge mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>>,
    records: Vec<EntityReadRecord>,
    entity_slots_examined: usize,
    work_units: usize,
    maximum_work_units: Option<usize>,
}

impl<'charge, 'memory> EntityKindScan<'charge, 'memory> {
    pub(super) fn funded(
        maximum_work_units: usize,
        charge: &'charge mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: Option<&'charge mut worth_execution::ExecutionResultMemory<'memory>>,
    ) -> Self {
        Self {
            charge: Some(charge),
            memory,
            records: Vec::new(),
            entity_slots_examined: 0,
            work_units: 0,
            maximum_work_units: Some(maximum_work_units),
        }
    }

    fn examine_slot(&mut self) -> Result<(), NativeScanStop<EntityKindTruthReadLimitExceeded>> {
        self.spend(super::super::read_prices::EXAMINE_SLOT)?;
        self.entity_slots_examined += 1;
        Ok(())
    }

    fn reserve_with(
        &mut self,
        clone_bytes: u64,
        materialize: impl FnOnce() -> EntityReadRecord,
    ) -> Result<(), NativeScanStop<EntityKindTruthReadLimitExceeded>> {
        self.spend(super::super::read_prices::RESERVE_RECORD)?;
        if let Some(memory) = self.memory.as_mut() {
            super::super::read_work::record_memory::prepare_push(
                memory,
                &mut self.records,
                clone_bytes,
            )
            .map_err(NativeScanStop::Request)?;
        }
        let record = materialize();
        if let Some(memory) = self.memory.as_mut() {
            super::super::read_work::record_memory::settle_clone(
                memory,
                clone_bytes,
                super::super::read_footprint::entity(&record),
            )
            .map_err(NativeScanStop::Request)?;
        }
        self.records.push(record);
        Ok(())
    }

    fn spend(
        &mut self,
        units: u64,
    ) -> Result<(), NativeScanStop<EntityKindTruthReadLimitExceeded>> {
        if self.maximum_work_units == Some(self.work_units) {
            return Err(NativeScanStop::Limit(EntityKindTruthReadLimitExceeded {
                entity_slots_examined: self.entity_slots_examined,
                entity_records_reserved: self.records.len(),
            }));
        }
        if let Some(charge) = self.charge.as_mut() {
            charge(units).map_err(NativeScanStop::request)?;
        }
        self.work_units += 1;
        Ok(())
    }

    pub(super) fn finish(self) -> BoundedEntityKindTruthRead {
        BoundedEntityKindTruthRead {
            records: self.records,
            entity_slots_examined: self.entity_slots_examined,
        }
    }
}

impl BoundedEntityKindTruthRead {
    pub(crate) fn owned_bytes(&self) -> u64 {
        super::super::read_footprint::entities(&self.records)
    }

    pub const fn entity_slots_examined(&self) -> usize {
        self.entity_slots_examined
    }

    pub const fn entity_records_reserved(&self) -> usize {
        self.records.len()
    }

    pub const fn work_units(&self) -> usize {
        self.entity_slots_examined + self.records.len()
    }

    pub fn records(&self) -> &[EntityReadRecord] {
        &self.records
    }

    pub fn into_records(self) -> Vec<EntityReadRecord> {
        self.records
    }
}

impl EntityKindTruthReadLimitExceeded {
    pub const fn entity_slots_examined(self) -> usize {
        self.entity_slots_examined
    }

    pub const fn entity_records_reserved(self) -> usize {
        self.entity_records_reserved
    }

    pub const fn consumed_work_units(self) -> usize {
        self.entity_slots_examined + self.entity_records_reserved
    }
}

impl ProjectionRecordReader<'_> {
    pub(in crate::visibility::materialization::read_records) fn bounded_entities_of_kind_in_state(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        kind_id: crate::identity::data::KindId,
        visibility: KindScanVisibility,
        maximum_work_units: usize,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<BoundedEntityKindTruthRead, NativeScanStop<EntityKindTruthReadLimitExceeded>> {
        let mut scan = EntityKindScan::funded(maximum_work_units, charge, Some(memory));
        let (partitions, partition_bytes) = super::super::read_work::record_memory::partition_ids(
            state,
            scan.memory
                .as_mut()
                .expect("native scan keeps its original loan"),
        )
        .map_err(NativeScanStop::Request)?;
        for partition_id in partitions {
            self.scan_entity_kind_in_partition(
                state,
                registry,
                partition_id,
                kind_id,
                visibility,
                &mut scan,
            )?;
        }
        super::super::read_work::record_memory::release(
            scan.memory
                .as_mut()
                .expect("native scan keeps its original loan"),
            partition_bytes,
        )
        .map_err(NativeScanStop::Request)?;
        let read = scan.finish();
        debug_assert!(
            super::truth_access::authoritative_entity_records_are_canonical(read.records())
        );
        Ok(read)
    }

    pub(super) fn scan_entity_kind_in_partition(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        partition_id: crate::identity::data::PartitionId,
        kind_id: crate::identity::data::KindId,
        visibility: KindScanVisibility,
        scan: &mut EntityKindScan<'_, '_>,
    ) -> Result<(), NativeScanStop<EntityKindTruthReadLimitExceeded>> {
        let current_version = self.current_version;
        let Some(partition) = state.get_partition(partition_id) else {
            return Ok(());
        };
        match visibility {
            KindScanVisibility::Live => {
                let (slots, held) = super::super::read_work::record_memory::slots(
                    &partition.entity_arena.live_bitset,
                    scan.memory
                        .as_mut()
                        .expect("native bounded scans carry the initiating result loan"),
                )
                .map_err(NativeScanStop::Request)?;
                for slot in slots {
                    scan.examine_slot()?;
                    if !slot_kind_matches_current(&partition.entity_arena, slot, kind_id) {
                        continue;
                    }
                    scan.reserve_with(
                        super::super::read_work::record_quote::entity(
                            registry, partition, slot, visibility,
                        )
                        .map_err(NativeScanStop::Request)?,
                        || {
                            self.instrumentation.count(|counters| {
                                counters.visible_authoritative_entity_records_materialized += 1;
                            });
                            materialize_current_authoritative_entity_record(
                                registry,
                                partition,
                                partition_id,
                                slot,
                            )
                            .expect("a matching live entity slot must materialize")
                        },
                    )?;
                }
                super::super::read_work::record_memory::release(
                    scan.memory
                        .as_mut()
                        .expect("the original loan remains present"),
                    held,
                )
                .map_err(NativeScanStop::Request)?;
            }
            KindScanVisibility::AtVersion(version_id) => {
                let (slots, held) = super::super::read_work::record_memory::occupied_slots(
                    &partition.entity_arena,
                    scan.memory
                        .as_mut()
                        .expect("native bounded scans carry the initiating result loan"),
                )
                .map_err(NativeScanStop::Request)?;
                for slot in slots {
                    scan.examine_slot()?;
                    self.instrumentation.count(|counters| {
                        counters.visibility_entity_slot_scans += 1;
                    });
                    if !entity_slot_matches_kind_at_version(
                        partition,
                        slot,
                        kind_id,
                        version_id,
                        current_version,
                    ) {
                        continue;
                    }
                    scan.reserve_with(
                        super::super::read_work::record_quote::entity(
                            registry, partition, slot, visibility,
                        )
                        .map_err(NativeScanStop::Request)?,
                        || {
                            self.instrumentation.count(|counters| {
                                counters.visible_authoritative_entity_records_materialized += 1;
                            });
                            materialize_authoritative_entity_record_at_version(
                                registry,
                                partition,
                                partition_id,
                                slot,
                                version_id,
                            )
                            .expect("a matching historical entity slot must materialize")
                        },
                    )?;
                }
                super::super::read_work::record_memory::release(
                    scan.memory
                        .as_mut()
                        .expect("the original loan remains present"),
                    held,
                )
                .map_err(NativeScanStop::Request)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod paid_stop_tests {
    use super::*;
    #[test]
    fn entity_scan_keeps_the_request_charge_failure() {
        for stop in [
            worth_execution::MapKernelStop::Cancelled,
            worth_execution::MapKernelStop::DeadlineElapsed,
            worth_execution::MapKernelStop::WorkCeiling,
        ] {
            let mut charge = |_| Err(stop);
            let mut scan = EntityKindScan::funded(8, &mut charge, None);
            assert_eq!(
                scan.examine_slot(),
                Err(NativeScanStop::Request(
                    crate::execution::denial::kernel_stop(stop)
                ))
            );
            assert_eq!(scan.finish().entity_records_reserved(), 0);
        }
    }
}
