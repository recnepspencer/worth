#[allow(unused_imports)]
use crate::phase_three_denials::*;

use worth_store_physical_format::ROOT_SELECTOR_BYTES;
use {
    worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery,
    worth_store_testkit_recovery_runtime::phase_three_support::expect_blocked,
    worth_store_testkit_recovery_runtime::phase_three_support::initialize_store,
    worth_store_testkit_recovery_runtime::phase_three_support::limit_declaration,
    worth_store_testkit_recovery_runtime::phase_three_support::limits,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_checkpoint,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_genesis,
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
    assert_eq!(
        blocked.cause().damage(),
        Some(worth_store_recovery_runtime::PhysicalRecoveryBlockKind::RootProtocol)
    );
    assert_eq!(blocked.store_identity(), primary_store);
    assert_eq!(blocked.evidence().counters.current_root_rejected, 1);
    assert!(matches!(
        blocked.evidence().source_denials.as_slice(),
        [
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
                denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Integrity(_),
            },
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
                denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Absent,
            },
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootSlot {
                slot: worth_store_physical_format::RootSelectorRole::Current,
                denial: worth_store_recovery_physics::PhysicalRootCandidateDenial::SelectorAuthorityMismatch,
                observed_store: None,
                observed_role: None,
                observed_generation: None,
            },
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootSelection(
                worth_store_recovery_physics::PhysicalRootSelectionDenial::NoAdmittedRoot
            )
        ]
    ));
    assert_eq!(blocked.recovery_effects(), 0);
}

#[test]
fn torn_current_retains_unlinked_previous_selection_cause() {
    let blocked = root_case("unlinked-previous", |root| {
        let current = worth_store_physical_format::DurableRootSelector::decode(
            &std::fs::read(current_selector(root)).unwrap(),
        )
        .unwrap();
        let previous = worth_store_physical_format::DurableRootSelector::new(
            current.store_identity(),
            current.format(),
            worth_store_physical_format::RootSelectorIdentity::new(2).unwrap(),
            worth_store_physical_format::RootSelectorRole::Previous,
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
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::CurrentSelector,
                denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Integrity(_),
            },
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootSlot {
                slot: worth_store_physical_format::RootSelectorRole::Current,
                denial: worth_store_recovery_physics::PhysicalRootCandidateDenial::SelectorIntegrity,
                ..
            },
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootSelection(
                worth_store_recovery_physics::PhysicalRootSelectionDenial::PreviousFallbackUnlinked
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
            worth_store_recovery_runtime::PhysicalRecoveryCheckpointIntegrityDenial::Integrity(_)
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
            worth_store_recovery_runtime::PhysicalRecoveryCheckpointIntegrityDenial::Integrity(_)
        )
    });

    let mut declaration = limit_declaration(2, 8, 8 * 1024);
    declaration.dirty_frames = 1;
    let count_limits =
        worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(declaration).unwrap();
    let count = checkpoint_case("record-count", count_limits, |path, store| {
        write_checkpoint_with_two_dirty_records(path, store);
    });
    assert_checkpoint_denial(&count, |denial| {
        matches!(
            denial,
            worth_store_recovery_runtime::PhysicalRecoveryCheckpointIntegrityDenial::DirtyRecordLimit
        )
    });
    let limit = count
        .cause()
        .limit()
        .expect("the dirty-frame count is a limit");
    assert_eq!(
        (
            count.cause().phase(),
            limit.dimension(),
            limit.observed(),
            limit.admitted()
        ),
        (
            worth_store_recovery_runtime::PhysicalRecoveryBlockKind::Checkpoint,
            worth_store_recovery_runtime::PhysicalRecoveryLimitDimension::DirtyFrames,
            2,
            1
        )
    );
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
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::WalIntegrity(
                corruption,
            ) => Some(corruption),
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
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::WalTail(
                worth_store_recovery_physics::SelectedPhysicalWalTailDenial::SegmentGap
            )
        )));
}
