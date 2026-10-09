use super::{held_identity_overlap, ArenaRanges};

#[test]
fn free_range_over_historical_hold_is_damaged_until_the_release_root() {
    use super::{ArenaAccounting, BoundedMediaWalk};
    use crate::{
        OfflineIntegrityObservationLimits, OfflineIntegrityOutcome, OfflinePhysicalDamageCause,
    };
    use std::time::{Instant, SystemTime, UNIX_EPOCH};

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("worth-held-arena-{}-{nonce}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let root = std::fs::canonicalize(root).unwrap();
    std::fs::write(root.join("arena.data"), [0; 4096]).unwrap();
    let limits = OfflineIntegrityObservationLimits::new(8, 4096, 5, 4, 0, 10_000, 4096).unwrap();
    let mut walk = BoundedMediaWalk::new(limits, root.clone(), Instant::now());
    let mut accounting = ArenaAccounting::default();
    accounting.held_coverage("arena.data".to_owned(), 0, 4096, 7, 1, 10, 15);
    for generation in [14, 15] {
        accounting.geometry.insert(generation, (8192, 4096));
        accounting.roots.insert(
            (generation, "arena.data".to_owned()),
            ArenaRanges {
                free: vec![(0, 8192)],
                ..ArenaRanges::default()
            },
        );
    }
    let observations = accounting.finish(&root, &mut walk);
    assert_eq!(observations.len(), 2);
    assert!(
        matches!(observations[0].outcome(), OfflineIntegrityOutcome::Damaged(damage)
        if damage.cause() == OfflinePhysicalDamageCause::ScopeMismatch)
    );
    assert_eq!(observations[1].outcome(), &OfflineIntegrityOutcome::Intact);
    drop(walk);
    std::fs::remove_file(root.join("arena.data")).unwrap();
    std::fs::remove_dir(root).unwrap();
}

#[test]
fn independent_accounting_rejects_overlapping_routes_free_routes_and_gaps() {
    for (routed, free, expected) in [
        (vec![(0, 16), (8, 24)], vec![(24, 32)], Some((8, 16))),
        (vec![(0, 16)], vec![(8, 32)], Some((8, 16))),
        (vec![(0, 16)], vec![(24, 32)], Some((16, 24))),
        (vec![(0, 16)], vec![(16, 64)], None),
    ] {
        assert_eq!(
            ArenaRanges {
                routed,
                free,
                historical: Vec::new(),
                uncertain: Vec::new()
            }
            .inspect(32),
            expected
        );
    }
}

#[test]
fn held_source_identity_rejects_new_route_over_its_physical_bytes() {
    let path = "families/records/arenas/arena-0000000000000002.data";
    let holds = vec![(path.to_owned(), 4096, 8192, 7, 3, 10, 14)];
    assert_eq!(
        held_identity_overlap(12, path, Some(&vec![(4096, 8192, 7, 3, 12)]), &holds),
        None
    );
    assert_eq!(
        held_identity_overlap(12, path, Some(&vec![(4096, 8192, 9, 1, 12)]), &holds),
        Some((4096, 8192))
    );
    assert_eq!(
        held_identity_overlap(12, path, Some(&vec![(6144, 10240, 9, 1, 12)]), &holds),
        Some((6144, 8192))
    );
    assert_eq!(
        held_identity_overlap(14, path, Some(&vec![(4096, 8192, 9, 1, 14)]), &holds),
        None
    );
}

#[test]
fn two_live_historical_protections_cannot_overlap_with_distinct_identities() {
    let path = "families/records/arenas/arena-0000000000000002.data";
    let holds = vec![
        (path.to_owned(), 4096, 8192, 7, 3, 10, 14),
        (path.to_owned(), 6144, 10240, 8, 1, 11, 15),
    ];
    assert_eq!(
        held_identity_overlap(12, path, None, &holds),
        Some((6144, 8192))
    );
}

#[test]
fn later_clean_free_space_cannot_certify_an_unrelated_predecessor_gap() {
    let mut previous = ArenaRanges {
        routed: vec![(0, 4096)],
        free: vec![(8192, 16384)],
        historical: Vec::new(),
        uncertain: vec![(4096, 8192)],
    };
    assert_eq!(
        previous.inspect(16384),
        None,
        "later free membership explains only the bytes, not the transition"
    );
    assert!(
        previous.has_unproven_gap(16384),
        "a predecessor gap must classify Unknown, never Intact"
    );
}
