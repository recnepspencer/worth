use super::super::*;
use super::eligibility::EvictionEligibility;
use worth_store_physical_format::RecordFrameCoordinate;

/// Proof that one frame was selected from the pool's legal eviction order
/// while the pool state remained exclusively locked.
///
/// Only the eviction selector can construct this token. Release paths consume
/// it, so a raw coordinate cannot invoke eviction execution.
#[derive(Debug)]
pub(in crate::physical_residency::pool) struct LegalEvictionVictim {
    coordinate: RecordFrameCoordinate,
}

impl LegalEvictionVictim {
    pub(super) const fn coordinate(&self) -> RecordFrameCoordinate {
        self.coordinate
    }
}

impl PoolState {
    pub(in crate::physical_residency::pool) fn select_oldest_legal_victim(
        &mut self,
    ) -> Option<LegalEvictionVictim> {
        let oldest = self.evictable_head?;
        let (coordinate, oldest_entry) = self
            .frames
            .frame_at_slot(oldest)
            .expect("an eviction-order identity remains resident");
        match oldest_entry.eviction_eligibility() {
            EvictionEligibility::Legal => {}
            exclusion => panic!(
                "{}: eviction order contained {exclusion:?}",
                exclusion
                    .violation_predicate()
                    .expect("an exclusion has a failure predicate")
            ),
        }
        self.detach_evictable_slot(oldest);
        Some(LegalEvictionVictim { coordinate })
    }
}
