use crate::allocation_generation_guard::{
    GuardGeneration, GuardIdentity, GuardRange, active_generation, generation_precedes,
    ranges_overlap,
};
use std::collections::BTreeMap;
use worth_store_physical_format::{
    ArenaExtentFrameIdentity, ExtentAllocationGeneration, ExtentArenaId, ExtentArenaRange,
    RecordArtifactFile, RecordFrameCoordinate,
};

/// Generations admitted by `WritebackFenceDrain` in this residency incarnation
/// and the arena ranges currently fenced against new writeback claims.
///
/// Admitted generations are kept as disjoint intervals keyed by
/// `(arena, offset)`. An admission strictly exceeds every overlapping
/// generation, so it replaces the parts of older intervals it covers: the
/// newest overlapping generation of any frame range is unchanged, and lookups
/// visit only the few intervals that overlap.
#[derive(Debug)]
pub(super) struct ArenaGenerationLedger {
    floor: GuardGeneration,
    seen: BTreeMap<(u64, u64), (u64, GuardGeneration)>,
    fenced: Vec<ExtentArenaRange>,
}

impl Default for ArenaGenerationLedger {
    fn default() -> Self {
        Self {
            floor: GuardGeneration { root: 0, seq: 0 },
            seen: BTreeMap::new(),
            fenced: Vec::new(),
        }
    }
}

impl ArenaGenerationLedger {
    /// Seeds the durable reopen frontier floor so newly admitted frames and
    /// active-generation queries never rewind to `(0, 0)` after reopen.
    pub(super) fn seed_floor(&mut self, floor: ExtentAllocationGeneration) {
        let next = guard_generation(floor);
        if generation_precedes(self.floor, next) {
            self.floor = next;
        }
    }

    fn overlapping_seen(&self, range: GuardRange) -> Vec<GuardIdentity> {
        let end = range.offset + range.length;
        let mut found: Vec<GuardIdentity> = self
            .seen
            .range((range.arena, 0)..(range.arena, end))
            .rev()
            .map(|(&(arena, offset), &(length, generation))| GuardIdentity {
                range: GuardRange {
                    arena,
                    offset,
                    length,
                },
                generation,
            })
            .take_while(|entry| ranges_overlap(entry.range, range))
            .collect();
        found.reverse();
        found
    }

    /// Every admitted interval overlapping `range`, plus the durable reopen
    /// floor when non-zero, in arena order.
    pub(super) fn overlapping(&self, range: GuardRange) -> Vec<GuardIdentity> {
        let mut found = self.overlapping_seen(range);
        if self.floor.root > 0 || self.floor.seq > 0 {
            found.push(GuardIdentity {
                range,
                generation: self.floor,
            });
        }
        found
    }

    /// The allocation identity a frame at `coordinate` carries now, if the
    /// coordinate addresses an extent arena.
    pub(super) fn identity_for(
        &self,
        coordinate: RecordFrameCoordinate,
    ) -> Option<ArenaExtentFrameIdentity> {
        let range = arena_range(coordinate)?;
        let guard = guard_range(range);
        let active = active_generation(guard, &self.overlapping(guard));
        Some(ArenaExtentFrameIdentity::new(
            range,
            physical_generation(active),
        ))
    }

    pub(super) fn generation_for(
        &self,
        coordinate: RecordFrameCoordinate,
    ) -> Option<ExtentAllocationGeneration> {
        self.identity_for(coordinate)
            .map(ArenaExtentFrameIdentity::generation)
    }

    pub(super) fn fences(&self, coordinate: RecordFrameCoordinate) -> bool {
        arena_range(coordinate).is_some_and(|range| {
            self.fenced
                .iter()
                .any(|fenced| ranges_overlap(guard_range(*fenced), guard_range(range)))
        })
    }

    pub(super) fn fence(&mut self, range: ExtentArenaRange) {
        if !self.fenced.contains(&range) {
            self.fenced.push(range);
        }
    }

    /// Records `generation` over `range` and lifts every fence the admission
    /// covers: the covering drain already settled each writeback those fences
    /// held.
    pub(super) fn admit(
        &mut self,
        range: ExtentArenaRange,
        generation: ExtentAllocationGeneration,
    ) {
        let admitted = guard_range(range);
        let end = range.end();
        for older in self.overlapping_seen(admitted) {
            let older_range = older.range;
            self.seen.remove(&(older_range.arena, older_range.offset));
            let older_end = older_range.offset + older_range.length;
            if older_range.offset < admitted.offset {
                self.seen.insert(
                    (older_range.arena, older_range.offset),
                    (admitted.offset - older_range.offset, older.generation),
                );
            }
            if older_end > end {
                self.seen.insert(
                    (older_range.arena, end),
                    (older_end - end, older.generation),
                );
            }
        }
        self.seen.insert(
            (admitted.arena, admitted.offset),
            (admitted.length, guard_generation(generation)),
        );
        self.clear_fence(range);
    }

    /// Carves out `range` from any overlapping fences once its drain settled.
    pub(super) fn clear_fence(&mut self, range: ExtentArenaRange) {
        let admitted = guard_range(range);
        let end = range.end();
        let mut remaining_fences = Vec::with_capacity(self.fenced.len());
        for fenced in self.fenced.drain(..) {
            let f = guard_range(fenced);
            if !ranges_overlap(admitted, f) {
                remaining_fences.push(fenced);
                continue;
            }
            let f_end = fenced.end();
            if f.offset < admitted.offset {
                if let Some(left) =
                    ExtentArenaRange::new(fenced.arena(), f.offset, admitted.offset - f.offset)
                {
                    remaining_fences.push(left);
                }
            }
            if f_end > end {
                if let Some(right) = ExtentArenaRange::new(fenced.arena(), end, f_end - end) {
                    remaining_fences.push(right);
                }
            }
        }
        self.fenced = remaining_fences;
    }
}

pub(super) fn arena_range(coordinate: RecordFrameCoordinate) -> Option<ExtentArenaRange> {
    let RecordArtifactFile::ExtentArena { arena } = coordinate.artifact() else {
        return None;
    };
    ExtentArenaRange::new(
        ExtentArenaId::new(arena)?,
        coordinate.offset(),
        u64::from(coordinate.length()),
    )
}

pub(super) const fn guard_range(range: ExtentArenaRange) -> GuardRange {
    GuardRange {
        arena: range.arena().get(),
        offset: range.offset(),
        length: range.length(),
    }
}

pub(super) const fn guard_generation(generation: ExtentAllocationGeneration) -> GuardGeneration {
    GuardGeneration {
        root: generation.root_publication_generation(),
        seq: generation.extent_fence_seq(),
    }
}

pub(super) fn physical_generation(generation: GuardGeneration) -> ExtentAllocationGeneration {
    ExtentAllocationGeneration::at_durable_barrier(generation.root, generation.seq)
        .unwrap_or(ExtentAllocationGeneration::OPENED)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn range(offset: u64, length: u64) -> ExtentArenaRange {
        ExtentArenaRange::new(ExtentArenaId::new(3).unwrap(), offset, length).unwrap()
    }

    fn generation(seq: u64) -> ExtentAllocationGeneration {
        ExtentAllocationGeneration::at_durable_barrier(1, seq).unwrap()
    }

    fn active(ledger: &ArenaGenerationLedger, offset: u64, length: u64) -> GuardGeneration {
        let probe = guard_range(range(offset, length));
        active_generation(probe, &ledger.overlapping(probe))
    }

    #[test]
    fn admission_carves_older_intervals_without_changing_any_active_generation() {
        let mut ledger = ArenaGenerationLedger::default();
        ledger.admit(range(0, 64), generation(1));
        ledger.admit(range(16, 16), generation(2));
        ledger.admit(range(48, 32), generation(3));
        assert_eq!(ledger.seen.len(), 4);
        assert_eq!(active(&ledger, 0, 16), guard_generation(generation(1)));
        assert_eq!(active(&ledger, 0, 17), guard_generation(generation(2)));
        assert_eq!(active(&ledger, 32, 16), guard_generation(generation(1)));
        assert_eq!(active(&ledger, 40, 16), guard_generation(generation(3)));
        assert_eq!(active(&ledger, 80, 16), GuardGeneration { root: 0, seq: 0 });
        ledger.admit(range(0, 128), generation(4));
        assert_eq!(ledger.seen.len(), 1);
        assert_eq!(active(&ledger, 20, 1), guard_generation(generation(4)));
    }

    #[test]
    fn split_admissions_carve_fences_and_seeded_floor_prevents_rewind() {
        let mut ledger = ArenaGenerationLedger::default();
        ledger.seed_floor(generation(10));
        assert_eq!(active(&ledger, 0, 64), guard_generation(generation(10)));
        ledger.fence(range(0, 128));
        let coord = |offset, len| {
            RecordFrameCoordinate::new(RecordArtifactFile::ExtentArena { arena: 3 }, offset, len)
                .unwrap()
        };
        assert!(ledger.fences(coord(0, 32)));
        assert!(ledger.fences(coord(96, 32)));
        ledger.admit(range(0, 64), generation(11));
        assert!(!ledger.fences(coord(0, 32)));
        assert!(ledger.fences(coord(64, 32)));
        ledger.admit(range(64, 64), generation(12));
        assert!(!ledger.fences(coord(64, 32)));
        assert!(ledger.fenced.is_empty());
    }
}
