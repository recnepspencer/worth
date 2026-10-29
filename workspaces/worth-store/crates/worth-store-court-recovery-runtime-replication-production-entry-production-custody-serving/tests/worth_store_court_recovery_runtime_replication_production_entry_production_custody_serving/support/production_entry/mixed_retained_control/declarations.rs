//! A real failed-ingest V1 control survives a NoRelease checkpoint, two
//! distinct released-generation V3 drops, and an ordinary selected tail.
//! Store must rejoin that otherwise unrelated selected control independently.

pub(crate) use {std::fs, std::io::Read, std::io::Seek, std::io::SeekFrom, std::io::Write, std::path::Path};

pub(crate) use {worth_store::physical_runtime::PhysicalMutationDeadline, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving, worth_store_testkit_recovery_runtime::production_entry::pending_wal_world};
pub(crate) use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::ManifestEntryCapacity, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::RecordAppendBatch};
pub(crate) use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::decode_extent_chunk, worth_store_physical_format::encode_extent_chunk, worth_store_physical_format::BlobReclaimDescriptorV1, worth_store_physical_format::BlobRecordKind, worth_store_physical_format::BlobRecordV1, worth_store_physical_format::CurrentPhysicalRecordPlacement, worth_store_physical_format::DurableExtentManifest, worth_store_physical_format::DurableRootSelector, worth_store_physical_format::ExtentArenaFrameLayout, worth_store_physical_format::ExtentChunkCoordinate, worth_store_physical_format::RecordArtifactFile, worth_store_physical_format::SelectedRecordContentClass, worth_store_physical_format::DURABLE_EXTENT_FRAME_HEADER_BYTES, worth_store_physical_format::EXTENT_ARENA_MANIFEST_FRAME_BYTES, worth_store_physical_format::EXTENT_CHUNK_METADATA_BYTES};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::WorthStoreRecovery};

pub(crate) fn recover(root: &Path) -> worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff {
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(root));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        if let worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(failure) = &outcome {
            panic!("mixed current failed-ingest + two V3 rejoin failed: handoff={:?}; reopen={:?}; effects={}",
                failure.handoff_failure(), failure.reopen_failure(), failure.recovery_effects());
        }
        panic!("mixed V1 + two V3 C8/Store custody must recover: {outcome:?}");
    };
    *handoff
}

pub(crate) fn append_ordinary_tail(serving: &worth_store::physical_runtime::ServingPhysicalRuntime) {
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(64).unwrap())
        .admit(worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
            worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        ))
        .unwrap();
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([0xe7; 32]))
        .unwrap();
    let worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .prepare_durable_append(
                worth_store::physical_runtime::RecordAppendBatch::try_from_iter(
                    [b"mixed-v1-v3-ordinary-selected-tail".as_slice()],
                )
                .unwrap(),
                placement,
                worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                    key,
                    worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                ),
            )
            .into_raw()
    else {
        panic!("ordinary tail after mixed V1 and two V3s must prepare");
    };
    assert!(matches!(
        prepared.execute(),
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_)
    ));
}

pub(crate) fn read_v1_descriptor_frame(root: &Path, route: CurrentPhysicalRecordPlacement) -> Vec<u8> {
    let worth_store_physical_format::CurrentPhysicalRecordPlacement::Extent(extent) = route else {
        panic!("retained failed-ingest V1 descriptor must use an extent route");
    };
    let path = root.join("families/records/arenas").join(
        worth_store_physical_format::RecordArtifactFile::ExtentArena {
            arena: extent.arena_range().arena().get(),
        }
        .file_name(),
    );
    let mut file = std::fs::File::open(path).expect("selected V1 control arena");
    file.seek(std::io::SeekFrom::Start(extent.arena_range().offset()))
        .unwrap();
    let mut manifest_bytes = [0; EXTENT_ARENA_MANIFEST_FRAME_BYTES];
    file.read_exact(&mut manifest_bytes).unwrap();
    let (manifest, format) = worth_store_physical_format::DurableExtentManifest::decode(&manifest_bytes).unwrap();
    assert_eq!(manifest.record(), extent.record());
    assert_eq!(manifest.extent_cell(), extent.extent_cell());
    assert_eq!(manifest.chunk_count(), 1);
    let layout = worth_store_physical_format::ExtentArenaFrameLayout::new(format, manifest.alignment()).unwrap();
    let offset = extent.arena_range().offset() + layout.chunk_offset(1).unwrap();
    file.seek(std::io::SeekFrom::Start(offset)).unwrap();
    let mut frame = vec![
        0;
        DURABLE_EXTENT_FRAME_HEADER_BYTES
            + EXTENT_CHUNK_METADATA_BYTES
            + manifest.logical_bytes() as usize
    ];
    file.read_exact(&mut frame).unwrap();
    let coordinate = worth_store_physical_format::ExtentChunkCoordinate::new(
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
        Ok(worth_store_physical_format::BlobRecordV1::ReclaimDescriptor(_))
    ));
    frame
}

pub(crate) fn reseal_v1_descriptor(root: &Path, route: CurrentPhysicalRecordPlacement, original: &[u8]) {
    let worth_store_physical_format::CurrentPhysicalRecordPlacement::Extent(extent) = route else {
        panic!("retained V1 route changed after C8 claim");
    };
    let path = root.join("families/records/arenas").join(
        worth_store_physical_format::RecordArtifactFile::ExtentArena {
            arena: extent.arena_range().arena().get(),
        }
        .file_name(),
    );
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("selected V1 arena for exact reseal");
    file.seek(std::io::SeekFrom::Start(extent.arena_range().offset()))
        .unwrap();
    let mut manifest_bytes = [0; EXTENT_ARENA_MANIFEST_FRAME_BYTES];
    file.read_exact(&mut manifest_bytes).unwrap();
    let (manifest, format) = worth_store_physical_format::DurableExtentManifest::decode(&manifest_bytes).unwrap();
    let layout = worth_store_physical_format::ExtentArenaFrameLayout::new(format, manifest.alignment()).unwrap();
    let offset = extent.arena_range().offset() + layout.chunk_offset(1).unwrap();
    file.seek(std::io::SeekFrom::Start(offset)).unwrap();
    let mut observed = vec![0; original.len()];
    file.read_exact(&mut observed).unwrap();
    assert_eq!(
        observed, original,
        "exact C8-read V1 frame must be retained"
    );
    let coordinate = worth_store_physical_format::ExtentChunkCoordinate::new(
        extent.record(),
        extent.extent_cell(),
        manifest.logical_bytes(),
        0,
        1,
    )
    .unwrap();
    let (payload, _) = decode_extent_chunk(original, coordinate).unwrap();
    let worth_store_physical_format::BlobRecordV1::ReclaimDescriptor(value) = decode_blob_record(payload).unwrap() else {
        panic!("selected control must remain the original V1 descriptor");
    };
    let mut changed_attempt = value.reclaim_attempt();
    changed_attempt[0] ^= 0x40;
    let changed = worth_store_physical_format::BlobReclaimDescriptorV1::new(
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
        Ok(worth_store_physical_format::BlobRecordV1::ReclaimDescriptor(_))
    ));
    file.seek(std::io::SeekFrom::Start(offset)).unwrap();
    file.write_all(&replacement).unwrap();
    file.sync_all().unwrap();
}

pub(crate) fn selected_generation(root: &Path) -> u64 {
    let bytes = std::fs::read(
        root.join("families/records")
            .join(worth_store_physical_format::RecordArtifactFile::CurrentRootSelector.file_name()),
    )
    .unwrap();
    worth_store_physical_format::DurableRootSelector::decode(&bytes)
        .unwrap()
        .root_generation()
}