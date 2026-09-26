//! Bounded authoritative enumeration of live relations of one kind.

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
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
        maximum_work_units: usize,
    ) -> Result<BoundedRelationKindTruthRead, RelationKindTruthReadDenial> {
        let state = self.runtime.storage_access().current_edition();
        let registry = &self.runtime.config.schema.registry;
        let mut scan = RelationKindScan::new(maximum_work_units, true);
        for partition_id in state.partition_ids() {
            self.scan_relation_kind_in_partition(
                &state,
                registry,
                partition_id,
                kind_id,
                version_id,
                &mut scan,
            )?;
        }
        Ok(scan.finish())
    }

    pub(super) fn scan_relation_kind_in_partition(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        partition_id: crate::identity::data::PartitionId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
        scan: &mut RelationKindScan,
    ) -> Result<(), RelationKindTruthReadDenial> {
        let current_version = VersionSource::current_version_id(self.runtime);
        let Some(partition) = state.get_partition(partition_id) else {
            return Ok(());
        };
        if version_id == current_version {
            for slot in partition.relation_arena.live_bitset.iter_set_slots() {
                scan.examine_slot()?;
                if !slot_kind_matches_current(&partition.relation_arena, slot, kind_id) {
                    continue;
                }
                scan.reserve_with(partition_id, slot, || {
                    materialize_current_authoritative_relation_record(
                        registry,
                        partition,
                        partition_id,
                        slot,
                    )
                })?;
            }
        } else {
            for slot in partition.relation_arena.occupied_slots() {
                scan.examine_slot()?;
                self.runtime.services.instrumentation.count(|counters| {
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
                scan.reserve_with(partition_id, slot, || {
                    materialize_authoritative_relation_record_at_version(
                        registry,
                        partition,
                        partition_id,
                        slot,
                        version_id,
                    )
                })?;
            }
        }
        Ok(())
    }
}

pub(super) struct RelationKindScan {
    records: Vec<RelationReadRecord>,
    relation_slots_examined: usize,
    relation_records_reserved: usize,
    work_units: usize,
    maximum_work_units: usize,
    require_materialization: bool,
}

impl RelationKindScan {
    pub(super) const fn new(maximum_work_units: usize, require_materialization: bool) -> Self {
        Self {
            records: Vec::new(),
            relation_slots_examined: 0,
            relation_records_reserved: 0,
            work_units: 0,
            maximum_work_units,
            require_materialization,
        }
    }

    fn spend(&mut self) -> Result<(), RelationKindTruthReadDenial> {
        if self.work_units == self.maximum_work_units {
            return Err(RelationKindTruthReadDenial::WorkLimitExceeded(
                RelationKindTruthReadLimitExceeded {
                    relation_slots_examined: self.relation_slots_examined,
                    relation_records_reserved: self.relation_records_reserved,
                },
            ));
        }
        self.work_units += 1;
        Ok(())
    }

    fn examine_slot(&mut self) -> Result<(), RelationKindTruthReadDenial> {
        self.spend()?;
        self.relation_slots_examined += 1;
        Ok(())
    }

    fn reserve_with(
        &mut self,
        partition_id: crate::identity::data::PartitionId,
        slot: usize,
        materialize: impl FnOnce() -> Option<RelationReadRecord>,
    ) -> Result<(), RelationKindTruthReadDenial> {
        self.spend()?;
        self.relation_records_reserved += 1;
        if let Some(record) = materialize() {
            self.records.push(record);
        } else if self.require_materialization {
            return Err(RelationKindTruthReadDenial::UnreadableRelationSlot { partition_id, slot });
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

    pub(super) fn into_records(self) -> Vec<RelationReadRecord> {
        self.records
    }
}

impl BoundedRelationKindTruthRead {
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
            bounded.reserve_with(partition_id, 7, || None),
            Err(RelationKindTruthReadDenial::UnreadableRelationSlot {
                partition_id,
                slot: 7,
            }),
        );

        let mut legacy = RelationKindScan::new(usize::MAX, false);
        legacy.examine_slot().unwrap();
        legacy.reserve_with(partition_id, 7, || None).unwrap();
        assert!(legacy.into_records().is_empty());
    }
}
