#![allow(unexpected_cfgs)]

//! Pure `AllocationGenerationGuard` kernel for Store Milestone 4 (`M4.5`).
//!
//! Plain `cargo` compiles these exact bodies without `vstd`;
//! `tools/verus/run_verus.py check` proves them against the
//! `AddressReuseGenerationAndWriteFenceSound` contract: an arena range is
//! admitted for reuse only after a completed reuse barrier, with no
//! overlapping in-flight writeback, at a generation strictly above every
//! overlapping generation already seen.

#[macro_use]
mod macros;
#[cfg(all(test, not(verus_keep_ghost)))]
mod tests;

#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

verus_items! {
    /// One arena byte range `[offset, offset + length)`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct GuardRange {
        pub(crate) arena: u64,
        pub(crate) offset: u64,
        pub(crate) length: u64,
    }

    /// Lexicographic `(root_publication_generation, extent_fence_seq)`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct GuardGeneration {
        pub(crate) root: u64,
        pub(crate) seq: u64,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct GuardIdentity {
        pub(crate) range: GuardRange,
        pub(crate) generation: GuardGeneration,
    }

    /// The durable barrier that last moved a range toward allocator reuse.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum ReuseBarrier {
        RetirementIncomplete,
        CompletedRetirement,
        CancelUncovered,
        CheckpointCoveredCancel,
    }
}

#[cfg(verus_keep_ghost)]
verus! {
    spec fn spec_before_end(x: u64, start: u64, length: u64) -> bool {
        (x as int) < (start as int) + (length as int)
    }

    spec fn spec_overlaps(a: GuardRange, b: GuardRange) -> bool {
        a.arena == b.arena
            && spec_before_end(a.offset, b.offset, b.length)
            && spec_before_end(b.offset, a.offset, a.length)
    }

    spec fn spec_precedes(a: GuardGeneration, b: GuardGeneration) -> bool {
        a.root < b.root || (a.root == b.root && a.seq < b.seq)
    }

    spec fn spec_barrier_admits(barrier: ReuseBarrier) -> bool {
        barrier == ReuseBarrier::CompletedRetirement
            || barrier == ReuseBarrier::CheckpointCoveredCancel
    }

    spec fn spec_drained(range: GuardRange, writes: Seq<GuardRange>) -> bool {
        forall|i: int| #![auto] 0 <= i < writes.len() ==> !spec_overlaps(range, writes[i])
    }

    spec fn spec_exceeds_seen(
        range: GuardRange,
        seen: Seq<GuardIdentity>,
        next: GuardGeneration,
    ) -> bool {
        forall|i: int| #![auto] 0 <= i < seen.len() && spec_overlaps(range, seen[i].range)
            ==> spec_precedes(seen[i].generation, next)
    }
}

verus_fn! {
    /// `x < start + length` evaluated without overflow.
    fn before_end(x: u64, start: u64, length: u64) -> (result: bool);
    ensures { result == spec_before_end(x, start, length), }
    body {
        x < start || x - start < length
    }
}

verus_fn! {
    /// Exact, split and coalesced sub/super-range overlap in one arena.
    pub(crate) fn ranges_overlap(a: GuardRange, b: GuardRange) -> (result: bool);
    ensures { result == spec_overlaps(a, b), }
    body {
        a.arena == b.arena
            && before_end(a.offset, b.offset, b.length)
            && before_end(b.offset, a.offset, a.length)
    }
}

verus_fn! {
    pub(crate) fn generation_precedes(a: GuardGeneration, b: GuardGeneration) -> (result: bool);
    ensures { result == spec_precedes(a, b), }
    body {
        a.root < b.root || (a.root == b.root && a.seq < b.seq)
    }
}

verus_scan_fn! {
    /// No in-flight writeback overlaps `range`.
    pub(crate) fn writebacks_drained(range: GuardRange, writes: &[GuardRange]) -> (result: bool);
    ensures { result == spec_drained(range, writes@), }
    init {
        let mut i: usize = 0;
    }
    while (i < writes.len())
    invariant {
        0 <= i <= writes@.len(),
        forall|k: int| #![auto] 0 <= k < i ==> !spec_overlaps(range, writes@[k]),
    }
    decreases { writes@.len() - i, }
    step {
        if ranges_overlap(range, writes[i]) {
            return false;
        }
        i += 1;
    }
    after {
        true
    }
}

verus_scan_fn! {
    /// `next` is strictly above every generation seen on an overlapping range.
    pub(crate) fn exceeds_seen_generations(
        range: GuardRange,
        seen: &[GuardIdentity],
        next: GuardGeneration,
    ) -> (result: bool);
    ensures { result == spec_exceeds_seen(range, seen@, next), }
    init {
        let mut i: usize = 0;
    }
    while (i < seen.len())
    invariant {
        0 <= i <= seen@.len(),
        forall|k: int| #![auto] 0 <= k < i && spec_overlaps(range, seen@[k].range)
            ==> spec_precedes(seen@[k].generation, next),
    }
    decreases { seen@.len() - i, }
    step {
        let entry = seen[i];
        if ranges_overlap(range, entry.range) && !generation_precedes(entry.generation, next) {
            return false;
        }
        i += 1;
    }
    after {
        true
    }
}

verus_fn! {
    /// The `AdmitToAllocator` decision shared by retired and cancelled ranges.
    pub(crate) fn can_admit_for_reuse(
        range: GuardRange,
        writes: &[GuardRange],
        barrier: ReuseBarrier,
        seen: &[GuardIdentity],
        next: GuardGeneration,
    ) -> (result: bool);
    ensures {
        result == (spec_barrier_admits(barrier)
            && spec_drained(range, writes@)
            && spec_exceeds_seen(range, seen@, next)),
    }
    body {
        let admits = matches!(
            barrier,
            ReuseBarrier::CompletedRetirement | ReuseBarrier::CheckpointCoveredCancel
        );
        admits && writebacks_drained(range, writes) && exceeds_seen_generations(range, seen, next)
    }
}

verus_fn! {
    /// A frame or writeback serves a request only at the exact identity.
    pub(crate) fn frame_lookup_matches(
        frame: GuardIdentity,
        range: GuardRange,
        generation: GuardGeneration,
    ) -> (result: bool);
    ensures { result == (frame.range == range && frame.generation == generation), }
    body {
        frame.range.arena == range.arena
            && frame.range.offset == range.offset
            && frame.range.length == range.length
            && frame.generation.root == generation.root
            && frame.generation.seq == generation.seq
    }
}

verus_scan_fn! {
    /// The newest generation seen on any range overlapping `range`, or the
    /// opened generation `(0, 0)` when none overlaps.
    pub(crate) fn active_generation(
        range: GuardRange,
        seen: &[GuardIdentity],
    ) -> (result: GuardGeneration);
    ensures {
        forall|k: int| #![auto] 0 <= k < seen@.len() && spec_overlaps(range, seen@[k].range)
            ==> !spec_precedes(result, seen@[k].generation),
        (result.root == 0 && result.seq == 0) || exists|k: int| #![auto] 0 <= k < seen@.len()
            && spec_overlaps(range, seen@[k].range) && seen@[k].generation == result,
    }
    init {
        let mut best = GuardGeneration { root: 0, seq: 0 };
        let mut i: usize = 0;
    }
    while (i < seen.len())
    invariant {
        0 <= i <= seen@.len(),
        forall|k: int| #![auto] 0 <= k < i && spec_overlaps(range, seen@[k].range)
            ==> !spec_precedes(best, seen@[k].generation),
        (best.root == 0 && best.seq == 0) || exists|k: int| #![auto] 0 <= k < i
            && spec_overlaps(range, seen@[k].range) && seen@[k].generation == best,
    }
    decreases { seen@.len() - i, }
    step {
        let entry = seen[i];
        if ranges_overlap(range, entry.range) && generation_precedes(best, entry.generation) {
            best = entry.generation;
        }
        i += 1;
    }
    after {
        best
    }
}
