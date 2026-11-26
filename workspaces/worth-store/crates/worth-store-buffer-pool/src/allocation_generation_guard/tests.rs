use super::*;

const fn range(offset: u64, length: u64) -> GuardRange {
    GuardRange {
        arena: 1,
        offset,
        length,
    }
}

const fn generation(root: u64, seq: u64) -> GuardGeneration {
    GuardGeneration { root, seq }
}

#[test]
fn reuse_requires_barrier_drained_writes_and_a_strictly_newer_generation() {
    let freed = range(0, 8192);
    let seen = [GuardIdentity {
        range: range(4096, 4096),
        generation: generation(5, 20),
    }];
    let next = generation(5, 21);
    let ok = |barrier, writes: &[GuardRange], next| {
        can_admit_for_reuse(freed, writes, barrier, &seen, next)
    };
    assert!(ok(ReuseBarrier::CompletedRetirement, &[], next));
    assert!(ok(
        ReuseBarrier::CheckpointCoveredCancel,
        &[range(8192, 4096)],
        next
    ));
    assert!(!ok(ReuseBarrier::RetirementIncomplete, &[], next));
    assert!(!ok(ReuseBarrier::CancelUncovered, &[], next));
    // Split and coalesced in-flight writebacks both block reuse.
    assert!(!ok(
        ReuseBarrier::CompletedRetirement,
        &[range(4096, 4096)],
        next
    ));
    assert!(!ok(
        ReuseBarrier::CompletedRetirement,
        &[range(0, 16384)],
        next
    ));
    // Reissuing the seen generation without a fence advance is refused.
    assert!(!ok(
        ReuseBarrier::CheckpointCoveredCancel,
        &[],
        generation(5, 20)
    ));
    assert_eq!(active_generation(range(0, 4096), &seen), generation(0, 0));
    assert_eq!(
        active_generation(range(6144, 2048), &seen),
        generation(5, 20)
    );
}

#[test]
fn lookup_matches_only_the_exact_range_and_generation() {
    let frame = GuardIdentity {
        range: range(0, 4096),
        generation: generation(2, 9),
    };
    assert!(frame_lookup_matches(
        frame,
        range(0, 4096),
        generation(2, 9)
    ));
    assert!(!frame_lookup_matches(
        frame,
        range(0, 4096),
        generation(2, 10)
    ));
    assert!(!frame_lookup_matches(
        frame,
        range(0, 2048),
        generation(2, 9)
    ));
    assert!(ranges_overlap(
        range(u64::MAX - 1, 1),
        range(u64::MAX - 4096, 4096)
    ));
}
