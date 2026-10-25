//! Bounded authoritative enumeration of live relations of one kind.

use super::super::native_scan_stop::NativeScanStop;
use super::*;

#[derive(Debug)]
pub struct BoundedRelationKindTruthRead {
    records: Vec<RelationReadRecord>,
    relation_slots_examined: usize,
    relation_records_reserved: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RelationKindTruthReadLimitExceeded {
    relation_slots_examined: usize,
    relation_records_reserved: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationKindTruthReadDenial {
    WorkLimitExceeded(RelationKindTruthReadLimitExceeded),
    UnreadableRelationSlot {
        partition_id: crate::identity::data::PartitionId,
        slot: usize,
    },
}

impl<'runtime> VisibilityReadContext<'runtime> {
    /// Scans one relation kind at a version, refusing before spending more
    /// than the declared slot-examination and record-materialization work.
    pub fn bounded_visible_relations_of_kind(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
        maximum_work_units: usize,
    ) -> Result<
        Result<BoundedRelationKindTruthRead, RelationKindTruthReadDenial>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        let edition = self.runtime.storage_access().current_edition();
        edition
            .demand_all(execution)
            .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
        crate::visibility::materialization::read_records::read_work::scan_with_memory(
            execution,
            |context, memory| {
                let state = edition.clone();
                ProjectionRecordReader::new(
                    self.runtime.current_version_id(),
                    &self.runtime.services.instrumentation,
                )
                .bounded_relations_of_kind_in_state(
                    &state,
                    &self.runtime.config.schema.registry,
                    kind_id,
                    KindScanVisibility::for_version(self.runtime, version_id),
                    maximum_work_units,
                    &mut |units| context.checkpoint(units),
                    memory,
                )
            },
            |result| super::super::read_footprint::relations(&result.records),
        )
    }
}

pub(super) struct RelationKindScan<'charge, 'memory> {
    memory: Option<&'charge mut worth_execution::ExecutionResultMemory<'memory>>,
    charge: Option<&'charge mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>>,
    records: Vec<RelationReadRecord>,
    relation_slots_examined: usize,
    relation_records_reserved: usize,
    work_units: usize,
    maximum_work_units: Option<usize>,
    require_materialization: bool,
}

impl<'charge, 'memory> RelationKindScan<'charge, 'memory> {
    #[cfg(test)]
    pub(super) const fn new(maximum_work_units: usize, require_materialization: bool) -> Self {
        Self {
            charge: None,
            memory: None,
            records: Vec::new(),
            relation_slots_examined: 0,
            relation_records_reserved: 0,
            work_units: 0,
            maximum_work_units: Some(maximum_work_units),
            require_materialization,
        }
    }

    pub(super) fn funded(
        maximum_work_units: usize,
        require_materialization: bool,
        charge: &'charge mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: Option<&'charge mut worth_execution::ExecutionResultMemory<'memory>>,
    ) -> Self {
        Self {
            charge: Some(charge),
            memory,
            records: Vec::new(),
            relation_slots_examined: 0,
            relation_records_reserved: 0,
            work_units: 0,
            maximum_work_units: Some(maximum_work_units),
            require_materialization,
        }
    }

    fn spend(&mut self, units: u64) -> Result<(), NativeScanStop<RelationKindTruthReadDenial>> {
        if self.maximum_work_units == Some(self.work_units) {
            return Err(NativeScanStop::Limit(
                RelationKindTruthReadDenial::WorkLimitExceeded(
                    RelationKindTruthReadLimitExceeded {
                        relation_slots_examined: self.relation_slots_examined,
                        relation_records_reserved: self.relation_records_reserved,
                    },
                ),
            ));
        }
        if let Some(charge) = self.charge.as_mut() {
            charge(units).map_err(NativeScanStop::request)?;
        }
        self.work_units += 1;
        Ok(())
    }

    fn examine_slot(&mut self) -> Result<(), NativeScanStop<RelationKindTruthReadDenial>> {
        self.spend(super::super::read_prices::EXAMINE_SLOT)?;
        self.relation_slots_examined += 1;
        Ok(())
    }

    fn reserve_with(
        &mut self,
        partition_id: crate::identity::data::PartitionId,
        slot: usize,
        clone_bytes: u64,
        materialize: impl FnOnce() -> Option<RelationReadRecord>,
    ) -> Result<(), NativeScanStop<RelationKindTruthReadDenial>> {
        self.spend(super::super::read_prices::RESERVE_RECORD)?;
        self.relation_records_reserved += 1;
        if let Some(memory) = self.memory.as_mut() {
            super::super::read_work::record_memory::prepare_push(
                memory,
                &mut self.records,
                clone_bytes,
            )
            .map_err(NativeScanStop::Request)?;
        }
        if let Some(record) = materialize() {
            if let Some(memory) = self.memory.as_mut() {
                super::super::read_work::record_memory::settle_clone(
                    memory,
                    clone_bytes,
                    super::super::read_footprint::relation(&record),
                )
                .map_err(NativeScanStop::Request)?;
            }
            self.records.push(record);
        } else if self.require_materialization {
            return Err(NativeScanStop::Limit(
                RelationKindTruthReadDenial::UnreadableRelationSlot { partition_id, slot },
            ));
        }
        Ok(())
    }

    pub(super) fn finish(self) -> BoundedRelationKindTruthRead {
        let mut records = self.records;
        super::truth_record_access::sort_authoritative_relation_records(&mut records);
        BoundedRelationKindTruthRead {
            records,
            relation_slots_examined: self.relation_slots_examined,
            relation_records_reserved: self.relation_records_reserved,
        }
    }

    #[cfg(test)]
    pub(super) fn into_records(self) -> Vec<RelationReadRecord> {
        self.records
    }
}

impl BoundedRelationKindTruthRead {
    pub(crate) fn owned_bytes(&self) -> u64 {
        super::super::read_footprint::relations(&self.records)
    }

    pub const fn relation_slots_examined(&self) -> usize {
        self.relation_slots_examined
    }

    pub const fn relation_records_reserved(&self) -> usize {
        self.relation_records_reserved
    }

    pub const fn work_units(&self) -> usize {
        self.relation_slots_examined + self.relation_records_reserved
    }

    pub fn records(&self) -> &[RelationReadRecord] {
        &self.records
    }

    pub fn into_records(self) -> Vec<RelationReadRecord> {
        self.records
    }
}

impl RelationKindTruthReadLimitExceeded {
    pub const fn relation_slots_examined(self) -> usize {
        self.relation_slots_examined
    }

    pub const fn relation_records_reserved(self) -> usize {
        self.relation_records_reserved
    }

    pub const fn consumed_work_units(self) -> usize {
        self.relation_slots_examined + self.relation_records_reserved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_scan_refuses_a_matched_slot_that_cannot_materialize() {
        let partition_id = crate::identity::data::PartitionId(3);
        let mut bounded = RelationKindScan::new(2, true);
        bounded.examine_slot().unwrap();
        assert_eq!(
            bounded.reserve_with(partition_id, 7, 0, || None),
            Err(NativeScanStop::Limit(
                RelationKindTruthReadDenial::UnreadableRelationSlot {
                    partition_id,
                    slot: 7,
                }
            )),
        );

        let mut legacy = RelationKindScan::new(usize::MAX, false);
        legacy.examine_slot().unwrap();
        legacy.reserve_with(partition_id, 7, 0, || None).unwrap();
        assert!(legacy.into_records().is_empty());
    }
}

#[cfg(test)]
mod paid_stop_tests;

impl ProjectionRecordReader<'_> {
    pub(in crate::visibility::materialization::read_records) fn bounded_relations_of_kind_in_state(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        kind_id: crate::identity::data::KindId,
        visibility: KindScanVisibility,
        maximum_work_units: usize,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<BoundedRelationKindTruthRead, NativeScanStop<RelationKindTruthReadDenial>> {
        let mut scan = RelationKindScan::funded(maximum_work_units, true, charge, Some(memory));
        let (partitions, partition_bytes) = super::super::read_work::record_memory::partition_ids(
            state,
            scan.memory
                .as_mut()
                .expect("native scan keeps its original loan"),
        )
        .map_err(NativeScanStop::Request)?;
        for partition_id in partitions {
            self.scan_relation_kind_in_partition(
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
        Ok(scan.finish())
    }

    pub(super) fn scan_relation_kind_in_partition(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        partition_id: crate::identity::data::PartitionId,
        kind_id: crate::identity::data::KindId,
        visibility: KindScanVisibility,
        scan: &mut RelationKindScan<'_, '_>,
    ) -> Result<(), NativeScanStop<RelationKindTruthReadDenial>> {
        let current_version = self.current_version;
        let Some(partition) = state.get_partition(partition_id) else {
            return Ok(());
        };
        match visibility {
            KindScanVisibility::Live => {
                let (slots, held) = super::super::read_work::record_memory::slots(
                    &partition.relation_arena.live_bitset,
                    scan.memory
                        .as_mut()
                        .expect("native bounded scans carry the initiating result loan"),
                )
                .map_err(NativeScanStop::Request)?;
                for slot in slots {
                    scan.examine_slot()?;
                    if !slot_kind_matches_current(&partition.relation_arena, slot, kind_id) {
                        continue;
                    }
                    scan.reserve_with(
                        partition_id,
                        slot,
                        super::super::read_work::record_quote::relation(
                            registry, partition, slot, visibility,
                        )
                        .map_err(NativeScanStop::Request)?,
                        || {
                            materialize_current_authoritative_relation_record(
                                registry,
                                partition,
                                partition_id,
                                slot,
                            )
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
                    &partition.relation_arena,
                    scan.memory
                        .as_mut()
                        .expect("native bounded scans carry the initiating result loan"),
                )
                .map_err(NativeScanStop::Request)?;
                for slot in slots {
                    scan.examine_slot()?;
                    self.instrumentation.count(|counters| {
                        counters.visibility_relation_slot_scans += 1;
                    });
                    if !relation_slot_matches_kind_at_version(
                        partition,
                        slot,
                        kind_id,
                        version_id,
                        current_version,
                    ) {
                        continue;
                    }
                    scan.reserve_with(
                        partition_id,
                        slot,
                        super::super::read_work::record_quote::relation(
                            registry, partition, slot, visibility,
                        )
                        .map_err(NativeScanStop::Request)?,
                        || {
                            materialize_authoritative_relation_record_at_version(
                                registry,
                                partition,
                                partition_id,
                                slot,
                                version_id,
                            )
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
