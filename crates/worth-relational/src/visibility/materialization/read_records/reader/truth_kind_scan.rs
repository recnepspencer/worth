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
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
        maximum_work_units: usize,
    ) -> Result<BoundedEntityKindTruthRead, EntityKindTruthReadLimitExceeded> {
        let state = self.runtime.storage_access().current_edition();
        let registry = &self.runtime.config.schema.registry;
        let mut scan = EntityKindScan::new(maximum_work_units);
        for partition_id in state.partition_ids() {
            self.scan_entity_kind_in_partition(
                &state,
                registry,
                partition_id,
                kind_id,
                version_id,
                &mut scan,
            )?;
        }
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
        version_id: crate::identity::data::VersionId,
        scan: &mut EntityKindScan,
    ) -> Result<(), EntityKindTruthReadLimitExceeded> {
        let current_version = VersionSource::current_version_id(self.runtime);
        let Some(partition) = state.get_partition(partition_id) else {
            return Ok(());
        };
        if version_id == current_version {
            for slot in partition.entity_arena.live_bitset.iter_set_slots() {
                scan.examine_slot()?;
                if !slot_kind_matches_current(&partition.entity_arena, slot, kind_id) {
                    continue;
                }
                scan.reserve_with(|| {
                    self.runtime.services.instrumentation.count(|counters| {
                        counters.visible_authoritative_entity_records_materialized += 1;
                    });
                    materialize_current_authoritative_entity_record(
                        registry,
                        partition,
                        partition_id,
                        slot,
                    )
                    .expect("a matching live entity slot must materialize")
                })?;
            }
        } else {
            for slot in partition.entity_arena.occupied_slots() {
                scan.examine_slot()?;
                self.runtime.services.instrumentation.count(|counters| {
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
                scan.reserve_with(|| {
                    self.runtime.services.instrumentation.count(|counters| {
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
                })?;
            }
        }
        Ok(())
    }
}

/// The running cost of one kind scan and the records it has reserved so far.
pub(super) struct EntityKindScan {
    records: Vec<EntityReadRecord>,
    entity_slots_examined: usize,
    work_units: usize,
    maximum_work_units: usize,
}

impl EntityKindScan {
    pub(super) const fn new(maximum_work_units: usize) -> Self {
        Self {
            records: Vec::new(),
            entity_slots_examined: 0,
            work_units: 0,
            maximum_work_units,
        }
    }

    fn examine_slot(&mut self) -> Result<(), EntityKindTruthReadLimitExceeded> {
        self.spend()?;
        self.entity_slots_examined += 1;
        Ok(())
    }

    fn reserve_with(
        &mut self,
        materialize: impl FnOnce() -> EntityReadRecord,
    ) -> Result<(), EntityKindTruthReadLimitExceeded> {
        self.spend()?;
        self.records.push(materialize());
        Ok(())
    }

    fn spend(&mut self) -> Result<(), EntityKindTruthReadLimitExceeded> {
        if self.work_units == self.maximum_work_units {
            return Err(EntityKindTruthReadLimitExceeded {
                entity_slots_examined: self.entity_slots_examined,
                entity_records_reserved: self.records.len(),
            });
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

    pub(super) fn into_records(self) -> Vec<EntityReadRecord> {
        self.records
    }
}

impl BoundedEntityKindTruthRead {
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
