pub(crate) use std::fs;
pub(crate) use worth_store_recovery_runtime::PhysicalRecoveryBlockKind;
pub(crate) use {
    crate::c11_arena_copy_crash::kill_after_final_wal, crate::c11_arena_copy_crash::load_identity,
    crate::c11_arena_copy_crash::pre_final, crate::c11_arena_copy_crash::recover,
    crate::c11_arena_copy_crash::selected_extent, crate::copy_frame, std::path::Path,
    std::path::PathBuf, worth_proof::TransitionOutcome,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store_physical_format::DurableExtentRecordPlacement,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
    worth_store_testkit_recovery_runtime::support::append,
    worth_store_testkit_recovery_runtime::support::configuration,
    worth_store_testkit_recovery_runtime::support::generation,
    worth_store_testkit_recovery_runtime::support::open,
    worth_store_testkit_recovery_runtime::support::recovery_request,
    worth_store_testkit_recovery_runtime::support::scan,
    worth_store_testkit_recovery_runtime::support::EXTENT_BYTES,
};
pub(crate) use {
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalRetirementDenial,
};
pub(crate) use {
    worth_store_physical_format::decode_data_frame_page_lsn,
    worth_store_physical_format::DurableExtentManifest,
    worth_store_physical_format::DurableFrameKind,
    worth_store_physical_format::ExtentArenaFrameLayout,
    worth_store_physical_format::RecordArtifactFile,
    worth_store_physical_format::EXTENT_ARENA_MANIFEST_FRAME_BYTES,
};

pub(crate) fn assert_redo_blocked_before_effect(root: &Path) {
    let catalog = std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap();
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(recovery_request(root));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
        panic!("missing or invalid historical copy evidence must block: {outcome:?}")
    };
    assert_eq!(
        blocked.cause().damage(),
        Some(worth_store_recovery_runtime::PhysicalRecoveryBlockKind::RedoPlanning)
    );
    assert_eq!(blocked.recovery_effects(), 0);
    assert_eq!(
        std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap(),
        catalog,
        "historical proof denial must not publish a root"
    );
}

pub(crate) fn arena_file(root: &Path, placement: DurableExtentRecordPlacement) -> PathBuf {
    root.join("families/records/arenas").join(
        worth_store_physical_format::RecordArtifactFile::ExtentArena {
            arena: placement.arena_range().arena().get(),
        }
        .file_name(),
    )
}

pub(crate) fn arena_range_bytes(root: &Path, placement: DurableExtentRecordPlacement) -> Vec<u8> {
    let arena = std::fs::read(arena_file(root, placement)).unwrap();
    let range = placement.arena_range();
    let start = range.offset() as usize;
    let end = (range.end() as usize).min(arena.len());
    assert!(
        end > start,
        "selected destination must have persisted bytes"
    );
    arena[start..end].to_vec()
}

pub(crate) fn arena_range_prefix(
    root: &Path,
    placement: DurableExtentRecordPlacement,
    len: usize,
) -> Vec<u8> {
    let arena = std::fs::read(arena_file(root, placement)).unwrap();
    let range = placement.arena_range();
    assert!(len <= range.length() as usize);
    let start = range.offset() as usize;
    arena[start..start + len].to_vec()
}

pub(crate) fn write_arena_range(
    root: &Path,
    placement: DurableExtentRecordPlacement,
    bytes: &[u8],
) {
    let path = arena_file(root, placement);
    let mut arena = std::fs::read(&path).unwrap();
    let range = placement.arena_range();
    assert!(bytes.len() <= range.length() as usize);
    let start = range.offset() as usize;
    arena[start..start + bytes.len()].copy_from_slice(bytes);
    std::fs::write(path, arena).unwrap();
}

pub(crate) fn alter_retained_destination_page_lsn(
    root: &Path,
    placement: DurableExtentRecordPlacement,
) {
    let path = arena_file(root, placement);
    let mut arena = std::fs::read(&path).unwrap();
    let range = placement.arena_range();
    let start = range.offset() as usize;
    assert!(
        start + EXTENT_ARENA_MANIFEST_FRAME_BYTES <= arena.len(),
        "the selected destination manifest must be fully persisted"
    );
    let manifest = &arena[start..start + EXTENT_ARENA_MANIFEST_FRAME_BYTES];
    let (manifest, format) =
        worth_store_physical_format::DurableExtentManifest::decode(manifest).unwrap();
    let relative =
        worth_store_physical_format::ExtentArenaFrameLayout::new(format, manifest.alignment())
            .unwrap()
            .chunk_offset(1)
            .unwrap() as usize;
    let chunk = start + relative;
    assert_eq!(&arena[chunk..chunk + 8], b"WRC5FRM\0");
    assert_eq!(
        arena[chunk + 8],
        worth_store_physical_format::DurableFrameKind::Extent as u8
    );
    let payload = u32::from_le_bytes(arena[chunk + 24..chunk + 28].try_into().unwrap()) as usize;
    let frame_end = chunk + 48 + payload;
    assert!(
        frame_end <= arena.len() && frame_end <= range.end() as usize,
        "the mutated chunk frame must be fully persisted inside its allocated range"
    );
    let frame = &mut arena[chunk..frame_end];
    let before =
        decode_data_frame_page_lsn(frame, worth_store_physical_format::DurableFrameKind::Extent)
            .unwrap();
    frame[36..44].copy_from_slice(&(before.get() + 1).to_le_bytes());
    let checksum = crc32c(&frame[..44], &frame[48..]);
    frame[44..48].copy_from_slice(&checksum.to_le_bytes());
    assert_eq!(
        decode_data_frame_page_lsn(frame, worth_store_physical_format::DurableFrameKind::Extent)
            .unwrap()
            .get(),
        before.get() + 1,
        "negative twin retains valid C5 framing"
    );
    std::fs::write(path, arena).unwrap();
}

pub(crate) fn crc32c(prefix: &[u8], payload: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in prefix.iter().chain(payload) {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0x82f6_3b78 & mask);
        }
    }
    !crc
}
