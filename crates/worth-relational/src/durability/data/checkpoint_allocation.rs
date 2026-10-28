//! Diagnostic comparison of allocator state against actual reservation evidence.
use super::{CheckpointRecordFrontier, CheckpointRecordIdentity};
use crate::history::data::RecordAllocationClass;
use crate::transactions::data::RecordRef;
use std::collections::BTreeMap;

/// Read-only allocator coordinates from a decoded checkpoint. No allocation authority.
#[derive(Clone, Copy, Debug)]
pub struct CheckpointAllocatorInspection<'a> {
    pub frontiers: &'a [CheckpointRecordFrontier],
    pub generations: &'a [CheckpointRecordIdentity],
}

/// Why two checkpoint allocators do not differ by exactly the supplied report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckpointReservationAdvanceDenial {
    DuplicateFrontier,
    DuplicateGeneration,
    SlotOverflow,
    GenerationOverflow,
    UnreservedAppendGap {
        reservation: CheckpointRecordIdentity,
        frontier: u64,
    },
    GenerationStep {
        reservation: CheckpointRecordIdentity,
        expected: u32,
    },
    FrontiersDiffer,
    GenerationsDiffer,
}

impl CheckpointAllocatorInspection<'_> {
    /// Reconstruct the exact after-state from ordered reservation evidence.
    /// Append slots must be contiguous; each reserved slot advances its generation
    /// exactly once, including reclaimed slots. Unreported moves and regressions
    /// fail. This diagnostic does not authenticate a report or grant write authority;
    /// callers obtain complete evidence from the runtime's reservation observer.
    /// O((A+R) log(A+R)) work and O(A+R) scratch for coordinates A and reservations R.
    pub fn verify_reported_advance(
        &self,
        after: &Self,
        reservations: impl IntoIterator<Item = CheckpointRecordIdentity>,
    ) -> Result<(), CheckpointReservationAdvanceDenial> {
        use CheckpointReservationAdvanceDenial as D;
        let mut expected_frontiers = frontiers(self.frontiers)?;
        let mut expected_generations = generations(self.generations)?;
        for reservation in reservations {
            let next = reservation.slot.checked_add(1).ok_or(D::SlotOverflow)?;
            let frontier = expected_frontiers
                .entry((reservation.class, reservation.partition_id))
                .or_insert(0);
            if reservation.slot > *frontier {
                return Err(D::UnreservedAppendGap {
                    reservation,
                    frontier: *frontier,
                });
            }
            if reservation.slot == *frontier {
                *frontier = next;
            }
            let generation = expected_generations
                .entry((
                    reservation.class,
                    reservation.partition_id,
                    reservation.slot,
                ))
                .or_insert(0);
            let expected = generation.checked_add(1).ok_or(D::GenerationOverflow)?;
            if reservation.generation != expected {
                return Err(D::GenerationStep {
                    reservation,
                    expected,
                });
            }
            *generation = expected;
        }
        if expected_frontiers != frontiers(after.frontiers)? {
            return Err(D::FrontiersDiffer);
        }
        if expected_generations != generations(after.generations)? {
            return Err(D::GenerationsDiffer);
        }
        Ok(())
    }
}

fn frontiers(
    rows: &[CheckpointRecordFrontier],
) -> Result<BTreeMap<(RecordAllocationClass, u32), u64>, CheckpointReservationAdvanceDenial> {
    unique(
        rows.iter()
            .map(|f| ((f.class, f.partition_id), f.next_slot)),
        CheckpointReservationAdvanceDenial::DuplicateFrontier,
    )
}
fn generations(
    rows: &[CheckpointRecordIdentity],
) -> Result<BTreeMap<(RecordAllocationClass, u32, u64), u32>, CheckpointReservationAdvanceDenial> {
    unique(
        rows.iter()
            .map(|g| ((g.class, g.partition_id, g.slot), g.generation)),
        CheckpointReservationAdvanceDenial::DuplicateGeneration,
    )
}
fn unique<K: Ord, V>(
    rows: impl Iterator<Item = (K, V)>,
    denial: CheckpointReservationAdvanceDenial,
) -> Result<BTreeMap<K, V>, CheckpointReservationAdvanceDenial> {
    let mut result = BTreeMap::new();
    for (key, value) in rows {
        if result.insert(key, value).is_some() {
            return Err(denial);
        }
    }
    Ok(result)
}

impl From<RecordRef> for CheckpointRecordIdentity {
    fn from(record: RecordRef) -> Self {
        match record {
            RecordRef::Entity(id) => Self {
                class: RecordAllocationClass::Entity,
                partition_id: id.partition_value(),
                slot: id.local_slot_value(),
                generation: id.generation_value(),
            },
            RecordRef::Relation(id) => Self {
                class: RecordAllocationClass::Relation,
                partition_id: id.partition_value(),
                slot: id.local_slot_value(),
                generation: id.generation_value(),
            },
        }
    }
}

#[cfg(test)]
mod tests;
