#[allow(dead_code)]
mod phase_three_support;

use std::num::NonZeroU64;

use phase_three_support::*;
use worth_store_physical_format::{
    CheckpointBindingCompactionHeader, CheckpointDirtyFrameBasis, CheckpointRootBasis,
    CheckpointStreamEncoder, CheckpointWalSourceRange, DurableRootSelector,
    PhysicalCheckpointIdentity, PhysicalCheckpointSource, RecordArtifactFile,
    RecordFrameCoordinate, RootSelectorRole, ROOT_SELECTOR_BYTES,
};
use worth_store_recovery_physics::{
    PhysicalRootCandidateDenial, PhysicalRootSelectionDenial, SelectedPhysicalWalTailDenial,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryBlock, PhysicalRecoveryBlockKind, PhysicalRecoveryCheckpointIntegrityDenial,
    PhysicalRecoveryLimits, PhysicalRecoveryRootProtocolArtifact,
    PhysicalRecoveryRootProtocolDenial, PhysicalRecoverySourceDenial,
};

#[test]
fn foreign_store_selector_is_rejected_through_the_persisted_boundary() {
    let parent = tempfile::tempdir().unwrap();
    let primary_root = parent.path().join("primary");
    let alternate_root = parent.path().join("alternate");
    let primary_store = initialize_store(&primary_root);
    let alternate_store = initialize_store(&alternate_root);
    assert_ne!(primary_store, alternate_store);
    publish_synthetic_genesis(&primary_root, primary_store);
    publish_synthetic_genesis(&alternate_root, alternate_store);
    let alternate_records = alternate_root.join("families").join("records");
    let primary_records = primary_root.join("families").join("records");
    std::fs::copy(
        alternate_records.join("root-current.selector"),
        primary_records.join("root-current.selector"),
    )
    .unwrap();
    std::fs::copy(
        alternate_records
            .join("roots")
            .join("root-0000000000000001.manifest"),
        primary_records
            .join("roots")
            .join("root-0000000000000001.manifest"),
    )
    .unwrap();

    let discovered = admitted_recovery(&primary_root).discover().unwrap();
    assert_eq!(discovered.counters().current_root_admitted, 0);
    assert_eq!(discovered.counters().current_root_rejected, 1);
    let blocked = expect_blocked(
        discovered
            .select()
            .err()
            .expect("foreign persisted Store must block"),
    );
    assert_eq!(blocked.kind, PhysicalRecoveryBlockKind::RootProtocol);
    assert_eq!(blocked.store_identity(), primary_store);
    assert_eq!(blocked.evidence().counters.current_root_rejected, 1);
    assert!(matches!(
        blocked.evidence().source_denials.as_slice(),
        [
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
                denial: PhysicalRecoveryRootProtocolDenial::Integrity(_),
            },
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
                denial: PhysicalRecoveryRootProtocolDenial::Absent,
            },
            PhysicalRecoverySourceDenial::RootSlot {
                slot: RootSelectorRole::Current,
                denial: PhysicalRootCandidateDenial::SelectorAuthorityMismatch,
                observed_store: None,
                observed_role: None,
                observed_generation: None,
            },
            PhysicalRecoverySourceDenial::RootSelection(
                PhysicalRootSelectionDenial::NoAdmittedRoot
            )
        ]
    ));
    assert_eq!(blocked.recovery_effects(), 0);
}

#[test]
fn torn_current_retains_unlinked_previous_selection_cause() {
    let blocked = root_case("unlinked-previous", |root| {
        let current =
            DurableRootSelector::decode(&std::fs::read(current_selector(root)).unwrap()).unwrap();
        let previous = DurableRootSelector::new(
            current.store_identity(),
            current.format(),
            worth_store_physical_format::RootSelectorIdentity::new(2).unwrap(),
            RootSelectorRole::Previous,
            current.root_generation(),
            None,
            None,
        )
        .unwrap();
        std::fs::write(
            root.join("families")
                .join("records")
                .join("root-previous.selector"),
            previous.encode(),
        )
        .unwrap();
        std::fs::write(current_selector(root), [0_u8; ROOT_SELECTOR_BYTES]).unwrap();
    });
    assert!(matches!(
        blocked.evidence().source_denials.as_slice(),
        [
            PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
                denial: PhysicalRecoveryRootProtocolDenial::Integrity(_),
            },
            PhysicalRecoverySourceDenial::RootSlot {
                slot: RootSelectorRole::Current,
                denial: PhysicalRootCandidateDenial::SelectorIntegrity,
                ..
            },
            PhysicalRecoverySourceDenial::RootSelection(
                PhysicalRootSelectionDenial::PreviousFallbackUnlinked
            )
        ]
    ));
}

#[test]
fn checkpoint_denials_retain_truncation_integrity_and_count_causes() {
    let truncated = checkpoint_case("truncated", limits(), |path, _| {
        std::fs::write(path, b"short").unwrap();
    });
    assert_checkpoint_denial(&truncated, |denial| {
        matches!(
            denial,
            PhysicalRecoveryCheckpointIntegrityDenial::Integrity(_)
        )
    });
    assert_eq!(
        truncated.evidence().counters.checkpoint_integrity_attempts,
        1
    );
    assert_eq!(
        truncated
            .evidence()
            .counters
            .checkpoint_integrity_rejections,
        1
    );

    let integrity = checkpoint_case("integrity", limits(), |path, store| {
        publish_synthetic_checkpoint(path.parent().unwrap().parent().unwrap(), store);
        let mut bytes = std::fs::read(path).unwrap();
        *bytes.last_mut().unwrap() ^= 0x5a;
        std::fs::write(path, bytes).unwrap();
    });
    assert_checkpoint_denial(&integrity, |denial| {
        matches!(
            denial,
            PhysicalRecoveryCheckpointIntegrityDenial::Integrity(_)
        )
    });

    let mut declaration = limit_declaration(2, 8, 8 * 1024);
    declaration.dirty_frames = 1;
    let count_limits = PhysicalRecoveryLimits::admit(declaration).unwrap();
    let count = checkpoint_case("record-count", count_limits, |path, store| {
        write_checkpoint_with_two_dirty_records(path, store);
    });
    assert_checkpoint_denial(&count, |denial| {
        matches!(
            denial,
            PhysicalRecoveryCheckpointIntegrityDenial::DirtyRecordLimit {
                observed: 2,
                admitted: 1
            }
        )
    });
}

#[test]
fn corrupt_wal_counts_scanned_separately_from_valid_and_retains_identity() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    publish_synthetic_checkpoint(&root, store);
    let families = root.join("families");
    let (path, mut bytes) =
        worth_store_test_support::harness::recovery::wal_tail::prepare_persisted_wal_frame(
            &families,
            1,
            2,
            3,
            "corrupt-canonical",
            b"payload",
        );
    *bytes.last_mut().unwrap() ^= 0xa5;
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, bytes).unwrap();

    let discovered = admitted_recovery(&root).discover().unwrap();
    assert_eq!(discovered.counters().wal_segments_scanned, 1);
    assert_eq!(discovered.counters().valid_wal_segments, 0);
    assert_eq!(discovered.counters().wal_frames, 1);
    assert_eq!(discovered.counters().valid_wal_frames, 0);
    assert_eq!(discovered.counters().wal_corruption_denials, 1);
    let blocked = expect_blocked(
        discovered
            .select()
            .err()
            .expect("corrupt canonical WAL must block"),
    );
    let corruption = blocked
        .evidence()
        .source_denials
        .iter()
        .find_map(|denial| match denial {
            PhysicalRecoverySourceDenial::WalIntegrity(corruption) => Some(corruption),
            _ => None,
        })
        .expect("corrupt canonical WAL must retain typed artifact evidence");
    assert_eq!(
        corruption.artifact(),
        path.file_name().unwrap().to_string_lossy()
    );
    assert_eq!(corruption.identity().segment().get(), 1);
    assert!(format!("{:?}", corruption.rejection()).contains("ChecksumMismatch"));
}

#[test]
fn wal_gap_retains_the_exact_continuity_denial_and_frontier() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    publish_synthetic_checkpoint(&root, store);
    let families = root.join("families");
    for (segment, begin, end) in [(1, 2, 3), (3, 3, 4)] {
        let (path, bytes) =
            worth_store_test_support::harness::recovery::wal_tail::prepare_persisted_wal_frame(
                &families,
                segment,
                begin,
                end,
                "gap-frame",
                b"payload",
            );
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }
    let blocked = expect_blocked(
        admitted_recovery(&root)
            .discover()
            .unwrap()
            .select()
            .err()
            .expect("WAL gap must block"),
    );
    assert_eq!(blocked.evidence().lsn, Some(2));
    assert!(blocked
        .evidence()
        .source_denials
        .iter()
        .any(|denial| matches!(
            denial,
            PhysicalRecoverySourceDenial::WalTail(SelectedPhysicalWalTailDenial::SegmentGap)
        )));
}

fn root_case(name: &str, mutate: impl FnOnce(&std::path::Path)) -> PhysicalRecoveryBlock {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join(name);
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    mutate(&root);
    expect_blocked(
        admitted_recovery(&root)
            .discover()
            .unwrap()
            .select()
            .err()
            .expect("root denial must block"),
    )
}

fn checkpoint_case(
    name: &str,
    limits: PhysicalRecoveryLimits,
    write: impl FnOnce(
        &std::path::Path,
        worth_store_physical_format::store_namespace::StableStoreIdentity,
    ),
) -> PhysicalRecoveryBlock {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join(name);
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    let path = root.join("families").join("checkpoint.current");
    write(&path, store);
    expect_blocked(
        admitted_recovery_with_limits(&root, limits)
            .discover()
            .unwrap()
            .select()
            .err()
            .expect("checkpoint denial must block"),
    )
}

fn assert_checkpoint_denial(
    blocked: &PhysicalRecoveryBlock,
    expected: impl Fn(&PhysicalRecoveryCheckpointIntegrityDenial) -> bool,
) {
    assert!(blocked
        .evidence()
        .source_denials
        .iter()
        .any(|denial| match denial {
            PhysicalRecoverySourceDenial::CheckpointIntegrity(observed) => expected(observed),
            _ => false,
        }));
}

fn current_selector(root: &std::path::Path) -> std::path::PathBuf {
    root.join("families")
        .join("records")
        .join("root-current.selector")
}

fn write_checkpoint_with_two_dirty_records(
    path: &std::path::Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
) {
    let identity = PhysicalCheckpointIdentity::new(store, NonZeroU64::new(1).unwrap());
    let source = PhysicalCheckpointSource::concurrent(
        identity,
        CheckpointWalSourceRange::new(1, 2).unwrap(),
        CheckpointRootBasis::new(1, 7),
        1,
    );
    let (mut encoder, header) = CheckpointStreamEncoder::begin(source);
    let records = [1_u64, 2].map(|segment| {
        encoder.encode_dirty_basis(CheckpointDirtyFrameBasis::new(
            RecordFrameCoordinate::new(
                RecordArtifactFile::Segment {
                    segment,
                    generation: 1,
                },
                0,
                64,
            )
            .unwrap(),
            segment,
        ))
    });
    let cutover = CheckpointBindingCompactionHeader::new(1, 2).unwrap();
    let (compaction, cutover_record) = encoder.begin_binding_compaction(cutover);
    let (_, footer) = compaction.finish();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&header);
    for record in records {
        bytes.extend_from_slice(&record);
    }
    bytes.extend_from_slice(&cutover_record);
    bytes.extend_from_slice(&footer);
    std::fs::write(path, bytes).unwrap();
}
