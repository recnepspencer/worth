#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_reclaim::retirement_tests::*;
use {std::num::NonZeroU16, std::num::NonZeroU32};

use worth_proof::TransitionOutcome;
use {
    worth_store::physical_runtime::BlobReclaimContinuationFailure,
    worth_store::physical_runtime::BlobReclaimRetirementBudget,
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
    worth_store::physical_runtime::PhysicalRetirementDenial,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
};

use {
    crate::physical_blob_journeys::blob_abort::deadline,
    crate::physical_blob_journeys::blob_abort::limits as terminal_limits,
    crate::physical_blob_journeys::blob_crash::establish_recovery_frontier,
    crate::physical_blob_journeys::blob_reclaim::abandoned_prefix,
    crate::physical_blob_journeys::blob_reclaim::assert_intact,
    crate::physical_blob_journeys::blob_reclaim::request,
    crate::physical_blob_journeys::blob_reclaim::CHUNK, std::num::NonZeroU64,
    worth_store::physical_runtime::AdmittedBlobScope,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimReceipt,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::BlobReclaimRetirement,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::ServingPhysicalRuntime, worth_store_blob_chunks::BlobChunkSize,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
};

#[test]
fn pending_retirement_continues_only_on_issuing_store_and_credits_once() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.blob.reclaim.retirement-continuation.scope");
    let serving = serving_from_initialization(directory.path());
    let mut receipt = waiting_receipt(&serving, &scope);
    let expected = receipt.displaced_extents()[0].range().length();
    let budget = continuation_budget();
    let other_directory = tempfile::tempdir().unwrap();
    let other = serving_from_initialization(other_directory.path());
    let other_appends = other.media_counters().append_attempts();
    let original_appends = serving.media_counters().append_attempts();
    let retirement = receipt.retirement();
    assert_eq!(
        other
            .blobs()
            .unwrap()
            .continue_reclaim_retirement(&mut receipt, budget),
        Err(worth_store::physical_runtime::BlobReclaimContinuationFailure::ForeignStoreOrRuntime)
    );
    assert_eq!(other.media_counters().append_attempts(), other_appends);
    assert_eq!(serving.media_counters().append_attempts(), original_appends);
    assert_eq!(receipt.bytes_released(), 0);
    assert_eq!(receipt.retirement(), retirement);
    other.close();

    serving.certification_release_owed_background_turn();
    assert_eq!(
        serving
            .blobs()
            .unwrap()
            .continue_reclaim_retirement(&mut receipt, budget)
            .unwrap(),
        worth_store::physical_runtime::BlobReclaimRetirement::Completed
    );
    assert_eq!(receipt.bytes_released(), expected);
    assert_eq!(
        serving
            .blobs()
            .unwrap()
            .continue_reclaim_retirement(&mut receipt, budget)
            .unwrap(),
        worth_store::physical_runtime::BlobReclaimRetirement::Completed
    );
    assert_eq!(receipt.bytes_released(), expected);
    serving.close();
    assert_intact(directory.path(), "retirement-continuation");
}

#[test]
fn generic_retirement_can_settle_a_pending_reclaim_receipt() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.blob.reclaim.external-retirement.scope");
    let serving = serving_from_initialization(directory.path());
    let mut receipt = waiting_receipt(&serving, &scope);
    let expected = receipt.displaced_extents()[0].range().length();

    serving.certification_release_owed_background_turn();
    serving.retire_displaced_segment().unwrap();
    assert_eq!(receipt.bytes_released(), 0);
    assert_eq!(
        serving
            .blobs()
            .unwrap()
            .continue_reclaim_retirement(&mut receipt, continuation_budget())
            .unwrap(),
        worth_store::physical_runtime::BlobReclaimRetirement::Completed
    );
    assert_eq!(receipt.bytes_released(), expected);
    assert_eq!(
        serving
            .blobs()
            .unwrap()
            .continue_reclaim_retirement(&mut receipt, continuation_budget())
            .unwrap(),
        worth_store::physical_runtime::BlobReclaimRetirement::Completed
    );
    assert_eq!(receipt.bytes_released(), expected);
    serving.close();
    assert_intact(directory.path(), "external-retirement-continuation");
}

#[test]
fn partially_external_batch_preserves_exact_remaining_credit() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.blob.reclaim.partial-retirement.scope");
    let serving = serving_from_initialization(directory.path());
    establish_recovery_frontier(&serving);
    let token = abandoned_prefix(&serving, &scope);
    let frontier = serving
        .blobs()
        .unwrap()
        .reclaim(request(token, &scope))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(
        frontier.retirement(),
        worth_store::physical_runtime::BlobReclaimRetirement::Completed
    );
    assert_eq!(frontier.remaining_payload_records(), 2);

    let batch_request = worth_store::physical_runtime::BlobReclaimRequest::abandoned(
        token,
        &scope,
        placement(),
        deadline(),
        worth_store::physical_runtime::BlobReclaimLimits::new(
            std::num::NonZeroU64::new(128).unwrap(),
            std::num::NonZeroU64::new(8 << 20).unwrap(),
            std::num::NonZeroU16::new(2).unwrap(),
        )
        .unwrap(),
    );
    serving.certification_owe_before_retirement_barrier();
    let mut receipt = serving
        .blobs()
        .unwrap()
        .reclaim(batch_request)
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(receipt.displaced_extents().len(), 2);
    assert_eq!(
        receipt.retirement(),
        worth_store::physical_runtime::BlobReclaimRetirement::Pending(
            worth_store::physical_runtime::PhysicalRetirementDenial::Waiting
        )
    );
    let exact_total = receipt
        .displaced_extents()
        .iter()
        .map(|extent| extent.range().length())
        .sum::<u64>();

    serving.certification_release_owed_background_turn();
    serving.retire_displaced_segment().unwrap();
    serving.certification_owe_before_retirement_barrier();
    assert_eq!(
        serving
            .blobs()
            .unwrap()
            .continue_reclaim_retirement(&mut receipt, continuation_budget())
            .unwrap(),
        worth_store::physical_runtime::BlobReclaimRetirement::Pending(
            worth_store::physical_runtime::PhysicalRetirementDenial::Waiting
        )
    );
    let first_credit = receipt.bytes_released();
    assert!(first_credit > 0 && first_credit < exact_total);
    assert!(receipt
        .displaced_extents()
        .iter()
        .any(|extent| extent.range().length() == first_credit));

    serving.certification_release_owed_background_turn();
    assert_eq!(
        serving
            .blobs()
            .unwrap()
            .continue_reclaim_retirement(&mut receipt, continuation_budget())
            .unwrap(),
        worth_store::physical_runtime::BlobReclaimRetirement::Completed
    );
    assert_eq!(receipt.bytes_released(), exact_total);
    serving.close();
    assert_intact(directory.path(), "partial-external-retirement");
}

#[test]
fn isolated_range_index_pressure_keeps_exact_reclaim_retirement_pending() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.blob.reclaim.tight-index.scope");
    let serving = serving_from_initialization(directory.path());
    establish_recovery_frontier(&serving);
    let blobs = serving.blobs().unwrap();
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (CHUNK * 18) as u64,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(32).unwrap(),
        deadline(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK as u64, limits)
        .unwrap();
    // Interleaving one frontier extent with each chunk creates isolated free
    // runs. The default 64 KiB index has only 15 entries after reconstruction.
    for ordinal in 0..17 {
        ingest.push(&[ordinal as u8; CHUNK]).unwrap();
        ingest.checkpoint().unwrap();
    }
    let token = ingest.resume_token();
    drop(ingest);
    blobs
        .abort_ingest(token, &scope, placement(), deadline(), terminal_limits())
        .unwrap();
    let request = worth_store::physical_runtime::BlobReclaimRequest::abandoned(
        token,
        &scope,
        placement(),
        deadline(),
        worth_store::physical_runtime::BlobReclaimLimits::new(
            std::num::NonZeroU64::new(128).unwrap(),
            std::num::NonZeroU64::new(16 << 20).unwrap(),
            std::num::NonZeroU16::new(32).unwrap(),
        )
        .unwrap(),
    );
    let mut receipt = blobs.reclaim(request).unwrap().wait().unwrap();
    assert!(receipt.displaced_extents().len() > 15, "{receipt:?}");
    // The reclaim's own retirement batch is bounded by a wall-clock deadline,
    // so under load it can stop before reaching a verdict. Every continuation
    // makes at least one native attempt, so one per extent reaches it.
    for _ in 0..receipt.displaced_extents().len() {
        if receipt.retirement()
            != worth_store::physical_runtime::BlobReclaimRetirement::AwaitingRetirement
        {
            break;
        }
        blobs
            .continue_reclaim_retirement(&mut receipt, continuation_budget())
            .unwrap();
    }
    assert_eq!(
        receipt.retirement(),
        worth_store::physical_runtime::BlobReclaimRetirement::Pending(
            worth_store::physical_runtime::PhysicalRetirementDenial::ArenaIndexCapacity
        ),
        "{receipt:?}"
    );
    let credited = receipt.bytes_released();
    let total = receipt
        .displaced_extents()
        .iter()
        .map(|extent| extent.range().length())
        .sum::<u64>();
    assert!(credited > 0 && credited < total);
    for _ in 0..3 {
        assert_eq!(
            blobs
                .continue_reclaim_retirement(&mut receipt, continuation_budget())
                .unwrap(),
            worth_store::physical_runtime::BlobReclaimRetirement::Pending(
                worth_store::physical_runtime::PhysicalRetirementDenial::ArenaIndexCapacity
            )
        );
        assert_eq!(receipt.bytes_released(), credited, "no invented credit");
    }
    serving.close();

    // Seventeen chunk/checkpoint pairs exceed the three-chunk crash profile's
    // rejoin envelope; this world is admitted by the multilevel profile.
    crate::blob_crash::recovery::recover_closed_store_with_profile(
        directory.path(),
        "c11-blob-multilevel-v1",
    );
    let reopened = serving_from_open(directory.path());
    assert_eq!(
        reopened
            .blobs()
            .unwrap()
            .continue_reclaim_retirement(&mut receipt, continuation_budget()),
        Err(worth_store::physical_runtime::BlobReclaimContinuationFailure::ForeignStoreOrRuntime),
        "the old runtime's receipt cannot be reused after reopen"
    );
    assert_eq!(receipt.bytes_released(), credited);
    let (format, _, _) = worth_store_testkit_store::fixture::configuration();
    let larger = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(64).unwrap())
        .arena_index_bytes(worth_store::physical_runtime::RecordByteLimit::new(256 * 1024).unwrap())
        .admit(format)
        .unwrap();
    let submission = reopened.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([0xa7; 32]),
        )
        .unwrap();
    let new_extent = vec![0xa7; CHUNK];
    let worth_proof::TransitionOutcome::Success(
        worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(append),
    ) = submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter(
                [new_extent.as_slice()],
            )
            .unwrap(),
            larger,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                deadline(),
            ),
        )
        .into_raw()
    else {
        panic!("fresh extent append must admit a larger allocator index");
    };
    let worth_store::physical_runtime::PhysicalMutationOutcome::Completed(appended) =
        append.execute()
    else {
        panic!("fresh extent append must publish independently");
    };
    assert_eq!(appended.persisted_records().len(), 1);
    let mut settled = false;
    for _ in 0..64 {
        match reopened.retire_displaced_segment() {
            Ok(()) => {}
            Err(worth_store::physical_runtime::PhysicalRetirementDenial::Absent) => {
                settled = true;
                break;
            }
            other => panic!("expanded fresh owner failed to settle: {other:?}"),
        }
    }
    assert!(settled, "retained native obligations did not drain");
    assert_eq!(
        reopened.retire_displaced_segment(),
        Err(worth_store::physical_runtime::PhysicalRetirementDenial::Absent),
        "completed extents may not retire twice"
    );
    reopened.close();
}
