//! Bounded authoritative enumeration of live relations of one kind.

pub mod budget;
pub mod directory;
use super::super::materialization::materialize_authoritative_relation_record_at_version;
use super::super::native_scan_stop::NativeScanStop;
use super::super::visibility::relation_slot_matches_kind_at_version;
use super::{
    kind_scan_visibility::KindScanVisibility, projection_record_reader::ProjectionRecordReader,
};
use crate::{storage::data::RelationReadRecord, storage::overlay::PartitionAccess};
use budget::RelationKindScan;

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
    pub fn bounded_relations_of_kind_in_state(
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
                .expect(/* Proof: The private bounded scan constructor installs its initiating result loan; callbacks only borrow/resize it and no path takes it before scan completion. */ "native scan keeps its original loan"),
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
                .expect(/* Proof: The private bounded scan constructor installs its initiating result loan; callbacks only borrow/resize it and no path takes it before scan completion. */ "native scan keeps its original loan"),
            partition_bytes,
        )
        .map_err(NativeScanStop::Request)?;
        Ok(scan.finish())
    }

    fn scan_relation_kind_in_partition(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        partition_id: crate::identity::data::PartitionId,
        kind_id: crate::identity::data::KindId,
        visibility: KindScanVisibility,
        scan: &mut RelationKindScan<'_, '_>,
    ) -> Result<(), NativeScanStop<RelationKindTruthReadDenial>> {
        let current_version = self.current_version;
        if matches!(visibility, KindScanVisibility::Live) {
            let Some(head) = state.get_current_partition(partition_id) else {
                return Ok(());
            };
            for slot in head.relation_arena().live_slots() {
                scan.examine_slot()?;
                if head
                    .relation_arena()
                    .get_slot(slot)
                    .is_none_or(|record| record.kind_id() != Some(kind_id))
                {
                    continue;
                }
                scan.reserve_with(
                    partition_id,
                    slot,
                    super::super::read_work::record_quote::relation_head(registry, head, slot)
                        .map_err(NativeScanStop::Request)?,
                    || {
                        head.relation_record(
                            registry,
                            crate::identity::data::RelationId::new(partition_id, slot as u64, 0),
                        )
                    },
                )?;
            }
            return Ok(());
        }
        let Some(partition) = state.get_partition(partition_id) else {
            return Ok(());
        };
        match visibility {
            KindScanVisibility::Live => unreachable!(
                /* Proof: The current-read dispatch returns via Head before this historical branch; only an immutable historical basis reaches this match. */
                "current scans returned through Head"
            ),
            KindScanVisibility::AtVersion(version_id) => {
                let (slots, held) = super::super::read_work::record_memory::occupied_slots(
                    &partition.relation_arena,
                    scan.memory
                        .as_mut()
                        .expect(/* Proof: The private bounded scan constructor installs its initiating result loan; callbacks only borrow/resize it and no path takes it before scan completion. */ "native bounded scans carry the initiating result loan"),
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
                        .expect(/* Proof: The private bounded scan constructor installs its initiating result loan; callbacks only borrow/resize it and no path takes it before scan completion. */ "the original loan remains present"),
                    held,
                )
                .map_err(NativeScanStop::Request)?;
            }
        }
        Ok(())
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
