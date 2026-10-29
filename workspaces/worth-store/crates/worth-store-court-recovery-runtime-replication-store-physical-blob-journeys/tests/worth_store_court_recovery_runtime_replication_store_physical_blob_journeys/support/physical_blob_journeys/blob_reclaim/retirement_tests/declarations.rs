pub(crate) use {std::num::NonZeroU16, std::num::NonZeroU32};

pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::BlobReclaimContinuationFailure, worth_store::physical_runtime::BlobReclaimRetirementBudget, worth_store::physical_runtime::ManifestEntryCapacity, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::PhysicalRetirementDenial, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordByteLimit};

pub(crate) use {worth_store::physical_runtime::AdmittedBlobScope, worth_store::physical_runtime::BlobCheckpointLimit, worth_store_blob_chunks::BlobChunkSize, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimReceipt, worth_store::physical_runtime::BlobReclaimRequest, worth_store::physical_runtime::BlobReclaimRetirement, crate::physical_blob_journeys::blob_reclaim::CHUNK, std::num::NonZeroU64, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::ServingPhysicalRuntime, crate::physical_blob_journeys::blob_reclaim::abandoned_prefix, worth_store_testkit_store::fixture::admitted_blob_scope, crate::physical_blob_journeys::blob_reclaim::assert_intact, crate::physical_blob_journeys::blob_abort::deadline, crate::physical_blob_journeys::blob_crash::establish_recovery_frontier, worth_store_testkit_store::fixture::placement, crate::physical_blob_journeys::blob_reclaim::request, worth_store_testkit_store::fixture::serving_from_initialization, worth_store_testkit_store::fixture::serving_from_open, crate::physical_blob_journeys::blob_abort::limits as terminal_limits};

pub(crate) fn continuation_budget() -> BlobReclaimRetirementBudget {
    worth_store::physical_runtime::BlobReclaimRetirementBudget::new(
        std::num::NonZeroU32::new(2).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
}

pub(crate) fn waiting_receipt(
    serving: &ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
) -> BlobReclaimReceipt {
    establish_recovery_frontier(serving);
    let token = abandoned_prefix(serving, scope);
    serving.certification_owe_before_retirement_barrier();
    let receipt = serving
        .blobs()
        .unwrap()
        .reclaim(request(token, scope))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(
        receipt.retirement(),
        worth_store::physical_runtime::BlobReclaimRetirement::Pending(worth_store::physical_runtime::PhysicalRetirementDenial::Waiting),
        "{receipt:?}"
    );
    assert_eq!(receipt.bytes_released(), 0);
    assert_eq!(receipt.displaced_extents().len(), 1);
    receipt
}