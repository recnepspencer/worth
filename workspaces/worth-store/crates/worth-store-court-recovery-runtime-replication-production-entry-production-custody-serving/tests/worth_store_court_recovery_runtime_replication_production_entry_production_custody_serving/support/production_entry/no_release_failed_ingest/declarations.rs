//! Genuine failed-ingest cleanup can coexist with selected NoRelease custody.

pub(crate) use worth_store::physical_runtime::BlobTerminalLimits;
pub(crate) use {
    std::num::NonZeroU16, std::num::NonZeroU64, std::path::Path, worth_proof::TransitionOutcome,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store_blob_chunks::BlobChunkSize,
    worth_store_testkit_recovery_runtime::production_entry::blob_scope_admission::admitted_blob_scope,
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving,
    worth_store_testkit_recovery_runtime::production_entry::initialized_recovery_world,
};
pub(crate) use {
    worth_store_physical_format::BlobRecordKind,
    worth_store_physical_format::CurrentPhysicalRecordPlacement,
    worth_store_physical_format::RecordArtifactFile,
    worth_store_physical_format::SelectedRecordContentClass,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
};

pub(crate) const CHUNK: usize = 64 << 10;

pub(crate) fn exercise(tier: bool, mutate_after_seal: bool) {
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
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(limits).expect("object identity");
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
            .unwrap(),
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
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
                .unwrap(),
            worth_store::physical_runtime::BlobTerminalLimits::new(
                std::num::NonZeroU64::new(128).unwrap(),
            ),
        )
        .expect("selected failure terminal");
    let receipt = blobs
        .reclaim(
            worth_store::physical_runtime::BlobReclaimRequest::abandoned(
                token,
                &scope,
                world.placement(),
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(
                    120_000,
                )
                .unwrap(),
                worth_store::physical_runtime::BlobReclaimLimits::new(
                    std::num::NonZeroU64::new(128).unwrap(),
                    std::num::NonZeroU64::new(8 << 20).unwrap(),
                    std::num::NonZeroU16::new(1).unwrap(),
                )
                .unwrap(),
            ),
        )
        .expect("failed-ingest reclaim admission")
        .wait()
        .expect("failed-ingest drop");
    assert_eq!(
        receipt.disposition(),
        worth_store::physical_runtime::BlobReclaimDisposition::Dropped
    );
    assert_eq!(receipt.dropped_records().len(), 1);
    if tier {
        world
            .serving()
            .certification_activate_tier_epoch(world.placement())
            .expect("tier activation");
    }
    let checkpoint = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new(if tier {
            [0xb1; 32]
        } else {
            [0xb2; 32]
        }),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(checkpoint).into_raw()
    else {
        panic!("selected NoRelease checkpoint admission")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);

    let worker = std::thread::Builder::new()
        .name("no-release-failed-ingest-recovery".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let outcome = worth_store_recovery_runtime::WorthStoreRecovery::certification_recover_with_custody_pauses(
                worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(&root),
                |_| {},
                || {},
            );
            let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
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
                        worth_store_physical_format::SelectedRecordContentClass::Blob(
                            worth_store_physical_format::BlobRecordKind::ReclaimDescriptor | worth_store_physical_format::BlobRecordKind::ReclaimDescriptorV2
                        )
                    )
                })
                .expect("genuine selected failed-ingest descriptor");
            let seal = handoff
                .into_core()
                .certification_release_media_into_checkpoint_custody()
                .expect("positive NoRelease serving seal");
            if mutate_after_seal {
                alter_selected_control(&root, control);
                worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal_expect_mismatch(
                    &root, seal,
                );
            } else {
                worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal(&root, seal);
            }
        })
        .expect("recovery worker");
    worker.join().expect("recovery worker");
}

pub(crate) fn alter_selected_control(root: &Path, route: CurrentPhysicalRecordPlacement) {
    use {std::io::Read, std::io::Seek, std::io::SeekFrom, std::io::Write};
    let worth_store_physical_format::CurrentPhysicalRecordPlacement::Extent(extent) = route else {
        panic!("selected failed-ingest descriptor must be an extent")
    };
    let artifact = worth_store_physical_format::RecordArtifactFile::ExtentArena {
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
