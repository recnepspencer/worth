//! Reopens a real namespace-durable group before its in-memory root advance.

#[allow(unused_imports)]
use crate::physical_record_journeys::maintenance_interference::publication_charge::cutover::*;
use std::collections::BTreeMap;
use std::path::Path;
use {std::num::NonZeroU32, std::num::NonZeroU64};

use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::PhysicalDataSettlementOutcome,
    worth_store::physical_runtime::PhysicalDurabilityStateReopenFailure,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalReadProtectionPolicy,
    worth_store::physical_runtime::PhysicalRecordAccessPolicy,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
    worth_store::physical_runtime::PhysicalRootNamespaceDurabilityOutcome,
    worth_store::physical_runtime::PhysicalRootPublicationPreparationOutcome,
    worth_store::physical_runtime::PhysicalRootReplacementOutcome,
    worth_store::physical_runtime::PhysicalSignalConstructionFailure,
    worth_store::physical_runtime::PhysicalWalGroupAppendOutcome,
    worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome,
    worth_store::physical_runtime::PhysicalWalPolicy,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordBootstrapFailure,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordCountLimit,
    worth_store::physical_runtime::RecordReadLimits,
    worth_store::physical_runtime::RecordScanOutcome,
    worth_store::physical_runtime::RecordScanRequest,
    worth_store::physical_runtime::ServingPhysicalRuntime,
    worth_store::physical_runtime::WalSegmentByteLimit,
    worth_store::physical_runtime::WalSegmentInventoryLimit,
};

use {
    crate::physical_record_journeys::maintenance_interference::publication_charge::prepared,
    crate::physical_record_journeys::maintenance_interference::publication_charge::require_dispatched,
    crate::physical_record_journeys::maintenance_interference::publication_charge::EXTENT_SEED,
    worth_store_testkit_store::publication_metadata_oracle as oracle,
};

#[test]
fn conflicting_complete_groups_reject_reopen_before_owner_installation() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CONFLICT_CHILD_NAME, "--nocapture"])
        .env(CONFLICT_CHILD_ROOT, &root)
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(74),
        "child must exit after one root selects two complete WAL groups; stdout: {}; stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::fs::read(parent.path().join("conflicting-groups.marker")).unwrap(),
        b"selected=2;complete-groups=2"
    );
    let before = file_inventory(&root);
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .admit(format)
        .unwrap();
    let media = worth_store_testkit_store::physical_record_journeys::media(&root);
    let durability =
        worth_store_testkit_store::physical_record_journeys::durability_with_wal_policy(
            &media,
            worth_store::physical_runtime::PhysicalWalPolicy::segmented(
                worth_store::physical_runtime::WalSegmentByteLimit::new(
                    std::num::NonZeroU64::new(512 * 1024).unwrap(),
                ),
                worth_store::physical_runtime::WalSegmentInventoryLimit::new(
                    std::num::NonZeroU32::new(64).unwrap(),
                ),
            ),
        );
    let result = media
        .open_record_store(
            worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
                .with_residency_policy(worth_store_testkit_store::physical_record_journeys::maintenance_interference::residency(
                    format,
                    worth_store_testkit_store::physical_record_journeys::maintenance_interference::RESIDENT_BYTES,
                ))
                .with_read_protection_policy(worth_store::physical_runtime::PhysicalReadProtectionPolicy::default()),
        )
        .into_raw();
    match result {
        worth_proof::TransitionOutcome::Failed(failure) => assert!(matches!(
            failure.cause(),
            worth_store::physical_runtime::RecordBootstrapFailure::SignalConstruction(
                worth_store::physical_runtime::PhysicalSignalConstructionFailure::DurabilityStateReopenRejected(
                    worth_store::physical_runtime::PhysicalDurabilityStateReopenFailure::PublicationRetentionRejected
                )
            )
        )),
        _ => panic!("conflicting published-root attribution must reject before serving"),
    }
    let after = file_inventory(&root);
    assert_eq!(
        after.keys().collect::<Vec<_>>(),
        before.keys().collect::<Vec<_>>()
    );
    for (path, bytes) in before {
        assert!(
            after[&path] == bytes,
            "denied reopen changed {}",
            path.display()
        );
    }
}

#[test]
fn conflicting_complete_groups_child() {
    let Ok(root) = std::env::var(CONFLICT_CHILD_ROOT) else {
        return;
    };
    publish_conflicting_complete_groups(std::path::Path::new(&root));
}

#[test]
fn namespace_durable_group_reopens_without_memory_advance() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_NAME, "--nocapture"])
        .env(CHILD_ROOT, &root)
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(73),
        "the child must exit only after durable namespace synchronization; stdout: {}; stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::fs::read(parent.path().join("namespace-durable.marker")).unwrap(),
        b"generation=3"
    );

    let format = worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
        .admit()
        .unwrap();
    let expected = worth_store_testkit_store::publication_metadata_oracle::wal_file_bytes(&root)
        + worth_store_testkit_store::publication_metadata_oracle::actual_publication_metadata(
            &root, 2, format,
        )
        + worth_store_testkit_store::publication_metadata_oracle::actual_publication_metadata(
            &root, 3, format,
        );
    let reopened = worth_store_testkit_store::physical_record_journeys::maintenance_interference::reopen::open_interference(&root);
    assert_eq!(
        reopened
            .observer()
            .acquisition_snapshot()
            .unwrap()
            .root_generation(),
        3
    );
    assert_eq!(reopened.certification_charged_growth_bytes(), expected);
    let records = read_selected_records(&reopened);
    assert_eq!(records.len(), 10);
    // Scans follow record identity, not append order.
    let count = |expected: &[u8]| {
        records
            .iter()
            .filter(|bytes| bytes.as_slice() == expected)
            .count()
    };
    assert_eq!(count(EXTENT_SEED.as_slice()), 8);
    assert_eq!(count(b"left"), 1);
    assert_eq!(count(b"right"), 1);
    reopened.close();
}

#[test]
fn namespace_durable_before_memory_advance_child() {
    let Ok(root) = std::env::var(CHILD_ROOT) else {
        return;
    };
    publish_durable_group_without_memory_advance(std::path::Path::new(&root));
}
