use std::{fs, path::Path};

use worth_proof::{NonEmpty, TransitionOutcome};
use worth_signal::facade::TemporalDuration;
use worth_store::physical_runtime::{
    AdmittedRecordPlacementPolicy, CompletedPhysicalCheckpoint, PhysicalCheckpointDeadline,
    PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome, PhysicalCheckpointRequest,
    PhysicalMutationDeadline, PhysicalMutationIdempotencyKey, PhysicalMutationPreparationSuccess,
    PhysicalMutationRequest, PhysicalWalGroupAppendOutcome, PhysicalWalGroupBarrierOutcome,
    PreparedPhysicalMutation, SealedPhysicalDurabilityGroupMembers, ServingPhysicalRuntime,
};

pub(super) fn prepare(
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
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("physical mutation preparation must succeed"),
    }
}

pub(super) fn request(key: PhysicalMutationIdempotencyKey) -> PhysicalMutationRequest {
    PhysicalMutationRequest::platform_durable(
        key,
        PhysicalMutationDeadline::at(TemporalDuration::temporal_duration(1_000).unwrap()),
    )
}

pub(super) fn append_one(
    submission: &worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission,
    prepared: PreparedPhysicalMutation,
) -> SealedPhysicalDurabilityGroupMembers {
    match submission.append_prepared_wal_group(NonEmpty::new(prepared, Vec::new())) {
        PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("one-member WAL append must succeed"),
    }
}

pub(super) fn synchronize(
    submission: &worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission,
    appended: SealedPhysicalDurabilityGroupMembers,
) {
    assert!(matches!(
        submission.synchronize_appended_wal_group(appended),
        PhysicalWalGroupBarrierOutcome::Durable(_)
    ));
}

pub(super) fn success_checkpoint(
    serving: &ServingPhysicalRuntime,
    key: u8,
) -> CompletedPhysicalCheckpoint {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([key; 32]),
        PhysicalCheckpointDeadline::at(TemporalDuration::temporal_duration(1_000).unwrap()),
    );
    let handle = match serving.checkpoints().start(request).into_raw() {
        TransitionOutcome::Success(handle) => handle,
        TransitionOutcome::Denied(cause) => panic!("checkpoint denied: {cause:?}"),
        TransitionOutcome::Deferred(cause) => panic!("checkpoint deferred: {cause:?}"),
        TransitionOutcome::Stale(cause) => panic!("checkpoint stale: {cause:?}"),
        TransitionOutcome::Failed(cause) => panic!("checkpoint failed to start: {cause:?}"),
    };
    match handle.wait() {
        PhysicalCheckpointOutcome::Completed(completed) => completed,
        other => panic!("checkpoint failed: {other:?}"),
    }
}

pub(super) struct IndependentCheckpointReopenCounters {
    pub(super) artifact_bytes: u64,
    pub(super) bytes_read: u64,
    pub(super) dirty_bytes: u64,
    pub(super) binding_records: u64,
}

pub(super) fn inspect_checkpoint_reopen(root: &Path) -> IndependentCheckpointReopenCounters {
    let bytes = fs::read(root.join("families/checkpoint.current")).unwrap();
    let records = checkpoint_records(&bytes);
    let compaction_index = records
        .iter()
        .position(|record| record[9] == 3)
        .expect("checkpoint carries a binding-compaction header");
    let dirty_bytes = records[1..compaction_index]
        .iter()
        .map(|record| record.len() as u64)
        .sum::<u64>();
    IndependentCheckpointReopenCounters {
        artifact_bytes: bytes.len() as u64,
        bytes_read: bytes.len() as u64 - dirty_bytes,
        dirty_bytes,
        binding_records: (records.len() - compaction_index - 2) as u64,
    }
}

pub(super) fn checkpoint_records(bytes: &[u8]) -> Vec<&[u8]> {
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

pub(super) fn reseal_record_crc(record: &mut [u8]) {
    let checksum_offset = record.len() - 4;
    let checksum = crc32c(&record[..checksum_offset]);
    record[checksum_offset..].copy_from_slice(&checksum.to_le_bytes());
}

fn crc32c(bytes: &[u8]) -> u32 {
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
