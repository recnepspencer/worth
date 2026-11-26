//! The planner's per-artifact windows and denial mapping (`M5`, `M8`).
use super::*;
use worth_store_physical_format::{ExtentArenaId, ExtentArenaRange};

fn segment(source_root: u64) -> DisplacedArtifact {
    DisplacedArtifact {
        source_root,
        artifact: RetiredArtifact::Segment {
            segment: 4,
            generation: 2,
        },
        bytes: 64,
    }
}

fn extent(source_root: u64) -> DisplacedArtifact {
    DisplacedArtifact {
        source_root,
        artifact: RetiredArtifact::Extent {
            extent: 1,
            generation: 3,
            range: ExtentArenaRange::new(ExtentArenaId::new(1).unwrap(), 0, 4096).unwrap(),
        },
        bytes: 4096,
    }
}

fn arena(source_root: u64) -> DisplacedArtifact {
    DisplacedArtifact {
        source_root,
        artifact: RetiredArtifact::Arena {
            arena: 1,
            generation: 3,
        },
        bytes: 8192,
    }
}

fn plan(
    displaced: &DisplacedArtifact,
    reach: &ProtectedReachSet,
) -> Option<PhysicalRetirementDenial> {
    plan_reclaim(&ReclaimCandidate::of(displaced), reach, true, false)
}

#[test]
fn a_segment_is_reached_only_by_its_exact_root_or_its_inline_tail() {
    let candidate = ReclaimCandidate::of(&segment(7));
    assert_eq!(
        candidate.window(),
        ReachWindow::Root {
            generation: 7,
            inline_tail: (4, 2)
        }
    );
    for protected in [6, 8] {
        let mut reach = ProtectedReachSet::default();
        reach.protect_root(protected);
        reach.protect_inline_tail(4, 3);
        reach.protect_inline_tail(5, 2);
        assert_eq!(
            plan(&segment(7), &reach),
            None,
            "unrelated root {protected}"
        );
    }
    let mut reach = ProtectedReachSet::default();
    reach.protect_root(7);
    assert_eq!(
        plan(&segment(7), &reach),
        Some(PhysicalRetirementDenial::Protected)
    );
    let mut reach = ProtectedReachSet::default();
    reach.protect_inline_tail(4, 2);
    assert_eq!(
        plan(&segment(7), &reach),
        Some(PhysicalRetirementDenial::Protected)
    );
}

#[test]
fn extents_and_arenas_are_reached_by_every_root_at_or_below_their_window() {
    assert_eq!(
        ReclaimCandidate::of(&extent(9)).window(),
        ReachWindow::RootsAtOrBelow(9)
    );
    assert_eq!(
        ReclaimCandidate::of(&arena(9)).window(),
        ReachWindow::RootsAtOrBelow(8)
    );
    let held = |generation| {
        let mut reach = ProtectedReachSet::default();
        reach.protect_root(generation);
        reach
    };
    assert_eq!(
        plan(&extent(9), &held(9)),
        Some(PhysicalRetirementDenial::Protected)
    );
    assert_eq!(
        plan(&extent(9), &held(1)),
        Some(PhysicalRetirementDenial::Protected)
    );
    assert_eq!(plan(&extent(9), &held(10)), None);
    assert_eq!(plan(&arena(9), &held(9)), None);
    assert_eq!(
        plan(&arena(9), &held(8)),
        Some(PhysicalRetirementDenial::Protected)
    );
    let mut unrestored = ProtectedReachSet::default();
    unrestored.protect_all_roots();
    for displaced in [segment(3), extent(3), arena(3)] {
        assert_eq!(
            plan(&displaced, &unrestored),
            Some(PhysicalRetirementDenial::Protected)
        );
    }
}

#[test]
fn coverage_and_obligations_refuse_in_the_existing_denial_order() {
    let candidate = ReclaimCandidate::of(&extent(4));
    let empty = ProtectedReachSet::default();
    assert_eq!(
        plan_reclaim(&candidate, &empty, false, true),
        Some(PhysicalRetirementDenial::Retained)
    );
    assert_eq!(
        plan_reclaim(&candidate, &empty, true, true),
        Some(PhysicalRetirementDenial::Unresolved)
    );
    let mut reach = ProtectedReachSet::default();
    reach.protect_root(2);
    assert_eq!(
        plan_reclaim(&candidate, &reach, false, true),
        Some(PhysicalRetirementDenial::Protected)
    );
}

#[test]
fn prune_waits_for_a_checkpoint_past_the_source_root() {
    assert!(!checkpoint_covers(&segment(5), 5));
    assert!(checkpoint_covers(&segment(5), 6));
    assert!(!checkpoint_covers(&extent(5), 5));
    assert!(checkpoint_covers(&arena(5), 5));
    assert!(!checkpoint_covers(&arena(5), 4));
}
