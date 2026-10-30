use std::path::Path;
#[allow(unused_imports)]
use worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::*;
use {std::num::NonZeroU32, std::num::NonZeroU64};

use worth_store_physical_backend::MediaOperationRole;
use {sha2::Digest, sha2::Sha256};
use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
use {
    worth_store::physical_runtime::PhysicalDurabilityStateReopenFailure,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalSignalConstructionFailure,
    worth_store::physical_runtime::PhysicalWalGroupAppendOutcome,
    worth_store::physical_runtime::PhysicalWalOpenFailure,
    worth_store::physical_runtime::PhysicalWalPolicy,
    worth_store::physical_runtime::RecordBootstrapFailure,
    worth_store::physical_runtime::WalSegmentByteLimit,
    worth_store::physical_runtime::WalSegmentInventoryLimit,
};
use {worth_store_wal::WalArtifactStoreDenial, worth_store_wal::WalTopologyDenialKind};

use worth_store_testkit_store::physical_record_journeys::durability_admission::wal_append::prepared;
use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::durability_with_wal_policy,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::success,
};

#[test]
fn hostile_wal_inventories_fail_with_the_exact_reopen_denial() {
    for case in [
        worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::HostileInventory::InventoryLimit,
        worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::HostileInventory::NonCanonicalName,
        worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::HostileInventory::EmptySegment,
        worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::HostileInventory::SegmentIdentityMismatch,
        worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::HostileInventory::TornFrame,
        worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::HostileInventory::PayloadDigestMismatch,
        worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::HostileInventory::SegmentGap,
        worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::HostileInventory::LsnGap,
        worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::HostileInventory::GenerationMismatch,
    ] {
        let parent = tempfile::tempdir().unwrap();
        let store_root = parent.path().join("store");
        build_three_segment_inventory(&store_root);
        case.corrupt(&store_root);
        assert_eq!(
            reopen_failure(&store_root, case.policy()),
            case.expected(),
            "hostile case {case:?} must fail for its exact cause",
        );
    }
}

#[test]
fn trailing_empty_segment_from_interrupted_rotation_is_cleaned_before_reopen() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    build_three_segment_inventory_with_limit(&store_root, 4);
    let trailing = segment_path(&store_root, 4, 1);
    std::fs::write(&trailing, []).unwrap();
    let media_owner = media(&store_root);
    let durability = durability_with_wal_policy(&media_owner, wal_policy(SEGMENT_BYTES, 4));
    let (format, _, access) = configuration();

    let serving = match media_owner
        .open_record_store(worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(serving) => serving,
        worth_proof::TransitionOutcome::Failed(inspection) => match inspection.cause() {
            worth_store::physical_runtime::RecordBootstrapFailure::SignalConstruction(
                worth_store::physical_runtime::PhysicalSignalConstructionFailure::DurabilityStateReopenRejected(
                    worth_store::physical_runtime::PhysicalDurabilityStateReopenFailure::Wal(failure),
                ),
            ) => panic!("MUTANT_PREDICATE:c7-trailing-empty-wal-cleanup-omitted {failure:?}"),
            other => panic!("MUTANT_PREDICATE:c7-trailing-empty-wal-cleanup-omitted {other:?}"),
        },
        _ => panic!("MUTANT_PREDICATE:c7-trailing-empty-wal-cleanup-omitted"),
    };
    serving.close();
    if trailing.exists() {
        panic!("MUTANT_PREDICATE:c7-trailing-empty-wal-cleanup-omitted");
    }
}

#[test]
fn noncontiguous_empty_tail_is_rejected_without_cleanup() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    build_three_segment_inventory_with_limit(&store_root, 4);
    let noncontiguous = segment_path(&store_root, 5, 1);
    std::fs::write(&noncontiguous, []).unwrap();

    assert_eq!(
        reopen_failure(&store_root, wal_policy(SEGMENT_BYTES, 4)),
        worth_store::physical_runtime::PhysicalWalOpenFailure::Topology(
            worth_store_wal::WalTopologyDenialKind::NonContiguousSegment
        ),
    );
    assert!(noncontiguous.exists());
}

#[test]
fn oversized_first_segment_is_rejected_before_its_payload_is_read() {
    let empty_parent = tempfile::tempdir().unwrap();
    let empty_root = empty_parent.path().join("store");
    initialize_empty_store(&empty_root);
    let empty_media = media(&empty_root);
    let empty_observer = empty_media.observer();
    let before_empty = empty_observer.media_counters();
    let (format, _, access) = configuration();
    let empty_durability = durability_with_wal_policy(&empty_media, wal_policy(SEGMENT_BYTES, 3));
    success(
        empty_media.open_record_store(worth_store::physical_runtime::PhysicalRecordOpen::new(
            format,
            access,
            empty_durability,
        )),
    )
    .close();
    let empty_reads = empty_observer
        .media_counters()
        .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead)
        - before_empty
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead);

    let oversized_parent = tempfile::tempdir().unwrap();
    let oversized_root = oversized_parent.path().join("store");
    build_three_segment_inventory(&oversized_root);
    let first_segment = segment_path(&oversized_root, 1, 1);
    let observed = std::fs::metadata(first_segment).unwrap().len();
    let admitted = observed - 1;
    let oversized_media = media(&oversized_root);
    let oversized_observer = oversized_media.observer();
    let before_oversized = oversized_observer.media_counters();
    let durability = durability_with_wal_policy(&oversized_media, wal_policy(admitted, 3));
    let (format, _, access) = configuration();
    let failure = open_failure_from_media(
        oversized_media,
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability),
    );
    assert_eq!(
        failure,
        worth_store::physical_runtime::PhysicalWalOpenFailure::SegmentByteLimitExceeded {
            admitted,
            observed
        },
    );
    let oversized_reads = oversized_observer
        .media_counters()
        .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead)
        - before_oversized
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead);
    assert_eq!(
        oversized_reads, empty_reads,
        "oversized WAL rejection must add no payload read to ordinary bootstrap I/O",
    );
}
