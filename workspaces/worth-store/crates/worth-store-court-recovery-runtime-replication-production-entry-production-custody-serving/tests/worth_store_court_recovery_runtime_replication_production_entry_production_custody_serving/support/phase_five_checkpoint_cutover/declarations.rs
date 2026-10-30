pub(crate) use std::num::NonZeroU64;

pub(crate) use {
    std::path::Path, worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery,
    worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery_with_limits,
    worth_store_testkit_recovery_runtime::phase_three_support::expect_blocked,
    worth_store_testkit_recovery_runtime::phase_three_support::initialize_store,
    worth_store_testkit_recovery_runtime::phase_three_support::limit_declaration,
    worth_store_testkit_recovery_runtime::phase_three_support::limits,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_covered_wal,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_genesis,
};
pub(crate) use {
    worth_store_physical_format::CheckpointBindingCompactionHeader,
    worth_store_physical_format::CheckpointDirtyFrameBasis,
    worth_store_physical_format::CheckpointRootBasis,
    worth_store_physical_format::CheckpointStreamEncoder,
    worth_store_physical_format::CheckpointWalSourceRange,
    worth_store_physical_format::PhysicalCheckpointIdentity,
    worth_store_physical_format::PhysicalCheckpointSource,
    worth_store_physical_format::RecordArtifactFile,
    worth_store_physical_format::RecordFrameCoordinate,
    worth_store_physical_format::CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES,
    worth_store_physical_format::CHECKPOINT_DIRTY_FRAME_RECORD_BYTES,
    worth_store_physical_format::CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryBlockKind,
    worth_store_recovery_runtime::PhysicalRecoveryCheckpointIntegrityDenial,
    worth_store_recovery_runtime::PhysicalRecoveryLimitDimension,
    worth_store_recovery_runtime::PhysicalRecoveryLimits,
    worth_store_recovery_runtime::PhysicalRecoverySourceDenial,
};

pub(crate) fn checkpoint_with_dirty_and_bindings(
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    binding_count: usize,
) -> (PhysicalCheckpointIdentity, Vec<u8>) {
    let checkpoint = worth_store_physical_format::PhysicalCheckpointIdentity::new(
        store,
        std::num::NonZeroU64::new(1).unwrap(),
    );
    let source = worth_store_physical_format::PhysicalCheckpointSource::concurrent(
        checkpoint,
        worth_store_physical_format::CheckpointWalSourceRange::new(1, 2).unwrap(),
        worth_store_physical_format::CheckpointRootBasis::new(1, 7),
        1,
    );
    let (mut encoder, header) = worth_store_physical_format::CheckpointStreamEncoder::begin(source);
    let coordinate = worth_store_physical_format::RecordFrameCoordinate::new(
        worth_store_physical_format::RecordArtifactFile::RootManifest { generation: 1 },
        0,
        1,
    )
    .unwrap();
    let dirty = encoder.encode_dirty_basis(
        worth_store_physical_format::CheckpointDirtyFrameBasis::new(coordinate, 1),
    );
    let (mut compaction, compaction_record) = encoder.begin_binding_compaction(
        worth_store_physical_format::CheckpointBindingCompactionHeader::new(1, 2).unwrap(),
    );
    let mut bindings = Vec::new();
    for index in 0..binding_count {
        bindings.push(
            compaction
                .encode_binding_record(&[b'b', b'i', b'n', b'd', index as u8, 1, 2])
                .unwrap(),
        );
    }
    let (_, footer) = compaction.finish();
    let mut bytes = header;
    bytes.extend_from_slice(&dirty);
    bytes.extend_from_slice(&compaction_record);
    for binding in bindings {
        bytes.extend_from_slice(&binding);
    }
    bytes.extend_from_slice(&footer);
    (checkpoint, bytes)
}

pub(crate) fn write_checkpoint(root: &std::path::Path, bytes: &[u8]) {
    std::fs::write(root.join("families").join("checkpoint.current"), bytes).unwrap();
}

pub(crate) fn reseal_record_crc(record: &mut [u8]) {
    let checksum_offset = record.len() - 4;
    let checksum = crc32c(&record[..checksum_offset]);
    record[checksum_offset..].copy_from_slice(&checksum.to_le_bytes());
}

pub(crate) fn crc32c(bytes: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0x82f6_3b78 & mask);
        }
    }
    !crc
}
