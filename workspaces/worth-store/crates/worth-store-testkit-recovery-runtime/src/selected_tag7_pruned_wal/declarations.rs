//! A native cutover may retain tag-7 custody after its old C.9 WAL is gone.

pub use {std::fs, std::num::NonZeroU64, std::path::Path};

pub use {worth_proof::TransitionOutcome};
pub use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::ManifestEntryCapacity, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::RecordAppendBatch};
pub use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::WorthStoreRecovery};

pub fn rotate_completed_wal(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
) {
    let format = worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(64).unwrap())
        .admit(worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(format))
        .unwrap();
    let submission = serving.record_submission();
    for ordinal in 0..24_u8 {
        let key = submission
            .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(
                [0xe0 + ordinal; 32],
            ))
            .unwrap();
        let payload = vec![ordinal; 64 * 1024];
        let worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) =
            submission
                .prepare_durable_append(
                    worth_store::physical_runtime::RecordAppendBatch::try_from_iter([payload.as_slice()]).unwrap(),
                    placement,
                    worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                        key,
                        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                    ),
                )
                .into_raw()
        else {
            panic!("completed ordinary WAL rotation {ordinal} must prepare")
        };
        assert!(matches!(
            prepared.execute(),
            worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_)
        ));
    }
}

pub fn retained_wal_interval(root: &Path, start: u64, end: u64) -> bool {
    const HEADER: usize = 116;
    const FOOTER: usize = 32;
    for entry in std::fs::read_dir(root.join("families/wal")).unwrap() {
        let bytes = std::fs::read(entry.unwrap().path()).unwrap();
        let mut offset = 0_usize;
        while offset < bytes.len() {
            let header = bytes
                .get(offset..offset + HEADER)
                .expect("complete WAL header");
            assert_eq!(&header[..8], b"WORTHWAL");
            let frame_start = u64::from_le_bytes(header[28..36].try_into().unwrap());
            let frame_end = u64::from_le_bytes(header[36..44].try_into().unwrap());
            let payload =
                usize::try_from(u64::from_le_bytes(header[44..52].try_into().unwrap())).unwrap();
            let next = offset
                .checked_add(HEADER)
                .and_then(|value| value.checked_add(payload))
                .and_then(|value| value.checked_add(FOOTER))
                .expect("bounded WAL frame extent");
            assert!(next <= bytes.len() && frame_start < frame_end);
            if frame_start == start && frame_end == end {
                return true;
            }
            offset = next;
        }
    }
    false
}