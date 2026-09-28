use super::*;
use worth_store_physical_format::*;

fn intent() -> PhysicalExtentCopyIntent {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let source = DurableExtentRecordPlacement::new(
        PersistedRecordIdentity::new([7; 16], 9).unwrap(),
        PhysicalGenerationAuthority::for_canonical_physical_format()
            .record_extent_cell(PhysicalExtentId::from_raw(3).unwrap())
            .with_extent_generation(PhysicalGeneration::from_raw(4).unwrap()),
        40_000,
        ExtentArenaRange::new(ExtentArenaId::new(2).unwrap(), 53_248, 53_248).unwrap(),
    )
    .unwrap();
    PhysicalExtentCopyIntent::new(
        format,
        [19; 32],
        12,
        source,
        ExtentArenaRange::new(ExtentArenaId::new(8).unwrap(), 0, 53_248).unwrap(),
        4096,
        [23; 32],
    )
    .unwrap()
}

#[test]
fn resolution_does_not_release_original_intent_before_checkpoint_coverage() {
    let intent = intent();
    let mut entries = Vec::new();
    observe_copy_record(
        &mut entries,
        PhysicalExtentCopyRecord::Intent(intent),
        2,
        1,
        8,
        9,
        0,
    )
    .unwrap();
    assert!(entries[0].holds_at(u64::MAX));
    let resolution = PhysicalExtentCopyResolution::new(
        intent.operation(),
        entries[0].digest,
        8,
        PhysicalExtentCopyResolutionKind::Cancelled,
    )
    .unwrap();
    observe_copy_record(
        &mut entries,
        PhysicalExtentCopyRecord::Resolved(resolution),
        5,
        1,
        20,
        21,
        0,
    )
    .unwrap();
    assert!(entries[0].holds_at(20));
    assert!(!entries[0].holds_at(21));
    assert!(entries[0].overlaps(1, 9));
    assert!(!entries[0].overlaps(9, 20));
    assert_eq!(entries[0].wal_interval(), (2, 1, 8, 9));
}

#[test]
fn copy_resolution_rejects_wrong_original_lsn_digest_or_unproved_publication() {
    let intent = intent();
    for (lsn, digest, kind) in [
        (9, [0; 32], PhysicalExtentCopyResolutionKind::Cancelled),
        (8, [0; 32], PhysicalExtentCopyResolutionKind::Cancelled),
        (
            8,
            Sha256::digest(PhysicalExtentCopyRecord::Intent(intent).encode()).into(),
            PhysicalExtentCopyResolutionKind::Published {
                root_generation: 13,
                publication_lsn: 14,
            },
        ),
    ] {
        let mut entries = Vec::new();
        observe_copy_record(
            &mut entries,
            PhysicalExtentCopyRecord::Intent(intent),
            2,
            1,
            8,
            9,
            0,
        )
        .unwrap();
        let resolution =
            PhysicalExtentCopyResolution::new(intent.operation(), digest, lsn, kind).unwrap();
        assert!(observe_copy_record(
            &mut entries,
            PhysicalExtentCopyRecord::Resolved(resolution),
            5,
            1,
            20,
            21,
            0
        )
        .is_err());
        assert!(entries[0].resolution().is_none());
    }
}

#[test]
fn duplicate_intent_and_cancellation_after_final_wal_are_refused() {
    let intent = intent();
    let mut entries = Vec::new();
    observe_copy_record(
        &mut entries,
        PhysicalExtentCopyRecord::Intent(intent),
        2,
        1,
        8,
        9,
        0,
    )
    .unwrap();
    assert!(observe_copy_record(
        &mut entries,
        PhysicalExtentCopyRecord::Intent(intent),
        2,
        1,
        9,
        10,
        0
    )
    .is_err());
    entries[0].publication = Some((13, 14));
    let resolution = PhysicalExtentCopyResolution::new(
        intent.operation(),
        entries[0].digest,
        8,
        PhysicalExtentCopyResolutionKind::Cancelled,
    )
    .unwrap();
    assert!(observe_copy_record(
        &mut entries,
        PhysicalExtentCopyRecord::Resolved(resolution),
        5,
        1,
        20,
        21,
        0
    )
    .is_err());
}

#[test]
fn completed_original_segment_reclamation_bounds_the_copy_ledger() {
    let intent = intent();
    let mut entries = Vec::new();
    for segment in 1..100 {
        observe_copy_record(
            &mut entries,
            PhysicalExtentCopyRecord::Intent(intent),
            segment,
            1,
            8,
            9,
            0,
        )
        .unwrap();
        prune_reclaimed(&mut entries, segment, 1);
        assert_eq!(
            entries.len(),
            1,
            "unresolved source obligation cannot disappear"
        );
        let resolution = PhysicalExtentCopyResolution::new(
            intent.operation(),
            entries[0].digest,
            8,
            PhysicalExtentCopyResolutionKind::Cancelled,
        )
        .unwrap();
        observe_copy_record(
            &mut entries,
            PhysicalExtentCopyRecord::Resolved(resolution),
            segment + 1,
            1,
            20,
            21,
            0,
        )
        .unwrap();
        prune_reclaimed(&mut entries, segment + 1, 1);
        assert_eq!(
            entries.len(),
            1,
            "only the original intent segment owns its retained ledger charge"
        );
        prune_reclaimed(&mut entries, segment, 1);
        assert!(entries.is_empty());
    }
}
