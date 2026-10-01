use super::*;
use std::fs;
use worth_store::physical_runtime::{
    PhysicalMutationOutcome, PhysicalMutationPreparationSuccess, PhysicalRetirementDenial,
};
use worth_store_physical_format::{
    decode_data_frame_page_lsn, DurableExtentManifest, DurableFrameKind, ExtentArenaFrameLayout,
    RecordArtifactFile, EXTENT_ARENA_MANIFEST_FRAME_BYTES,
};
use worth_store_recovery_runtime::PhysicalRecoveryBlockKind;

#[test]
fn historical_copy_destination_is_retained_and_checked_after_later_rewrite() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    kill_after_final_wal(directory.path());
    let record = load_identity(directory.path());
    legacy_copy_frame::rewrite_one_durable_copy_as_v5(&root);
    legacy_copy_frame::assert_no_published_copy_resolution(&root);
    let intents = pre_final::independent_copy_intent::produced_copy_intents(&root);
    assert_eq!(intents.len(), 1);

    let first = recover(&root);
    let source = selected_extent(&first, record);
    drop(first);
    let copied = recover(&root);
    let destination = selected_extent(&copied, record);
    let copy_result_root = copied
        .selected_sources()
        .root()
        .selected()
        .selector()
        .root_generation();
    assert_eq!(
        destination.arena_range().arena().get(),
        intents[0].destination[0]
    );
    assert_eq!(
        destination.arena_range().offset(),
        intents[0].destination[1]
    );
    assert_eq!(
        destination.arena_range().length(),
        intents[0].destination[2]
    );
    assert_eq!(
        destination.extent_generation(),
        source.extent_generation() + 1
    );
    drop(copied);

    let retained_destination = arena_range_bytes(&root, destination);
    let serving = open(&root);
    let (_, placement, _) = configuration();
    let selected = scan(&serving)
        .into_iter()
        .filter(|(id, bytes)| {
            id.allocation_epoch() == record.allocation_epoch()
                && id.ordinal() == record.ordinal()
                && bytes.len() == EXTENT_BYTES
                && bytes.iter().all(|&byte| byte == 93)
        })
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 1, "copy must be selected exactly once");
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([221; 32]))
        .unwrap();
    let request = PhysicalMutationRequest::platform_durable(
        key,
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(rewrite)) =
        submission
            .rewrite_selected_extent_record(placement, request, selected[0].0)
            .into_raw()
    else {
        panic!("selected copied extent must admit an ordinary rewrite")
    };
    assert!(matches!(
        rewrite.execute(),
        PhysicalMutationOutcome::Completed(_)
    ));
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Protected),
        "an unresolved copy obligation must keep its protected history"
    );
    append(&serving, placement, [222; 32], b"later-root");
    assert_eq!(
        arena_range_prefix(&root, destination, retained_destination.len()),
        retained_destination
    );
    serving.close();
    let selected_root =
        generation(&fs::read(root.join("families/records/bootstrap.catalog")).unwrap());
    assert!(
        selected_root >= copy_result_root + 2,
        "the copy-result root must be historical, not selected or retained-previous"
    );

    let historical_root = root.join("families/records/roots").join(
        RecordArtifactFile::RootManifest {
            generation: copy_result_root,
        }
        .file_name(),
    );
    assert!(
        historical_root.is_file(),
        "copy-result root must remain addressable"
    );
    let hidden_root = directory.path().join("withheld-copy-result-root");
    fs::rename(&historical_root, &hidden_root).unwrap();
    assert_redo_blocked_before_effect(&root);
    fs::rename(&hidden_root, &historical_root).unwrap();

    alter_retained_destination_page_lsn(&root, destination);
    assert_redo_blocked_before_effect(&root);
    write_arena_range(&root, destination, &retained_destination);

    let recovered = recover(&root);
    let selected = selected_extent(&recovered, record);
    assert_ne!(selected.arena_range(), destination.arena_range());
    assert_eq!(
        selected.extent_generation(),
        destination.extent_generation() + 1
    );
    drop(recovered);
    let serving = open(&root);
    assert_eq!(
        scan(&serving)
            .iter()
            .filter(|(_, bytes)| bytes.len() == EXTENT_BYTES && bytes.iter().all(|&b| b == 93))
            .count(),
        1,
        "historical copy must not replay into the selected replacement"
    );
    serving.close();
    legacy_copy_frame::assert_no_published_copy_resolution(&root);
}

fn assert_redo_blocked_before_effect(root: &Path) {
    let catalog = fs::read(root.join("families/records/bootstrap.catalog")).unwrap();
    let outcome = WorthStoreRecovery::recover(recovery_request(root));
    let PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
        panic!("missing or invalid historical copy evidence must block: {outcome:?}")
    };
    assert_eq!(blocked.kind, PhysicalRecoveryBlockKind::RedoPlanning);
    assert_eq!(blocked.recovery_effects(), 0);
    assert_eq!(
        fs::read(root.join("families/records/bootstrap.catalog")).unwrap(),
        catalog,
        "historical proof denial must not publish a root"
    );
}

fn arena_file(root: &Path, placement: DurableExtentRecordPlacement) -> PathBuf {
    root.join("families/records/arenas").join(
        RecordArtifactFile::ExtentArena {
            arena: placement.arena_range().arena().get(),
        }
        .file_name(),
    )
}

fn arena_range_bytes(root: &Path, placement: DurableExtentRecordPlacement) -> Vec<u8> {
    let arena = fs::read(arena_file(root, placement)).unwrap();
    let range = placement.arena_range();
    let start = range.offset() as usize;
    let end = (range.end() as usize).min(arena.len());
    assert!(
        end > start,
        "selected destination must have persisted bytes"
    );
    arena[start..end].to_vec()
}

fn arena_range_prefix(root: &Path, placement: DurableExtentRecordPlacement, len: usize) -> Vec<u8> {
    let arena = fs::read(arena_file(root, placement)).unwrap();
    let range = placement.arena_range();
    assert!(len <= range.length() as usize);
    let start = range.offset() as usize;
    arena[start..start + len].to_vec()
}

fn write_arena_range(root: &Path, placement: DurableExtentRecordPlacement, bytes: &[u8]) {
    let path = arena_file(root, placement);
    let mut arena = fs::read(&path).unwrap();
    let range = placement.arena_range();
    assert!(bytes.len() <= range.length() as usize);
    let start = range.offset() as usize;
    arena[start..start + bytes.len()].copy_from_slice(bytes);
    fs::write(path, arena).unwrap();
}

fn alter_retained_destination_page_lsn(root: &Path, placement: DurableExtentRecordPlacement) {
    let path = arena_file(root, placement);
    let mut arena = fs::read(&path).unwrap();
    let range = placement.arena_range();
    let start = range.offset() as usize;
    assert!(
        start + EXTENT_ARENA_MANIFEST_FRAME_BYTES <= arena.len(),
        "the selected destination manifest must be fully persisted"
    );
    let manifest = &arena[start..start + EXTENT_ARENA_MANIFEST_FRAME_BYTES];
    let (manifest, format) = DurableExtentManifest::decode(manifest).unwrap();
    let relative = ExtentArenaFrameLayout::new(format, manifest.alignment())
        .unwrap()
        .chunk_offset(1)
        .unwrap() as usize;
    let chunk = start + relative;
    assert_eq!(&arena[chunk..chunk + 8], b"WRC5FRM\0");
    assert_eq!(arena[chunk + 8], DurableFrameKind::Extent as u8);
    let payload = u32::from_le_bytes(arena[chunk + 24..chunk + 28].try_into().unwrap()) as usize;
    let frame_end = chunk + 48 + payload;
    assert!(
        frame_end <= arena.len() && frame_end <= range.end() as usize,
        "the mutated chunk frame must be fully persisted inside its allocated range"
    );
    let frame = &mut arena[chunk..frame_end];
    let before = decode_data_frame_page_lsn(frame, DurableFrameKind::Extent).unwrap();
    frame[36..44].copy_from_slice(&(before.get() + 1).to_le_bytes());
    let checksum = crc32c(&frame[..44], &frame[48..]);
    frame[44..48].copy_from_slice(&checksum.to_le_bytes());
    assert_eq!(
        decode_data_frame_page_lsn(frame, DurableFrameKind::Extent)
            .unwrap()
            .get(),
        before.get() + 1,
        "negative twin retains valid C5 framing"
    );
    fs::write(path, arena).unwrap();
}

fn crc32c(prefix: &[u8], payload: &[u8]) -> u32 {
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
