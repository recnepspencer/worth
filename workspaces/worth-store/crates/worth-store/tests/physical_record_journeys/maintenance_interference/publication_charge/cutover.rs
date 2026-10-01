//! Reopens a real namespace-durable group before its in-memory root advance.

use std::collections::BTreeMap;
use std::num::{NonZeroU32, NonZeroU64};
use std::path::Path;

use worth_proof::{NonEmpty, TransitionOutcome};
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, ManifestEntryCapacity, PhysicalDataSettlementOutcome,
    PhysicalDurabilityStateReopenFailure, PhysicalMutationIdempotencyMaterial,
    PhysicalReadProtectionPolicy, PhysicalRecordAccessPolicy, PhysicalRecordFormatDeclaration,
    PhysicalRecordInitialization, PhysicalRecordOpen, PhysicalRecordPlacementPolicy,
    PhysicalRootNamespaceDurabilityOutcome, PhysicalRootPublicationPreparationOutcome,
    PhysicalRootReplacementOutcome, PhysicalSignalConstructionFailure,
    PhysicalWalGroupAppendOutcome, PhysicalWalGroupBarrierOutcome, PhysicalWalPolicy,
    RecordAppendBatch, RecordBootstrapFailure, RecordByteLimit, RecordCountLimit, RecordReadLimits,
    RecordScanOutcome, RecordScanRequest, ServingPhysicalRuntime, WalSegmentByteLimit,
    WalSegmentInventoryLimit,
};

use super::{oracle, prepared, require_dispatched, EXTENT_SEED};

const CHILD_ROOT: &str = "WORTH_STORE_PUBLICATION_CUTOVER_CHILD_ROOT";
const CHILD_NAME: &str =
    "maintenance_interference::publication_charge::cutover::namespace_durable_before_memory_advance_child";
const CONFLICT_CHILD_ROOT: &str = "WORTH_STORE_PUBLICATION_CONFLICT_CHILD_ROOT";
const CONFLICT_CHILD_NAME: &str =
    "maintenance_interference::publication_charge::cutover::conflicting_complete_groups_child";

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
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let media = super::super::super::media(&root);
    let durability = super::super::super::durability_with_wal_policy(
        &media,
        PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(NonZeroU64::new(512 * 1024).unwrap()),
            WalSegmentInventoryLimit::new(NonZeroU32::new(64).unwrap()),
        ),
    );
    let result = media
        .open_record_store(
            PhysicalRecordOpen::new(format, access, durability)
                .with_residency_policy(super::super::residency(
                    format,
                    super::super::RESIDENT_BYTES,
                ))
                .with_read_protection_policy(PhysicalReadProtectionPolicy::default()),
        )
        .into_raw();
    match result {
        TransitionOutcome::Failed(failure) => assert!(matches!(
            failure.cause(),
            RecordBootstrapFailure::SignalConstruction(
                PhysicalSignalConstructionFailure::DurabilityStateReopenRejected(
                    PhysicalDurabilityStateReopenFailure::PublicationRetentionRejected
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
    publish_conflicting_complete_groups(Path::new(&root));
}

fn publish_conflicting_complete_groups(root: &Path) -> ! {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let placement = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(2).unwrap())
        .admit(format)
        .unwrap();
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let media = super::super::super::media(root);
    let durability = super::super::super::durability_with_wal_policy(
        &media,
        PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(NonZeroU64::new(512 * 1024).unwrap()),
            WalSegmentInventoryLimit::new(NonZeroU32::new(64).unwrap()),
        ),
    );
    let serving = super::super::super::success(
        media.initialize_record_store(
            PhysicalRecordInitialization::new(format, placement, access, durability)
                .with_residency_policy(super::super::residency(
                    format,
                    super::super::RESIDENT_BYTES,
                )),
        ),
    );
    let submission = serving.certification_record_submission();
    let first = prepared(&submission, placement, [221; 32], b"selected-first");
    let second = prepared(&submission, placement, [222; 32], b"unselected-second");
    let append_one =
        |member| match submission.append_prepared_wal_group(NonEmpty::new(member, Vec::new())) {
            PhysicalWalGroupAppendOutcome::Appended(value) => value,
            _ => panic!("complete competing group was not appended"),
        };
    let first = append_one(first);
    let second = append_one(second);
    let first_basis = first.basis();
    let first = match submission.synchronize_appended_wal_group(first) {
        PhysicalWalGroupBarrierOutcome::Durable(group) => {
            group.into_members().into_vec().pop().unwrap()
        }
        _ => panic!("selected group WAL was not durable"),
    };
    let second = match submission.synchronize_appended_wal_group(second) {
        PhysicalWalGroupBarrierOutcome::Durable(group) => group.into_members(),
        _ => panic!("competing group WAL was not durable"),
    };
    assert_eq!(second.as_slice().len(), 1);
    let completed = serving.certification_complete_dispatched_mutation(
        first_basis,
        require_dispatched(submission.dispatch_wal_durable_data(first)),
    );
    assert_eq!(completed.current_root().generation(), 2);
    std::fs::write(
        root.parent().unwrap().join("conflicting-groups.marker"),
        b"selected=2;complete-groups=2",
    )
    .unwrap();
    std::process::exit(74)
}

fn file_inventory(root: &Path) -> BTreeMap<std::path::PathBuf, Vec<u8>> {
    let mut inventory = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                pending.push(entry.path());
            } else {
                // Media admission replaces this ephemeral process-ownership
                // diagnostic; it is not a durable Store data artifact.
                if entry.path() == root.join("namespace/mutation.lock") {
                    continue;
                }
                inventory.insert(entry.path(), std::fs::read(entry.path()).unwrap());
            }
        }
    }
    inventory
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

    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let expected = oracle::wal_file_bytes(&root)
        + oracle::actual_publication_metadata(&root, 2, format)
        + oracle::actual_publication_metadata(&root, 3, format);
    let reopened = super::super::reopen::open_interference(&root);
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

fn read_selected_records(serving: &ServingPhysicalRuntime) -> Vec<Vec<u8>> {
    let records = serving.records().unwrap();
    let mut scan = serving
        .records()
        .unwrap()
        .scan(RecordScanRequest::from_start().with_batch_limit(RecordCountLimit::new(16).unwrap()))
        .unwrap();
    let mut scratch = vec![0; 64_000];
    let mut found = Vec::new();
    while let RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() {
        for entry in batch.records() {
            // Extent scan rows intentionally contain identity, not inline
            // payload. Read both forms through the ordinary protected reader.
            let mut read = records
                .open(
                    entry.record_id(),
                    RecordReadLimits::new(RecordByteLimit::new(20_000).unwrap()),
                )
                .unwrap();
            let mut bytes = Vec::new();
            while let Some(chunk) = read.next_chunk().unwrap() {
                bytes.extend_from_slice(chunk.bytes());
            }
            found.push(bytes);
        }
        if batch.is_complete() {
            break;
        }
    }
    found
}

#[test]
fn namespace_durable_before_memory_advance_child() {
    let Ok(root) = std::env::var(CHILD_ROOT) else {
        return;
    };
    publish_durable_group_without_memory_advance(Path::new(&root));
}

fn publish_durable_group_without_memory_advance(root: &Path) -> ! {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let placement = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(2).unwrap())
        .admit(format)
        .unwrap();
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let media = super::super::super::media(root);
    let durability = super::super::super::durability_with_wal_policy(
        &media,
        PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(NonZeroU64::new(512 * 1024).unwrap()),
            WalSegmentInventoryLimit::new(NonZeroU32::new(64).unwrap()),
        ),
    );
    let serving = super::super::super::success(
        media.initialize_record_store(
            PhysicalRecordInitialization::new(format, placement, access, durability)
                .with_residency_policy(super::super::residency(
                    format,
                    super::super::RESIDENT_BYTES,
                )),
        ),
    );
    let seeds = (0..8).map(|_| EXTENT_SEED.as_slice()).collect::<Vec<_>>();
    let seeded = super::super::super::durable_publication::publish_single(
        &serving,
        placement,
        PhysicalMutationIdempotencyMaterial::new([211; 32]),
        RecordAppendBatch::try_from_iter(seeds).unwrap(),
    );
    assert_eq!(seeded.current_root().generation(), 2);
    let submission = serving.certification_record_submission();
    let left = prepared(&submission, placement, [212; 32], b"left");
    let right = prepared(&submission, placement, [213; 32], b"right");
    let appended = match submission.append_prepared_wal_group(NonEmpty::new(left, vec![right])) {
        PhysicalWalGroupAppendOutcome::Appended(value) => value,
        _ => panic!("cutover group did not append"),
    };
    let basis = appended.basis();
    let durable = match submission.synchronize_appended_wal_group(appended) {
        PhysicalWalGroupBarrierOutcome::Durable(group) => group.into_members(),
        _ => panic!("cutover WAL was not durable"),
    };
    let settled = durable
        .into_vec()
        .into_iter()
        .map(|member| {
            match require_dispatched(submission.dispatch_wal_durable_data(member))
                .settle_exact_effects()
            {
                PhysicalDataSettlementOutcome::Settled(value) => value,
                PhysicalDataSettlementOutcome::InspectionRequired { cause, .. } => {
                    panic!("cutover data was not settled: {cause:?}")
                }
            }
        })
        .collect::<Vec<_>>();
    let joined = submission
        .join_data_settled_group(
            basis,
            NonEmpty::try_from_vec(settled).unwrap_or_else(|_| unreachable!("two settled members")),
        )
        .unwrap_or_else(|rejected| panic!("cutover group rejected: {:?}", rejected.cause()));
    let prepared_root = match submission.prepare_root_publication(joined) {
        PhysicalRootPublicationPreparationOutcome::Prepared(value) => value,
        PhysicalRootPublicationPreparationOutcome::NotStarted(failure) => {
            panic!(
                "cutover root preparation did not start: {:?}",
                failure.cause()
            )
        }
        PhysicalRootPublicationPreparationOutcome::InspectionRequired(failure) => {
            panic!(
                "cutover root preparation required inspection: {:?}",
                failure.cause()
            )
        }
    };
    let replaced = match submission.replace_prepared_root(prepared_root) {
        PhysicalRootReplacementOutcome::Replaced(value) => value,
        PhysicalRootReplacementOutcome::NotStarted(failure) => {
            panic!(
                "cutover root replacement did not start: {:?}",
                failure.cause()
            )
        }
        PhysicalRootReplacementOutcome::InspectionRequired(failure) => {
            panic!(
                "cutover root replacement required inspection: {:?}",
                failure.effect_fate()
            )
        }
    };
    let durable = match submission.synchronize_replaced_root_namespace(replaced) {
        PhysicalRootNamespaceDurabilityOutcome::Durable(value) => value,
        PhysicalRootNamespaceDurabilityOutcome::NotStarted(failure) => {
            panic!(
                "cutover namespace synchronization did not start: {:?}",
                failure.cause()
            )
        }
        PhysicalRootNamespaceDurabilityOutcome::InspectionRequired(failure) => {
            panic!(
                "cutover namespace synchronization required inspection: {:?}",
                failure.effect_fate()
            )
        }
    };
    assert_eq!(durable.current_root_generation(), 3);
    std::fs::write(
        root.parent().unwrap().join("namespace-durable.marker"),
        b"generation=3",
    )
    .unwrap();
    // Exit bypasses Store/lease Drop and intentionally never calls the in-memory
    // current-root advance. The parent reconstructs from the durable artifacts.
    std::process::exit(73)
}
