//! Bounded authoritative enumeration of live relations of one kind.

use super::super::projection::exact_basis_reads::ExactProjectionReader;
use super::bounded_kind_scan::{BoundedRecordClass, KindScan};
use super::record_class::Relation;
use super::{
    kind_scan_visibility::KindScanVisibility, projection_record_reader::ProjectionRecordReader,
};
use crate::identity::data::PartitionId;
use crate::{storage::data::RelationReadRecord, storage::overlay::PartitionAccess};
use worth_relational_contracts::internal_port::execution::request_or::RequestOr;

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
        let mut bounded = KindScan::<Relation>::new(Some(2), true, None, None);
        bounded.examine_slot().unwrap();
        assert_eq!(
            bounded.reserve_with(partition_id, 7, 0, || None),
            Err(RequestOr::Inner(
                RelationKindTruthReadDenial::UnreadableRelationSlot {
                    partition_id,
                    slot: 7,
                }
            )),
        );

        let mut legacy = KindScan::<Relation>::new(Some(usize::MAX), false, None, None);
        legacy.examine_slot().unwrap();
        legacy.reserve_with(partition_id, 7, 0, || None).unwrap();
        assert!(finish_scan(legacy).into_records().is_empty());
    }
}

#[cfg(test)]
mod paid_stop_tests;

impl ProjectionRecordReader<'_> {
    pub(in crate::internal_port::visibility::materialization::read_records) fn relations_from_directory(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &ExactProjectionReader<'_>,
        kind: crate::identity::data::KindId,
        partition: Option<crate::identity::data::PartitionId>,
        maximum: Option<usize>,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<BoundedRelationKindTruthRead, RequestOr<RelationKindTruthReadDenial>> {
        let mut scan =
            KindScan::<Relation>::new(maximum, maximum.is_some(), Some(charge), Some(memory));
        self.scan_kind_from_directory(
            execution,
            source,
            kind,
            source.relation_ids(kind, partition),
            &mut scan,
        )?;
        Ok(finish_scan(scan))
    }

    pub fn bounded_relations_of_kind_in_state(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        kind_id: crate::identity::data::KindId,
        visibility: KindScanVisibility,
        maximum_work_units: usize,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<BoundedRelationKindTruthRead, RequestOr<RelationKindTruthReadDenial>> {
        let mut scan =
            KindScan::<Relation>::new(Some(maximum_work_units), true, Some(charge), Some(memory));
        self.scan_kind_in_state(state, registry, kind_id, visibility, &mut scan)?;
        let read = finish_scan(scan);
        Ok(read)
    }
}

fn finish_scan(scan: KindScan<'_, '_, Relation>) -> BoundedRelationKindTruthRead {
    let (records, relation_slots_examined, relation_records_reserved) = scan.finish();
    BoundedRelationKindTruthRead {
        records,
        relation_slots_examined,
        relation_records_reserved,
    }
}

impl BoundedRecordClass for Relation {
    type Denial = RelationKindTruthReadDenial;
    const COUNT_MATERIALIZED: bool = false;

    fn limit(examined: usize, reserved: usize) -> Self::Denial {
        RelationKindTruthReadDenial::WorkLimitExceeded(RelationKindTruthReadLimitExceeded {
            relation_slots_examined: examined,
            relation_records_reserved: reserved,
        })
    }

    fn unreadable(partition_id: PartitionId, slot: usize) -> Self::Denial {
        RelationKindTruthReadDenial::UnreadableRelationSlot { partition_id, slot }
    }
}

pub trait BoundedRelationKindTruthReadFieldsPort {
    fn owner_records(&self) -> &Vec<RelationReadRecord>;
}

impl BoundedRelationKindTruthReadFieldsPort for BoundedRelationKindTruthRead {
    fn owner_records(&self) -> &Vec<RelationReadRecord> {
        &self.records
    }
}
