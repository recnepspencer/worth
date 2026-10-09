use super::super::super::*;
use super::fixture::{created, owner, staged_path};

#[test]
fn positioned_ranges_append_truncate_allocation_and_metadata_are_real() {
    let (root, owner) = owner();
    let path = staged_path(&owner, 1);
    let handle = created(owner.create_new(&path));

    let write = handle.positioned_write(PositionedWriteRequest::new(3, b"abcdef"));
    assert!(matches!(
        write.result(),
        MediaOperationResult::Completed(CompletedMediaEffect::PositionedWriteCompleted(_))
    ));
    let mut range = [0_u8; 4];
    let read = handle.positioned_read(PositionedReadRequest::new(4, &mut range));
    assert!(matches!(
        read.result(),
        PositionedReadResult::Transferred(transfer) if transfer.bytes() == 4
    ));
    assert_eq!(&range, b"bcde");

    let append = handle.append(AppendRequest::new(b"XYZ"));
    assert!(matches!(
        append.result(),
        MediaOperationResult::Completed(CompletedMediaEffect::AppendCompleted(transfer))
            if transfer.start() == MediaTransferPosition::KnownAppendPosition(9)
    ));
    assert!(matches!(
        handle.metadata().result(),
        MediaMetadataResult::Observed(metadata) if metadata.logical_length() == 12
    ));

    assert_eq!(
        handle.truncate(TruncateRequest::new(5)).effect_status(),
        MediaEffectStatus::CompletedEffect
    );
    assert!(matches!(
        handle
            .allocate(AllocationRequest::new(
                32,
                8,
                MediaAllocationMode::LogicalLengthOnly,
            ))
            .result(),
        MediaAllocationResult::Completed(observation)
            if observation.logical_length() == 40
                && observation.physical() == MediaPhysicalAllocationPosture::NotRequested
    ));
    assert!(matches!(
        handle
            .allocate(AllocationRequest::new(
                64,
                8,
                MediaAllocationMode::SparsePhysicalRange,
            ))
            .result(),
        MediaAllocationResult::Failed(failure)
            if failure.effect_status() == MediaEffectStatus::UnsupportedCapability
    ));
    let counters = owner.counters();
    assert_eq!(counters.unsupported_capabilities(), 1);
    drop(handle);
    assert_eq!(
        std::fs::metadata(root.path().join("store").join(path.as_path()))
            .unwrap()
            .len(),
        40
    );
}

#[test]
fn peak_operation_bytes_track_the_largest_causal_boundary_request() {
    let (_root, owner) = owner();
    let path = staged_path(&owner, 31);
    let handle = created(owner.create_new(&path));
    let bytes = [0xA5; 512];
    assert_eq!(
        handle
            .positioned_write(PositionedWriteRequest::new(0, &bytes))
            .effect_status(),
        MediaEffectStatus::CompletedEffect
    );
    assert_eq!(owner.counters().peak_request_width_bytes(), 512);
}

#[test]
fn open_modes_empty_files_and_eof_crossings_preserve_exact_effects() {
    let (_root, owner) = owner();
    let path = staged_path(&owner, 24);
    let empty = created(owner.create_new(&path));
    let mut byte = [0_u8; 1];
    assert!(matches!(
        empty
            .positioned_read(PositionedReadRequest::new(0, &mut byte))
            .result(),
        PositionedReadResult::EndOfFile {
            requested_offset: 0
        }
    ));
    assert!(matches!(
        owner.create_new(&path).into_result(),
        NamespaceFileOpenResult::Failed(failure)
            if failure.effect_status() == MediaEffectStatus::DeniedBeforeEffect
    ));
    drop(empty);
    let existing = match owner.open_existing_for_mutation(&path).into_result() {
        NamespaceFileOpenResult::Opened {
            kind: NamespaceFileOpenKind::Existing,
            handle,
        } => handle,
        other => panic!("existing open changed mode: {other:?}"),
    };
    assert_eq!(
        existing
            .positioned_write(PositionedWriteRequest::new(0, b"four"))
            .effect_status(),
        MediaEffectStatus::CompletedEffect
    );
    let mut crossing = [0_u8; 4];
    let outcome = existing.positioned_read(PositionedReadRequest::new(3, &mut crossing));
    assert!(matches!(
        outcome.result(),
        PositionedReadResult::Failed(failure)
            if matches!(
                failure.kind(),
                MediaOperationFailureKind::PartialTransfer(transfer)
                    if transfer.completed_bytes() == 1
                        && transfer.continuation_position() == Some(4)
            )
    ));
    assert_eq!(crossing[0], b'r');
    let counters = owner.counters();
    assert_eq!(counters.positioned_read_attempts(), 2);
    assert_eq!(counters.eof_observations(), 1);
    assert_eq!(counters.short_transfers(), 1);
    assert_eq!(counters.retry_attempts(), 0);
    assert!(counters.is_conserved());
}

#[test]
fn read_only_open_observes_a_file_without_requesting_write_access() {
    let (root, owner) = owner();
    let path = staged_path(&owner, 30);
    let handle = created(owner.create_new(&path));
    assert_eq!(
        handle
            .positioned_write(PositionedWriteRequest::new(0, b"read-only"))
            .effect_status(),
        MediaEffectStatus::CompletedEffect
    );
    drop(handle);
    let physical = root.path().join("store").join(path.as_path());
    let mut permissions = std::fs::metadata(&physical)
        .expect("read-only fixture metadata")
        .permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(&physical, permissions).expect("make fixture read-only");

    let handle = match owner.open_existing(&path).into_result() {
        NamespaceFileOpenResult::Opened { handle, .. } => handle,
        other => panic!("read-only observation open failed: {other:?}"),
    };
    let mut bytes = [0_u8; 9];
    assert!(matches!(
        handle
            .positioned_read(PositionedReadRequest::new(0, &mut bytes))
            .result(),
        PositionedReadResult::Transferred(transfer) if transfer.bytes() == 9
    ));
    assert_eq!(&bytes, b"read-only");
}

#[test]
fn range_memory_shape_is_independent_of_large_logical_file_length() {
    if !super::super::allocation_probe::alone_in_its_process(
        module_path!(),
        "range_memory_shape_is_independent_of_large_logical_file_length",
    ) {
        return;
    }
    let (_root, owner) = owner();
    let payload = [0x5a_u8; 4096];
    let mut allocation_observations = Vec::new();
    for (value, logical_length) in [(21, 1_u64 << 20), (22, 64_u64 << 20), (23, 256_u64 << 20)] {
        let path = staged_path(&owner, value);
        let handle = created(owner.create_new(&path));
        assert_eq!(
            handle
                .truncate(TruncateRequest::new(logical_length))
                .effect_status(),
            MediaEffectStatus::CompletedEffect
        );
        let offset = logical_length - payload.len() as u64;
        let allocated = super::super::allocation_probe::allocated_bytes_during(|| {
            assert_eq!(
                handle
                    .positioned_write(PositionedWriteRequest::new(offset, &payload))
                    .effect_status(),
                MediaEffectStatus::CompletedEffect
            );
            let mut observed = [0_u8; 4096];
            assert!(matches!(
                handle
                    .positioned_read(PositionedReadRequest::new(offset, &mut observed))
                    .result(),
                PositionedReadResult::Transferred(transfer) if transfer.bytes() == 4096
            ));
            assert_eq!(observed, payload);
        });
        allocation_observations.push(allocated);
    }
    assert!(
        allocation_observations
            .windows(2)
            .all(|pair| pair[0] == pair[1]),
        "fixed range allocated by file length: {allocation_observations:?}"
    );
}
