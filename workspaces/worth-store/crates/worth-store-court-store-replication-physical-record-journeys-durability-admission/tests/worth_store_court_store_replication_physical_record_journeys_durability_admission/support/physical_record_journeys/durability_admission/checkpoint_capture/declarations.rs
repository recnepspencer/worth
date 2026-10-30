pub(crate) use std::fs;

pub(crate) use worth_signal::facade::TemporalDuration;
pub(crate) use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
pub(crate) use {
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalCheckpointStartDenial,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalWalGroupAppendOutcome,
    worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome,
};
pub(crate) use {
    worth_store_physical_format::decode_checkpoint_certificate,
    worth_store_physical_format::CheckpointCertificateKind,
    worth_store_physical_format::ReleaseCheckpointCertificateV1,
};

pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
};

pub(crate) fn checkpoint_request(key: u8) -> PhysicalCheckpointRequest {
    worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([key; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::at(
            worth_signal::facade::TemporalDuration::temporal_duration(1_000)
                .expect("deadline is positive"),
        ),
    )
}

pub(crate) fn checkpoint_records(bytes: &[u8]) -> Vec<&[u8]> {
    let mut records = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        assert!(
            bytes.len() - offset >= 20,
            "checkpoint record prefix is truncated"
        );
        let payload = u32::from_le_bytes(bytes[offset + 12..offset + 16].try_into().unwrap());
        let end = offset + 16 + payload as usize + 4;
        assert!(end <= bytes.len(), "checkpoint record exceeds the artifact");
        records.push(&bytes[offset..end]);
        offset = end;
    }
    assert!(
        records.len() >= 2,
        "checkpoint requires a header and footer"
    );
    records
}
