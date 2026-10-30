//! A non-global-tip A head survives native WAL cutover and authorizes A's successor.

pub(crate) use {std::fmt::Write, std::fs, std::path::Path, std::process::Command};
pub(crate) use {
    std::num::NonZeroU16, std::num::NonZeroU64, worth_proof::AdmittedBlobReleaseProof,
    worth_proof::TransitionOutcome, worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::ServingPhysicalRuntime,
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving,
    worth_store_testkit_recovery_runtime::production_entry::release_reopen::selected_release_certificates,
    worth_store_testkit_recovery_runtime::production_entry::release_reopen::selected_release_certificates_from_bytes,
    worth_store_testkit_recovery_runtime::production_entry::release_reopen::two_generations,
    worth_store_testkit_recovery_runtime::release_reopen::selected_head_oracle,
    worth_store_testkit_recovery_runtime::selected_tag7_pruned_wal,
};
pub(crate) use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordCountLimit,
    worth_store::physical_runtime::RecordScanOutcome,
    worth_store::physical_runtime::RecordScanRequest,
};
pub(crate) use {
    worth_store_physical_format::decode_blob_record,
    worth_store_physical_format::release_checkpoint_batch_records_digest_v1,
    worth_store_physical_format::BlobReclaimSourceBasisV1,
    worth_store_physical_format::BlobRecordV1,
    worth_store_physical_format::ReleaseCustodyHeadKeyV1,
    worth_store_physical_format::ReleasedGenerationReclaimBasisV1,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
};

pub(crate) const WAL_SEGMENT_BYTES: u64 = 1 << 20;
pub(crate) const CHILD_TEST: &str = "process::recovery_runtime::production_entry::release_reopen::per_object_pruned_successor::fresh_child_continues_a";
pub(crate) const ROOT_ENV: &str = "WORTH_C11_PRUNED_A_ROOT";
pub(crate) const OBJECT_ENV: &str = "WORTH_C11_PRUNED_A_OBJECT";
pub(crate) const GENERATION_ENV: &str = "WORTH_C11_PRUNED_A_GENERATION";

pub(crate) fn recover_serving(
    root: &Path,
    wal_bytes: NonZeroU64,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    let outcome =
        worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(root));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        match outcome {
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(block) => panic!(
                "A/B recovery blocked at {:?}: {:?}; limit={:?}; effects={}",
                block.cause(),
                block.evidence().planning_denial,
                block.cause().limit(),
                block.recovery_effects(),
            ),
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
                failure,
            ) => panic!(
                "A/B recovery indeterminate: reopen={:?}; handoff={:?}; effects={}",
                failure.reopen_failure(),
                failure.handoff_failure(),
                failure.recovery_effects(),
            ),
            other => panic!("fresh process must recover A/B selected custody: {other:?}"),
        }
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("Store seal");
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal_and_wal_segment_bytes(
        root, seal, wal_bytes,
    )
}

pub(crate) fn checkpoint(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    key: [u8; 32],
) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new(key),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        serving.checkpoints().start(request).into_raw()
    else {
        panic!("native checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
}

pub(crate) fn selected_source_for_key(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    key: ReleaseCustodyHeadKeyV1,
) -> ReleasedGenerationReclaimBasisV1 {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            worth_store::physical_runtime::RecordScanRequest::from_start()
                .with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(8).unwrap())
                .with_payload_limit(
                    worth_store::physical_runtime::RecordByteLimit::new(4096).unwrap(),
                ),
        )
        .unwrap();
    let mut scratch = [0_u8; 8192];
    let mut matches = Vec::new();
    while let worth_store::physical_runtime::RecordScanOutcome::Batch(batch) =
        scan.read_next_into(&mut scratch).unwrap()
    {
        for index in 0..batch.records().len() {
            let Some(bytes) = batch.payload(index) else {
                continue;
            };
            let Ok(worth_store_physical_format::BlobRecordV1::DropSetManifestV3(manifest)) =
                decode_blob_record(bytes)
            else {
                continue;
            };
            let worth_store_physical_format::BlobReclaimSourceBasisV1::ReleasedGeneration(source) =
                manifest.source_basis()
            else {
                continue;
            };
            if source.object() == key.object() && source.generation() == key.generation() {
                matches.push(source);
            }
        }
        if batch.is_complete() {
            break;
        }
    }
    let [source] = matches.as_slice() else {
        panic!("exactly one selected A source basis must match the recorded keyed head")
    };
    *source
}

pub(crate) fn encode_object(object: [u8; 16]) -> String {
    let mut encoded = String::new();
    for byte in object {
        write!(&mut encoded, "{byte:02x}").unwrap();
    }
    encoded
}

pub(crate) fn decode_object(encoded: &str) -> [u8; 16] {
    assert_eq!(encoded.len(), 32);
    std::array::from_fn(|index| u8::from_str_radix(&encoded[index * 2..index * 2 + 2], 16).unwrap())
}

pub(crate) fn released_world_with_wal_segment_bytes(
    maximum_dropped_records: u16,
    wal_segment_bytes: NonZeroU64,
) -> (
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
    worth_store::physical_runtime::BlobReclaimReceipt,
    worth_store_physical_format::PersistedRecordIdentity,
    AdmittedBlobReleaseProof,
) {
    worth_store_testkit_recovery_runtime::production_entry::release_reopen::released_world_in_tier(
        maximum_dropped_records,
        false,
        Some(wal_segment_bytes),
    )
}
