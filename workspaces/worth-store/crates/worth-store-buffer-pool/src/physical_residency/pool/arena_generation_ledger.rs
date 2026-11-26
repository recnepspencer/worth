use crate::allocation_generation_guard::{
    active_generation, ranges_overlap, GuardGeneration, GuardIdentity, GuardRange,
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
#[derive(Debug, Default)]
pub(super) struct ArenaGenerationLedger {
    seen: BTreeMap<(u64, u64), (u64, GuardGeneration)>,
    fenced: Vec<ExtentArenaRange>,
}

impl ArenaGenerationLedger {
    /// Every admitted interval overlapping `range`, in arena order.
    pub(super) fn overlapping(&self, range: GuardRange) -> Vec<GuardIdentity> {
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
        for older in self.overlapping(admitted) {
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
        self.fenced
            .retain(|fenced| !covers(admitted, guard_range(*fenced)));
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

const fn covers(outer: GuardRange, inner: GuardRange) -> bool {
    outer.arena == inner.arena
        && outer.offset <= inner.offset
        && inner.offset - outer.offset + inner.length <= outer.length
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
}
