//! A selected V2 source closure has one recovery-memory budget, not a fresh
//! full-sized allowance for each roster and media observation owner.

use std::{fs, io::ErrorKind, path::Path};

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{
    PhysicalRecoveryRejoinResidentDenial, RecordByteLimit, RecordCountLimit, RecordScanOutcome,
    RecordScanRequest, RecoveredPhysicalRuntimeConstructionDenial, RecoveryDiscoveryArtifact,
    ServingPhysicalRuntime,
};
use worth_store_physical_format::{
    decode_blob_record, BlobRecordV1, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    DurableRootSelector, PersistedRecordIdentity, RecordArtifactFile,
    ReleaseCheckpointAccumulatorV2, ReleaseCustodyHeadEntryV1,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDimension, PhysicalRecoveryOutcome,
    WorthStoreRecovery,
};

use super::*;

#[test]
fn selected_release_source_memory_denies_before_effects_and_sufficient_twin_opens_serving() {
    let (world, first, _) = release_reopen::released_world(1);
    assert!(
        first.remaining_payload_records() > 0,
        "partial released head remains current"
    );
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0xe1; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("actual release checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    let checkpoint_bytes =
        fs::read(root.join("families/checkpoint.current")).expect("selected release checkpoint");
    let (_, accumulator) =
        release_reopen::selected_release_certificates_from_bytes(&checkpoint_bytes);
    let heads = release_reopen::selected_head_oracle::selected_heads(&root, accumulator);
    let [head] = heads.as_slice() else {
        panic!("fixture must select exactly one genuine released head")
    };
    let selected_control_payload_bytes = selected_control_payload_bytes(world.serving(), *head);
    let head_roster_payload_lower_bound = (heads.len() as u64)
        .checked_mul(std::mem::size_of::<ReleaseCustodyHeadEntryV1>() as u64)
        .expect("actual selected head roster fits in u64");
    let root_payload_lower_bound =
        selected_source_root_payload_lower_bound(&root, &checkpoint_bytes, accumulator);
    drop(world);

    let worker = std::thread::Builder::new()
        .name("v2-source-memory-recovery".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let current = selector(&root, RecordArtifactFile::CurrentRootSelector);
            let previous = selector(&root, RecordArtifactFile::PreviousRootSelector);
            let checkpoint = fs::read(root.join("families/checkpoint.current"))
                .expect("selected certified checkpoint");
            let outcome = WorthStoreRecovery::recover(
                super::recovery_request::request_with_memory(&root, 512 << 10),
            );
            let PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
                panic!("bounded V2 source closure must block: {outcome:?}")
            };
            assert_eq!(blocked.kind, PhysicalRecoveryBlockKind::SelectedCustody);
            assert_eq!(blocked.recovery_effects(), 0);
            let evidence = blocked.evidence();
            let limit = evidence.limit.unwrap_or_else(|| {
                panic!("source-custody denial must preserve its typed memory cause: {evidence:?}")
            });
            assert_eq!(
                limit.dimension,
                PhysicalRecoveryLimitDimension::RecoveryMemoryBytes
            );
            assert_eq!(limit.admitted, 512 << 10);
            assert!(limit.observed > limit.admitted);
            assert!(
                matches!(evidence.planning_denial,
                    Some(worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::SelectedReleaseHead(
                        worth_store_recovery_runtime::PhysicalRecoverySelectedReleaseHeadDenial::ResidentBoundExceeded { .. }
                    ))),
                "must be the resident admission boundary, not a later plan-cost denial: {evidence:?}"
            );
            assert_eq!(evidence.artifact.as_deref(), Some("checkpoint-source-release-head-v2"));
            assert_eq!(
                selector(&root, RecordArtifactFile::CurrentRootSelector),
                current
            );
            assert_eq!(
                selector(&root, RecordArtifactFile::PreviousRootSelector),
                previous
            );
            assert_eq!(
                fs::read(root.join("families/checkpoint.current")).unwrap(),
                checkpoint,
            );

            let outcome = WorthStoreRecovery::recover(super::request(&root));
            let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("same selected media must recover with sufficient memory: {outcome:?}")
            };
            assert_eq!(handoff.core().recovery_effect_count(), 0);
            let store_rejoin_retained = handoff
                .store_rejoin_retained_bytes()
                .expect("successful recovery measured live Store-entry backing");
            let planning_peak = handoff.planning_counters().peak_recovery_bytes();
            drop(handoff);
            assert_eq!(
                selector(&root, RecordArtifactFile::CurrentRootSelector),
                current
            );
            assert_eq!(
                selector(&root, RecordArtifactFile::PreviousRootSelector),
                previous
            );
            assert_eq!(
                fs::read(root.join("families/checkpoint.current")).unwrap(),
                checkpoint,
            );

            let wal_files = fs::read_dir(root.join("families/wal"))
                .expect("genuine selected WAL directory")
                .map(|entry| entry.expect("selected WAL entry"))
                .filter(|entry| entry.file_type().expect("selected WAL entry type").is_file())
                .collect::<Vec<_>>();
            assert_eq!(wal_files.len(), 1, "fixture must have one real selected WAL file");
            let wal_name = wal_files[0].file_name();
            let wal_bytes = wal_files[0].metadata().expect("selected WAL metadata").len();
            let requested = usize::try_from(wal_bytes).expect("selected WAL bytes fit usize");
            assert!(requested > 0);
            let store_tight_limit = store_rejoin_retained
                .checked_add(wal_bytes - 1)
                .expect("measured Store-entry backing and WAL payload fit in u64");
            assert!(store_tight_limit > planning_peak);
            assert!(store_tight_limit < 16 << 20);
            let outcome = WorthStoreRecovery::recover(
                super::recovery_request::request_with_memory(&root, store_tight_limit),
            );
            let PhysicalRecoveryOutcome::PublicationIndeterminate(indeterminate) = outcome else {
                panic!("Store WAL payload allocation must deny after C8 admission: {outcome:?}")
            };
            assert_eq!(indeterminate.recovery_effects(), 0);
            let Some(RecoveredPhysicalRuntimeConstructionDenial::RejoinResidentRead {
                artifact,
                offset,
                requested: observed_requested,
                cause,
            }) = indeterminate.handoff_failure() else {
                panic!("genuine Store WAL read must retain its typed resident denial: {indeterminate:?}")
            };
            assert_eq!(artifact, RecoveryDiscoveryArtifact::WalArtifact(wal_name));
            assert_eq!(offset, 0);
            assert_eq!(observed_requested, requested);
            let PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted } = cause
            else {
                panic!("genuine Store WAL read must deny its resident budget: {cause:?}")
            };
            assert_eq!(admitted, store_tight_limit);
            assert!(required > store_rejoin_retained + wal_bytes);
            let witnessed_payload_floor = store_rejoin_retained
                .checked_add(wal_bytes)
                .and_then(|bytes| bytes.checked_add(root_payload_lower_bound))
                .and_then(|bytes| bytes.checked_add(selected_control_payload_bytes))
                .and_then(|bytes| bytes.checked_add(head_roster_payload_lower_bound))
                .expect("genuine selected media payload lower bound fits u64");
            assert!(
                required >= witnessed_payload_floor,
                "Store WAL denial must account for its live selected root, head roster and all three control payloads"
            );
            assert_eq!(
                selector(&root, RecordArtifactFile::CurrentRootSelector),
                current
            );
            assert_eq!(
                selector(&root, RecordArtifactFile::PreviousRootSelector),
                previous
            );
            assert_eq!(
                fs::read(root.join("families/checkpoint.current")).unwrap(),
                checkpoint,
            );

            let outcome = WorthStoreRecovery::recover(super::request(&root));
            let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("same V2 media must recover after Store budget denial: {outcome:?}")
            };
            let seal = handoff
                .into_core()
                .into_checkpoint_custody()
                .expect("sufficient V2 source admission yields checkpoint seal");
            super::open_serving_with_seal(&root, seal);
        })
        .expect("bounded recovery worker");
    worker
        .join()
        .expect("bounded recovery worker did not panic");
}

fn selector(root: &Path, artifact: RecordArtifactFile) -> Option<Vec<u8>> {
    let path = root.join("families/records").join(artifact.file_name());
    match fs::read(path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => panic!("selected selector read failed: {error}"),
    }
}

fn selected_source_root_payload_lower_bound(
    root: &Path,
    checkpoint: &[u8],
    accumulator: ReleaseCheckpointAccumulatorV2,
) -> u64 {
    let records = root.join("families/records");
    let selector_bytes =
        fs::read(records.join(RecordArtifactFile::CurrentRootSelector.file_name()))
            .expect("selected root selector");
    let selector = DurableRootSelector::decode(&selector_bytes).expect("selected selector frame");
    let root_bytes = fs::read(
        records.join("roots").join(
            RecordArtifactFile::RootManifest {
                generation: selector.root_generation(),
            }
            .file_name(),
        ),
    )
    .expect("selected root manifest");
    let (selected, format) =
        DurablePhysicalRootManifest::decode(&root_bytes, u16::MAX).expect("selected root frame");
    assert_eq!(format, selector.format());
    assert_eq!(selected.generation(), accumulator.base().root_generation());
    assert_eq!(
        <[u8; 32]>::from(Sha256::digest(&root_bytes)),
        accumulator.base().root_sha256(),
        "fixture checkpoint source and selected root must be the same media frame"
    );
    let free_bytes = fs::read(
        records.join("free-space").join(
            RecordArtifactFile::FreeSpaceManifest {
                generation: selected.generation(),
            }
            .file_name(),
        ),
    )
    .expect("selected free-space header");
    let (_, free_format) =
        DurableFreeSpaceManifestHeader::decode(&free_bytes, selected.node_capacity())
            .expect("selected free-space header frame");
    assert_eq!(free_format, format);
    u64::try_from(selector_bytes.len() + checkpoint.len() + free_bytes.len())
        .unwrap()
        .checked_add(u64::try_from(root_bytes.len()).unwrap() * 2)
        .expect("genuine root and checkpoint payloads fit u64")
}

fn selected_control_payload_bytes(
    serving: &ServingPhysicalRuntime,
    head: ReleaseCustodyHeadEntryV1,
) -> u64 {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            RecordScanRequest::from_start()
                .with_batch_limit(RecordCountLimit::new(8).unwrap())
                .with_payload_limit(RecordByteLimit::new(512 << 10).unwrap()),
        )
        .unwrap();
    let mut scratch = vec![0_u8; 512 << 10];
    let mut descriptor = None;
    let mut reservation = None;
    let mut manifest = None;
    while let RecordScanOutcome::Batch(records) = scan.read_next_into(&mut scratch).unwrap() {
        for index in 0..records.records().len() {
            let Some(bytes) = records.payload(index) else {
                continue;
            };
            let record = records.records()[index].record_id();
            let record = PersistedRecordIdentity::new(record.allocation_epoch(), record.ordinal())
                .expect("identity observed through real Serving scan");
            match decode_blob_record(bytes) {
                Ok(BlobRecordV1::ReclaimDescriptorV3(value)) => {
                    assert!(descriptor
                        .replace((
                            record,
                            bytes.len(),
                            value,
                            <[u8; 32]>::from(Sha256::digest(bytes))
                        ))
                        .is_none());
                }
                Ok(BlobRecordV1::OriginalDropReserved(value)) => {
                    assert!(reservation
                        .replace((
                            record,
                            bytes.len(),
                            value,
                            <[u8; 32]>::from(Sha256::digest(bytes))
                        ))
                        .is_none());
                }
                Ok(BlobRecordV1::DropSetManifestV3(_)) => {
                    assert!(manifest
                        .replace((record, bytes.len(), <[u8; 32]>::from(Sha256::digest(bytes))))
                        .is_none());
                }
                _ => {}
            }
        }
        if records.is_complete() {
            break;
        }
    }
    let (descriptor_record, descriptor_bytes, descriptor, descriptor_sha) =
        descriptor.expect("one selected genuine V3 descriptor");
    let (reservation_record, reservation_bytes, reservation, reservation_sha) =
        reservation.expect("one selected genuine reservation");
    let (manifest_record, manifest_bytes, manifest_sha) =
        manifest.expect("one selected genuine V3 manifest");
    assert_eq!(
        (descriptor_record, descriptor_sha),
        (head.descriptor_record(), head.descriptor_frame_sha256())
    );
    assert_eq!(
        (reservation_record, reservation_sha),
        (head.reservation_record(), head.reservation_frame_sha256())
    );
    assert_eq!(
        (manifest_record, manifest_sha),
        (head.manifest_record(), head.manifest_frame_sha256())
    );
    assert_eq!(
        (manifest_record, manifest_sha),
        (
            descriptor.base().manifest_record(),
            descriptor.base().manifest_frame_sha256()
        )
    );
    assert_eq!(reservation.manifest_record(), manifest_record);
    assert_eq!(reservation.manifest_frame_sha256(), manifest_sha);
    u64::try_from(descriptor_bytes + reservation_bytes + manifest_bytes)
        .expect("three genuine selected control payloads fit u64")
}
