//! One relation scan's original request and optional semantic work limit.
use super::*;
pub(super) struct RelationKindScan<'charge, 'memory> {
    pub(super) memory: Option<&'charge mut worth_execution::ExecutionResultMemory<'memory>>,
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
        Self::with_limit(
            Some(maximum_work_units),
            require_materialization,
            charge,
            memory,
        )
    }
    pub(super) fn with_limit(
        maximum_work_units: Option<usize>,
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
            maximum_work_units,
            require_materialization,
        }
    }

    pub(super) fn spend(
        &mut self,
        units: u64,
    ) -> Result<(), NativeScanStop<RelationKindTruthReadDenial>> {
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

    pub(super) fn examine_slot(
        &mut self,
    ) -> Result<(), NativeScanStop<RelationKindTruthReadDenial>> {
        self.spend(super::super::super::read_prices::EXAMINE_SLOT)?;
        self.relation_slots_examined += 1;
        Ok(())
    }

    pub(super) fn reserve_with(
        &mut self,
        partition_id: crate::identity::data::PartitionId,
        slot: usize,
        clone_bytes: u64,
        materialize: impl FnOnce() -> Option<RelationReadRecord>,
    ) -> Result<(), NativeScanStop<RelationKindTruthReadDenial>> {
        self.spend(super::super::super::read_prices::RESERVE_RECORD)?;
        self.relation_records_reserved += 1;
        if let Some(memory) = self.memory.as_mut() {
            super::super::super::read_work::record_memory::prepare_push(
                memory,
                &mut self.records,
                clone_bytes,
            )
            .map_err(NativeScanStop::Request)?;
        }
        if let Some(record) = materialize() {
            if let Some(memory) = self.memory.as_mut() {
                super::super::super::read_work::record_memory::settle_clone(
                    memory,
                    clone_bytes,
                    super::super::super::read_footprint::relation(&record),
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
        super::super::truth_record_access::sort_authoritative_relation_records(&mut records);
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
