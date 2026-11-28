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

use super::super::projection::exact_basis_reads::ExactProjectionReader;
use super::bounded_kind_scan::{BoundedRecordClass, KindScan};
use super::record_class::Entity;
use super::{
    kind_scan_visibility::KindScanVisibility, projection_record_reader::ProjectionRecordReader,
};
use crate::identity::data::PartitionId;
use crate::{storage::data::EntityReadRecord, storage::overlay::PartitionAccess};
use worth_relational_contracts::internal_port::execution::request_or::RequestOr;

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
    pub(in crate::internal_port::visibility::materialization::read_records) fn entities_from_directory(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &ExactProjectionReader<'_>,
        kind: crate::identity::data::KindId,
        partition: Option<crate::identity::data::PartitionId>,
        maximum: Option<usize>,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<BoundedEntityKindTruthRead, RequestOr<EntityKindTruthReadLimitExceeded>> {
        let mut scan = KindScan::<Entity>::new(maximum, true, Some(charge), Some(memory));
        self.scan_kind_from_directory(
            execution,
            source,
            kind,
            source.entity_ids(kind, partition),
            &mut scan,
        )?;
        Ok(finish_scan(scan))
    }

    pub fn bounded_entities_of_kind_in_state(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        kind_id: crate::identity::data::KindId,
        visibility: KindScanVisibility,
        maximum_work_units: usize,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<BoundedEntityKindTruthRead, RequestOr<EntityKindTruthReadLimitExceeded>> {
        let mut scan =
            KindScan::<Entity>::new(Some(maximum_work_units), true, Some(charge), Some(memory));
        self.scan_kind_in_state(state, registry, kind_id, visibility, &mut scan)?;
        let read = finish_scan(scan);
        debug_assert!(
            super::truth_access::authoritative_entity_records_are_canonical(read.records())
        );
        Ok(read)
    }
}

fn finish_scan(scan: KindScan<'_, '_, Entity>) -> BoundedEntityKindTruthRead {
    let (records, entity_slots_examined, _) = scan.finish();
    BoundedEntityKindTruthRead {
        records,
        entity_slots_examined,
    }
}

impl BoundedRecordClass for Entity {
    type Denial = EntityKindTruthReadLimitExceeded;
    const COUNT_MATERIALIZED: bool = true;

    fn limit(examined: usize, reserved: usize) -> Self::Denial {
        EntityKindTruthReadLimitExceeded {
            entity_slots_examined: examined,
            entity_records_reserved: reserved,
        }
    }

    fn unreadable(_: PartitionId, _: usize) -> Self::Denial {
        panic!("a matching visible entity slot must materialize")
    }
}

pub trait BoundedEntityKindTruthReadFieldsPort {
    fn owner_records(&self) -> &Vec<EntityReadRecord>;
}

impl BoundedEntityKindTruthReadFieldsPort for BoundedEntityKindTruthRead {
    fn owner_records(&self) -> &Vec<EntityReadRecord> {
        &self.records
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
            let mut scan = KindScan::<Entity>::new(Some(8), true, Some(&mut charge), None);
            assert_eq!(
                scan.examine_slot(),
                Err(RequestOr::Request(crate::execution::denial::kernel_stop(
                    stop
                )))
            );
            assert_eq!(finish_scan(scan).entity_records_reserved(), 0);
        }
    }
}
