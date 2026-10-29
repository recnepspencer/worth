//! Every dependency of a terminal head retired, live one at a time. Each
//! denial precedes any WAL, root or release-ledger effect, and the same
//! retirement succeeds once the dependency is gone.

pub(crate) use {std::num::NonZeroU64, std::thread, std::time::Duration, std::time::Instant};

pub(crate) use {worth_proof::AdmittedBlobReleaseProof, worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::production::PhysicalMutationCheckpoint, worth_store::physical_runtime::BlobAppendFailure, worth_store::physical_runtime::BlobReclaimDisposition, worth_store::physical_runtime::BlobReclaimRetirement, worth_store::physical_runtime::CertificationScopedAllocation, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalOperationAllocationScope, worth_store::physical_runtime::PhysicalReadProtectionDenial, worth_store::physical_runtime::PhysicalRetirementDenial, worth_store::physical_runtime::PreparedPhysicalMutation, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::ServingPhysicalRuntime};

pub(crate) use {crate::physical_blob_journeys::terminal_head_retirement::checkpoint, crate::physical_blob_journeys::terminal_head_retirement::deadline, crate::physical_blob_journeys::terminal_head_retirement::denied, crate::physical_blob_journeys::terminal_head_retirement::publish, crate::physical_blob_journeys::terminal_head_retirement::release_batch, crate::physical_blob_journeys::terminal_head_retirement::release_request, crate::physical_blob_journeys::terminal_head_retirement::release_to_terminal, crate::physical_blob_journeys::terminal_head_retirement::retire, crate::physical_blob_journeys::terminal_head_retirement::settle, worth_store::physical_runtime::BlobTerminalHeadRetirementDenial as Denial, worth_store::physical_runtime::BlobTerminalHeadRetirementFailure as Failure, crate::physical_blob_journeys::terminal_head_retirement::WHOLE_GENERATION};
pub(crate) use {worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization};

/// One Store whose only release head is terminal and checkpoint-attested.
pub(crate) fn attested_world() -> (
    tempfile::TempDir,
    ServingPhysicalRuntime,
    AdmittedBlobReleaseProof,
) {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let proof = publish(&serving, 0x31);
    release_to_terminal(&serving, &proof);
    (directory, serving, proof)
}

pub(crate) fn prepared_append(serving: &ServingPhysicalRuntime, seed: u8) -> PreparedPhysicalMutation {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([seed; 32]))
        .unwrap();
    let worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .prepare_durable_append(
                worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"terminal-head-contender".as_slice()]).unwrap(),
                placement(),
                worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(key, deadline()),
            )
            .into_raw()
    else {
        panic!("an ordinary append must prepare beside a settled release");
    };
    prepared
}

/// Holds every Recovery byte the pool would still grant except `spare`. The
/// bound is found against the real pool.
pub(crate) fn hold_recovery_bytes_leaving(
    serving: &ServingPhysicalRuntime,
    spare: u64,
) -> CertificationScopedAllocation {
    let residency = serving.certification_physical_residency();
    let admit = |bytes: u64| {
        residency.admit_operation_scope(
            worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery,
            std::num::NonZeroU64::new(bytes).unwrap(),
        )
    };
    let (mut granted, mut refused) = (1_u64, 1_u64 << 48);
    assert!(admit(granted).is_ok() && admit(refused).is_err());
    while refused - granted > 1 {
        let middle = granted + (refused - granted) / 2;
        if admit(middle).is_ok() {
            granted = middle;
        } else {
            refused = middle;
        }
    }
    let hold = admit(granted - spare).expect("the competing grant fits the pool");
    assert!(
        admit(spare).is_ok() && admit(spare + 1).is_err(),
        "the hold leaves the pool exactly {spare} bytes"
    );
    hold
}