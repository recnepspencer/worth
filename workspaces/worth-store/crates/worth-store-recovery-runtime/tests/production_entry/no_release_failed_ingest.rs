//! Genuine failed-ingest cleanup can coexist with selected NoRelease custody.

use super::*;
use worth_store::physical_runtime::BlobTerminalLimits;
use worth_store_physical_format::{
    BlobRecordKind, CurrentPhysicalRecordPlacement, RecordArtifactFile, SelectedRecordContentClass,
};
use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, WorthStoreRecovery};

const CHUNK: usize = 64 << 10;

#[test]
fn selected_no_release_with_failed_ingest_drop_opens_serving() {
    exercise(false, false);
}

#[test]
fn selected_tier_no_release_with_failed_ingest_drop_opens_serving() {
    exercise(true, false);
}

#[test]
fn changed_failed_ingest_control_after_seal_denies_serving_open() {
    exercise(false, true);
}

fn exercise(tier: bool, mutate_after_seal: bool) {
    let world = initialized_recovery_world(if tier {
        "no-release-failed-ingest-tier"
    } else {
        "no-release-failed-ingest"
    });
    let scope = admitted_blob_scope(if tier {
        "c11.recovery.no-release.failed.tier.scope"
    } else {
        "c11.recovery.no-release.failed.scope"
    });
    let blobs = world.serving().blobs().expect("blob owner");
    let limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(limits).expect("object identity");
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, world.placement(), CHUNK as u64, limits)
        .expect("begin failed ingest");
    ingest.push(&[0x51; CHUNK]).expect("selected chunk");
    let token = ingest.resume_token();
    drop(ingest);
    blobs
        .abort_ingest(
            token,
            &scope,
            world.placement(),
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
            BlobTerminalLimits::new(NonZeroU64::new(128).unwrap()),
        )
        .expect("selected failure terminal");
    let receipt = blobs
        .reclaim(BlobReclaimRequest::abandoned(
            token,
            &scope,
            world.placement(),
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
            BlobReclaimLimits::new(
                NonZeroU64::new(128).unwrap(),
                NonZeroU64::new(8 << 20).unwrap(),
                NonZeroU16::new(1).unwrap(),
            )
            .unwrap(),
        ))
        .expect("failed-ingest reclaim admission")
        .wait()
        .expect("failed-ingest drop");
    assert_eq!(receipt.disposition(), BlobReclaimDisposition::Dropped);
    assert_eq!(receipt.dropped_records().len(), 1);
    drop(blobs);
    if tier {
        world
            .serving()
            .certification_activate_tier_epoch(world.placement())
            .expect("tier activation");
    }
    let checkpoint = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(if tier { [0xb1; 32] } else { [0xb2; 32] }),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(checkpoint).into_raw()
    else {
        panic!("selected NoRelease checkpoint admission")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);

    let worker = std::thread::Builder::new()
        .name("no-release-failed-ingest-recovery".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let outcome = WorthStoreRecovery::certification_recover_with_custody_pauses(
                super::certified_release_serving::request(&root),
                |_| {},
                || {},
            );
            let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("selected failed-ingest NoRelease did not rejoin Store: {outcome:?}")
            };
            let control = handoff
                .selected_sources()
                .page_facts()
                .placements()
                .iter()
                .copied()
                .find(|route| {
                    matches!(
                        route.content_class(),
                        SelectedRecordContentClass::Blob(
                            BlobRecordKind::ReclaimDescriptor | BlobRecordKind::ReclaimDescriptorV2
                        )
                    )
                })
                .expect("genuine selected failed-ingest descriptor");
            let seal = handoff
                .into_core()
                .into_checkpoint_custody()
                .expect("positive NoRelease serving seal");
            if mutate_after_seal {
                alter_selected_control(&root, control);
                super::certified_release_serving::open_serving_with_seal_expect_mismatch(
                    &root, seal,
                );
            } else {
                super::certified_release_serving::open_serving_with_seal(&root, seal);
            }
        })
        .expect("recovery worker");
    worker.join().expect("recovery worker");
}

fn alter_selected_control(root: &Path, route: CurrentPhysicalRecordPlacement) {
    use std::io::{Read, Seek, SeekFrom, Write};
    let CurrentPhysicalRecordPlacement::Extent(extent) = route else {
        panic!("selected failed-ingest descriptor must be an extent")
    };
    let artifact = RecordArtifactFile::ExtentArena {
        arena: extent.arena_range().arena().get(),
    };
    let path = root
        .join("families/records/arenas")
        .join(artifact.file_name());
    let mut handle = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("selected control arena");
    let offset = extent.arena_range().offset();
    handle.seek(SeekFrom::Start(offset)).unwrap();
    let mut byte = [0];
    handle.read_exact(&mut byte).unwrap();
    byte[0] ^= 1;
    handle.seek(SeekFrom::Start(offset)).unwrap();
    handle.write_all(&byte).unwrap();
    handle.sync_all().unwrap();
}
