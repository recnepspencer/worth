//! A real failed-ingest V1 control survives a NoRelease checkpoint, two
//! distinct released-generation V3 drops, and an ordinary selected tail.
//! Store must rejoin that otherwise unrelated selected control independently.

use std::{
    fs,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use super::*;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, ManifestEntryCapacity, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationOutcome, PhysicalMutationPreparationSuccess, PhysicalMutationRequest,
    PhysicalRecordFormatDeclaration, PhysicalRecordPlacementPolicy, RecordAppendBatch,
};
use worth_store_physical_format::{
    decode_blob_record, decode_extent_chunk, encode_extent_chunk, BlobReclaimDescriptorV1,
    BlobRecordKind, BlobRecordV1, CurrentPhysicalRecordPlacement, DurableExtentManifest,
    DurableRootSelector, ExtentArenaFrameLayout, ExtentChunkCoordinate, RecordArtifactFile,
    SelectedRecordContentClass, DURABLE_EXTENT_FRAME_HEADER_BYTES,
    EXTENT_ARENA_MANIFEST_FRAME_BYTES, EXTENT_CHUNK_METADATA_BYTES,
};
use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, WorthStoreRecovery};

#[test]
fn mixed_v1_two_v3_historical_rejoin_opens_then_resealed_v1_denies_serving() {
    let world = pending_wal_world::first_with_failed_ingest_control();
    world.kill_distinct_release_before_checkpoint();
    let checkpoint = fs::read(world.root().join("families/checkpoint.current")).unwrap();
    let first = recover(world.root());
    let seal = first
        .into_core()
        .into_checkpoint_custody()
        .expect("two real V3 drops must receive Store seal");
    let serving = certified_release_serving::admit_serving_with_seal(world.root(), seal);
    let before_tail = selected_generation(world.root());
    append_ordinary_tail(&serving);
    assert!(
        selected_generation(world.root()) > before_tail,
        "ordinary publication must advance the selected root after both V3 drops"
    );
    serving.close();
    assert_eq!(
        fs::read(world.root().join("families/checkpoint.current")).unwrap(),
        checkpoint,
        "NoRelease checkpoint must remain selected across both V3s and ordinary tail"
    );

    let fresh = recover(world.root());
    let routes = fresh.selected_sources().page_facts().placements();
    assert_eq!(
        routes
            .iter()
            .filter(|route| {
                route.content_class()
                    == SelectedRecordContentClass::Blob(BlobRecordKind::ReclaimDescriptorV3)
            })
            .count(),
        2,
        "both genuine V3 results must remain selected"
    );
    assert!(
        routes.iter().any(|route| {
            route.content_class()
                == SelectedRecordContentClass::Blob(BlobRecordKind::DropSetManifestV2)
        }),
        "genuine failed-ingest V2 manifest must remain selected"
    );
    let descriptor = routes
        .iter()
        .copied()
        .find(|route| {
            route.content_class()
                == SelectedRecordContentClass::Blob(BlobRecordKind::ReclaimDescriptor)
        })
        .expect("failed-ingest V1 descriptor must survive the ordinary selected tail");
    let original_frame = read_v1_descriptor_frame(world.root(), descriptor);
    let seal = fresh
        .into_core()
        .into_checkpoint_custody()
        .expect("historical-only mixed custody must seal");
    certified_release_serving::open_serving_with_seal_without_checkpoint(world.root(), seal);

    let negative = recover(world.root());
    let seal = negative
        .into_core()
        .into_checkpoint_custody()
        .expect("unchanged mixed media must receive a second genuine Store seal");
    reseal_v1_descriptor(world.root(), descriptor, &original_frame);
    certified_release_serving::open_serving_with_seal_expect_mismatch(world.root(), seal);
}

fn recover(root: &Path) -> worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff {
    let outcome = WorthStoreRecovery::recover(certified_release_serving::request(root));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        if let PhysicalRecoveryOutcome::PublicationIndeterminate(failure) = &outcome {
            panic!("mixed current failed-ingest + two V3 rejoin failed: handoff={:?}; reopen={:?}; effects={}",
                failure.handoff_failure(), failure.reopen_failure(), failure.recovery_effects());
        }
        panic!("mixed V1 + two V3 C8/Store custody must recover: {outcome:?}");
    };
    handoff
}

fn append_ordinary_tail(serving: &worth_store::physical_runtime::ServingPhysicalRuntime) {
    let placement = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
        .admit(AdmittedPhysicalRecordFormat::admit(
            PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        ))
        .unwrap();
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0xe7; 32]))
        .unwrap();
    let TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .prepare_durable_append(
                RecordAppendBatch::try_from_iter(
                    [b"mixed-v1-v3-ordinary-selected-tail".as_slice()],
                )
                .unwrap(),
                placement,
                PhysicalMutationRequest::platform_durable(
                    key,
                    PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                ),
            )
            .into_raw()
    else {
        panic!("ordinary tail after mixed V1 and two V3s must prepare");
    };
    assert!(matches!(
        prepared.execute(),
        PhysicalMutationOutcome::Completed(_)
    ));
}

fn read_v1_descriptor_frame(root: &Path, route: CurrentPhysicalRecordPlacement) -> Vec<u8> {
    let CurrentPhysicalRecordPlacement::Extent(extent) = route else {
        panic!("retained failed-ingest V1 descriptor must use an extent route");
    };
    let path = root.join("families/records/arenas").join(
        RecordArtifactFile::ExtentArena {
            arena: extent.arena_range().arena().get(),
        }
        .file_name(),
    );
    let mut file = fs::File::open(path).expect("selected V1 control arena");
    file.seek(SeekFrom::Start(extent.arena_range().offset()))
        .unwrap();
    let mut manifest_bytes = [0; EXTENT_ARENA_MANIFEST_FRAME_BYTES];
    file.read_exact(&mut manifest_bytes).unwrap();
    let (manifest, format) = DurableExtentManifest::decode(&manifest_bytes).unwrap();
    assert_eq!(manifest.record(), extent.record());
    assert_eq!(manifest.extent_cell(), extent.extent_cell());
    assert_eq!(manifest.chunk_count(), 1);
    let layout = ExtentArenaFrameLayout::new(format, manifest.alignment()).unwrap();
    let offset = extent.arena_range().offset() + layout.chunk_offset(1).unwrap();
    file.seek(SeekFrom::Start(offset)).unwrap();
    let mut frame = vec![
        0;
        DURABLE_EXTENT_FRAME_HEADER_BYTES
            + EXTENT_CHUNK_METADATA_BYTES
            + manifest.logical_bytes() as usize
    ];
    file.read_exact(&mut frame).unwrap();
    let coordinate = ExtentChunkCoordinate::new(
        extent.record(),
        extent.extent_cell(),
        manifest.logical_bytes(),
        0,
        1,
    )
    .unwrap();
    let (payload, observed_format) = decode_extent_chunk(&frame, coordinate).unwrap();
    assert_eq!(observed_format, format);
    assert!(matches!(
        decode_blob_record(payload),
        Ok(BlobRecordV1::ReclaimDescriptor(_))
    ));
    frame
}

fn reseal_v1_descriptor(root: &Path, route: CurrentPhysicalRecordPlacement, original: &[u8]) {
    let CurrentPhysicalRecordPlacement::Extent(extent) = route else {
        panic!("retained V1 route changed after C8 claim");
    };
    let path = root.join("families/records/arenas").join(
        RecordArtifactFile::ExtentArena {
            arena: extent.arena_range().arena().get(),
        }
        .file_name(),
    );
    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("selected V1 arena for exact reseal");
    file.seek(SeekFrom::Start(extent.arena_range().offset()))
        .unwrap();
    let mut manifest_bytes = [0; EXTENT_ARENA_MANIFEST_FRAME_BYTES];
    file.read_exact(&mut manifest_bytes).unwrap();
    let (manifest, format) = DurableExtentManifest::decode(&manifest_bytes).unwrap();
    let layout = ExtentArenaFrameLayout::new(format, manifest.alignment()).unwrap();
    let offset = extent.arena_range().offset() + layout.chunk_offset(1).unwrap();
    file.seek(SeekFrom::Start(offset)).unwrap();
    let mut observed = vec![0; original.len()];
    file.read_exact(&mut observed).unwrap();
    assert_eq!(
        observed, original,
        "exact C8-read V1 frame must be retained"
    );
    let coordinate = ExtentChunkCoordinate::new(
        extent.record(),
        extent.extent_cell(),
        manifest.logical_bytes(),
        0,
        1,
    )
    .unwrap();
    let (payload, _) = decode_extent_chunk(original, coordinate).unwrap();
    let BlobRecordV1::ReclaimDescriptor(value) = decode_blob_record(payload).unwrap() else {
        panic!("selected control must remain the original V1 descriptor");
    };
    let mut changed_attempt = value.reclaim_attempt();
    changed_attempt[0] ^= 0x40;
    let changed = BlobReclaimDescriptorV1::new(
        value.store(),
        changed_attempt,
        value.source_basis_digest(),
        value.manifest_record(),
        value.manifest_frame_sha256(),
        value.manifest_count(),
        value.source_root_generation(),
        value.candidate_root_generation(),
    )
    .unwrap()
    .encode();
    assert_eq!(changed.len(), payload.len());
    let replacement = encode_extent_chunk(format, coordinate, &changed).unwrap();
    assert_eq!(replacement.len(), original.len());
    assert!(matches!(
        decode_blob_record(decode_extent_chunk(&replacement, coordinate).unwrap().0),
        Ok(BlobRecordV1::ReclaimDescriptor(_))
    ));
    file.seek(SeekFrom::Start(offset)).unwrap();
    file.write_all(&replacement).unwrap();
    file.sync_all().unwrap();
}

fn selected_generation(root: &Path) -> u64 {
    let bytes = fs::read(
        root.join("families/records")
            .join(RecordArtifactFile::CurrentRootSelector.file_name()),
    )
    .unwrap();
    DurableRootSelector::decode(&bytes)
        .unwrap()
        .root_generation()
}
