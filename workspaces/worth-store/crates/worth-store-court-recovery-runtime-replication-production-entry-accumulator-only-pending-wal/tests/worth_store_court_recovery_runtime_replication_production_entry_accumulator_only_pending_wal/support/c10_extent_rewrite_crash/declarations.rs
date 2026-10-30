pub(crate) use {
    crate::c10_extent_rewrite_crash::arena_io::arena_path,
    crate::c10_extent_rewrite_crash::arena_io::extent_payload,
    crate::c10_extent_rewrite_crash::arena_io::flip_arena_byte,
    crate::c10_extent_rewrite_crash::arena_io::payload,
};

pub(crate) use std::thread;
pub(crate) use {std::io::Seek, std::io::SeekFrom, std::io::Write};
pub(crate) use {std::path::Path, std::path::PathBuf};
pub(crate) use {std::process::Command, std::process::Stdio};
pub(crate) use {std::time::Duration, std::time::Instant};

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint;
pub(crate) use worth_store_testkit_recovery_runtime::phase_three_support::recovery_request_with_limits;
pub(crate) use {
    crate::c10_extent_rewrite_crash::c10_crash_evidence::directory_snapshot,
    crate::c10_extent_rewrite_crash::c10_crash_evidence::ordinary_limits,
    crate::c10_extent_rewrite_crash::c10_crash_evidence::wal_contains_rewrite,
};
pub(crate) use {
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationRequest,
};
pub(crate) use {
    worth_store_physical_format::decode_data_frame_page_lsn,
    worth_store_physical_format::decode_extent_chunk,
    worth_store_physical_format::encode_data_frame_page_lsn,
    worth_store_physical_format::prepare_extent_chunk,
    worth_store_physical_format::CurrentPhysicalRecordPlacement,
    worth_store_physical_format::DurableExtentManifest,
    worth_store_physical_format::DurableExtentRecordPlacement,
    worth_store_physical_format::DurableFrameKind,
    worth_store_physical_format::ExtentArenaFrameLayout,
    worth_store_physical_format::ExtentArenaRange,
    worth_store_physical_format::ExtentChunkCoordinate,
    worth_store_physical_format::PersistedRecordIdentity,
    worth_store_physical_format::DURABLE_EXTENT_FRAME_HEADER_BYTES,
    worth_store_physical_format::EXTENT_ARENA_MANIFEST_FRAME_BYTES,
    worth_store_physical_format::EXTENT_CHUNK_METADATA_BYTES,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff,
    worth_store_recovery_runtime::WorthStoreRecovery,
};
pub(crate) use {
    worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};

pub(crate) const CHILD_MARKER: &str = "C10_EXTENT_REWRITE_MARKER";
pub(crate) const CHILD_CHECKPOINT: &str = "C10_EXTENT_REWRITE_CHECKPOINT";
pub(crate) const PAGE_BYTES: usize = 16 * 1024;
pub(crate) const PAYLOAD_BYTES: usize = 40_000;
pub(crate) const ARENAS: &str = "families/records/arenas";

pub(crate) fn assert_blocked_without_successor(marker: &Path, root: &Path) {
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        recovery_request_with_limits(root, ordinary_limits()),
    );
    assert!(matches!(
        outcome,
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(_)
    ));
    assert_eq!(
        selected_route(
            root,
            worth_store_testkit_recovery_runtime::c10_phase_five_read::load_identity(marker)
        )
        .generation,
        1
    );
}

pub(crate) fn replace_source_payload(root: &Path) {
    let record = worth_store_testkit_recovery_runtime::arena_route::selected_routes(root)
        .into_iter()
        .find(|route| route.generation == 1)
        .expect("the killed root retains its source extent");
    let path = arena_path(root, record.range);
    let original = std::fs::read(&path).unwrap();
    let range_start = record.range.offset() as usize;
    let (manifest, format) = worth_store_physical_format::DurableExtentManifest::decode(
        &original[range_start..range_start + EXTENT_ARENA_MANIFEST_FRAME_BYTES],
    )
    .unwrap();
    assert_eq!(manifest.record(), record.record);
    assert_eq!(manifest.extent().get(), record.extent);
    assert_eq!(manifest.extent_cell().generation().get(), record.generation);
    assert_eq!(manifest.logical_bytes(), record.payload_bytes);
    let layout =
        worth_store_physical_format::ExtentArenaFrameLayout::new(format, manifest.alignment())
            .unwrap();
    assert!(layout.admits(record.range, manifest.chunk_count()));
    let mut forged = payload();
    forged[PAYLOAD_BYTES / 2] ^= 0xff;
    let capacity = PAGE_BYTES - DURABLE_EXTENT_FRAME_HEADER_BYTES - EXTENT_CHUNK_METADATA_BYTES;
    let mut file = std::fs::OpenOptions::new().write(true).open(&path).unwrap();
    for (ordinal, start) in (1_u32..).zip((0..forged.len()).step_by(capacity)) {
        let offset = range_start + layout.chunk_offset(ordinal).unwrap() as usize;
        let length = (forged.len() - start).min(capacity);
        let coordinate = worth_store_physical_format::ExtentChunkCoordinate::new(
            manifest.record(),
            manifest.extent_cell(),
            manifest.logical_bytes(),
            start as u64,
            ordinal,
        )
        .unwrap();
        let mut chunk = prepare_extent_chunk(format, coordinate, length).unwrap();
        chunk
            .payload_mut()
            .copy_from_slice(&forged[start..start + length]);
        let mut frame = chunk.seal();
        let page_lsn = decode_data_frame_page_lsn(
            &original[offset..offset + frame.len()],
            worth_store_physical_format::DurableFrameKind::Extent,
        )
        .unwrap();
        encode_data_frame_page_lsn(
            &mut frame,
            worth_store_physical_format::DurableFrameKind::Extent,
            page_lsn,
        )
        .unwrap();
        file.seek(std::io::SeekFrom::Start(offset as u64)).unwrap();
        file.write_all(&frame).unwrap();
    }
    file.sync_all().unwrap();
}

pub(crate) fn rewrite_checkpoint(name: &str) -> CertificationPhysicalMutationCheckpoint {
    match name {
        "before-wal" => worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::BeforeWalAppend,
        "after-wal" => worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterWalDurability,
        "after-data" => worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterDataSettlement,
        "during-root" => worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::DuringRootPublication,
        "after-replace" => worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterRootReplacement,
        "before-ack" => worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::BeforeTerminalFinalization,
        other => panic!("unknown extent rewrite checkpoint {other}"),
    }
}

pub(crate) fn checkpoint_source(world: &PhysicalResidencyStoreWorld) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x93; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let handle = match world.serving().checkpoints().start(request).into_raw() {
        worth_proof::TransitionOutcome::Success(handle) => handle,
        _ => panic!("source checkpoint was not admitted"),
    };
    match handle.wait() {
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_) => {}
        _ => panic!("source checkpoint must complete"),
    }
}

pub(crate) fn kill_child(checkpoint: &str) -> (tempfile::TempDir, PathBuf) {
    let parent = tempfile::tempdir().unwrap();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "process::recovery_runtime::c10_extent_rewrite_crash::extent_rewrite_child_parks_at_checkpoint",
            "--ignored",
            "--nocapture",
        ])
        .env(CHILD_MARKER, parent.path())
        .env(CHILD_CHECKPOINT, checkpoint)
        .env("TMP", parent.path())
        .env("TEMP", parent.path())
        .env("TMPDIR", parent.path())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("launch extent rewrite child");
    let reached = parent.path().join("reached");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while !reached.exists() {
        if std::time::Instant::now() > deadline {
            let _ = child.kill();
            panic!("extent rewrite child did not reach {checkpoint}");
        }
        if let Some(status) = child.try_wait().unwrap() {
            let output = child.wait_with_output().unwrap();
            panic!(
                "extent rewrite child exited early: {status}\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    child.kill().unwrap();
    let _ = child.wait();
    let root = std::path::PathBuf::from(
        std::fs::read_to_string(parent.path().join("root-path.txt")).unwrap(),
    );
    (parent, root)
}

/// Recovers three times and returns the payload and extent generation the
/// first and the settled passes select. A handoff reports the root recovery
/// selected before publishing, so the second pass shows what the first
/// published; no extent data or manifest may change after the first pass.
pub(crate) fn recover_until_settled(
    marker: &Path,
    root: &Path,
) -> ((Vec<u8>, u64), (Vec<u8>, u64)) {
    let record = worth_store_testkit_recovery_runtime::c10_phase_five_read::load_identity(marker);
    let first = selected(root, &recover(root), record);
    let arenas = directory_snapshot(root, ARENAS);
    let roots = directory_snapshot(root, "families/records/roots");
    let second = selected(root, &recover(root), record);
    let third = selected(root, &recover(root), record);
    assert_eq!(second, third, "recovery must be idempotent once settled");
    assert_eq!(arenas, directory_snapshot(root, ARENAS));
    assert_eq!(roots, directory_snapshot(root, "families/records/roots"));
    (first, second)
}

pub(crate) fn recover(root: &Path) -> RecoveredPhysicalRuntimeHandoff {
    match worth_store_recovery_runtime::WorthStoreRecovery::recover(recovery_request_with_limits(
        root,
        ordinary_limits(),
    )) {
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) => *handoff,
        _ => panic!("killed extent rewrite must recover"),
    }
}

pub(crate) fn selected(
    root: &Path,
    handoff: &RecoveredPhysicalRuntimeHandoff,
    record: PersistedRecordIdentity,
) -> (Vec<u8>, u64) {
    let placement = handoff
        .selected_sources()
        .page_facts()
        .placements()
        .iter()
        .find_map(|placement| match placement {
            worth_store_physical_format::CurrentPhysicalRecordPlacement::Extent(extent)
                if extent.record() == record =>
            {
                Some(*extent)
            }
            _ => None,
        })
        .expect("selected root does not place the record in an extent");
    (
        extent_payload(root, placement),
        placement.extent_generation(),
    )
}

/// The one addressed extent the selected root routes for `record`.
pub(crate) fn selected_route(
    root: &std::path::Path,
    record: worth_store_physical_format::PersistedRecordIdentity,
) -> worth_store_testkit_recovery_runtime::arena_route::RoutedExtent {
    let matching = worth_store_testkit_recovery_runtime::arena_route::selected_routes(root)
        .into_iter()
        .filter(|route| route.record == record)
        .collect::<Vec<_>>();
    assert_eq!(
        matching.len(),
        1,
        "selected root has one addressed extent for the record"
    );
    matching[0]
}
