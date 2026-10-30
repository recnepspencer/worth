pub(crate) use {std::fs, std::path::Path};

pub(crate) use worth_signal::facade::TemporalDuration;
pub(crate) use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
pub(crate) use {
    worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    worth_store::physical_runtime::CompletedPhysicalCheckpoint,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyKey,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::PhysicalWalGroupAppendOutcome,
    worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome,
    worth_store::physical_runtime::PreparedPhysicalMutation,
    worth_store::physical_runtime::SealedPhysicalDurabilityGroupMembers,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

pub(crate) fn prepare(
    submission: &worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission,
    placement: AdmittedRecordPlacementPolicy,
    key: PhysicalMutationIdempotencyKey,
    bytes: &[u8],
) -> PreparedPhysicalMutation {
    match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([bytes]).unwrap(),
            placement,
            request(key),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        _ => panic!("physical mutation preparation must succeed"),
    }
}

pub(crate) fn request(key: PhysicalMutationIdempotencyKey) -> PhysicalMutationRequest {
    worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
        key,
        worth_store::physical_runtime::PhysicalMutationDeadline::at(
            worth_signal::facade::TemporalDuration::temporal_duration(1_000).unwrap(),
        ),
    )
}

pub(crate) fn append_one(
    submission: &worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission,
    prepared: PreparedPhysicalMutation,
) -> SealedPhysicalDurabilityGroupMembers {
    match submission.append_prepared_wal_group(worth_proof::NonEmpty::new(prepared, Vec::new())) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(appended) => {
            appended
        }
        _ => panic!("one-member WAL append must succeed"),
    }
}

pub(crate) fn synchronize(
    submission: &worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission,
    appended: SealedPhysicalDurabilityGroupMembers,
) {
    assert!(matches!(
        submission.synchronize_appended_wal_group(appended),
        worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome::Durable(_)
    ));
}

pub(crate) fn success_checkpoint(
    serving: &ServingPhysicalRuntime,
    key: u8,
) -> CompletedPhysicalCheckpoint {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([key; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::at(
            worth_signal::facade::TemporalDuration::temporal_duration(1_000).unwrap(),
        ),
    );
    let handle = match serving.checkpoints().start(request).into_raw() {
        worth_proof::TransitionOutcome::Success(handle) => handle,
        worth_proof::TransitionOutcome::Denied(cause) => panic!("checkpoint denied: {cause:?}"),
        worth_proof::TransitionOutcome::Deferred(cause) => panic!("checkpoint deferred: {cause:?}"),
        worth_proof::TransitionOutcome::Stale(cause) => panic!("checkpoint stale: {cause:?}"),
        worth_proof::TransitionOutcome::Failed(cause) => {
            panic!("checkpoint failed to start: {cause:?}")
        }
    };
    match handle.wait() {
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(completed) => completed,
        other => panic!("checkpoint failed: {other:?}"),
    }
}

pub(crate) struct IndependentCheckpointReopenCounters {
    pub(crate) artifact_bytes: u64,
    pub(crate) bytes_read: u64,
    pub(crate) dirty_bytes: u64,
    pub(crate) binding_records: u64,
    pub(crate) certificate_records: u64,
}

pub(crate) fn inspect_checkpoint_reopen(root: &Path) -> IndependentCheckpointReopenCounters {
    let bytes = std::fs::read(root.join("families/checkpoint.current")).unwrap();
    let records = checkpoint_records(&bytes);
    let compaction_index = records
        .iter()
        .position(|record| record[9] == 3)
        .expect("checkpoint carries a binding-compaction header");
    let dirty_bytes = records[1..compaction_index]
        .iter()
        .map(|record| record.len() as u64)
        .sum::<u64>();
    // Tier (6) and release-custody (7) certificates follow the bindings.
    let tail = &records[compaction_index + 1..records.len() - 1];
    let certificate_records = tail
        .iter()
        .filter(|record| matches!(record[9], 6 | 7))
        .count() as u64;
    IndependentCheckpointReopenCounters {
        artifact_bytes: bytes.len() as u64,
        bytes_read: bytes.len() as u64 - dirty_bytes,
        dirty_bytes,
        binding_records: tail.len() as u64 - certificate_records,
        certificate_records,
    }
}

pub(crate) fn checkpoint_records(bytes: &[u8]) -> Vec<&[u8]> {
    let mut records = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        assert!(bytes.len() - offset >= 20);
        let payload = u32::from_le_bytes(bytes[offset + 12..offset + 16].try_into().unwrap());
        let end = offset + 16 + payload as usize + 4;
        assert!(end <= bytes.len());
        records.push(&bytes[offset..end]);
        offset = end;
    }
    records
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
