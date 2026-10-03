use std::num::{NonZeroU16, NonZeroU32};

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    BlobReclaimContinuationFailure, BlobReclaimRetirementBudget, ManifestEntryCapacity,
    PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PhysicalRecordPlacementPolicy,
    PhysicalRetirementDenial, RecordAppendBatch, RecordByteLimit,
};

use super::*;

fn continuation_budget() -> BlobReclaimRetirementBudget {
    BlobReclaimRetirementBudget::new(
        NonZeroU32::new(2).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
}

fn waiting_receipt(
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
        BlobReclaimRetirement::Pending(PhysicalRetirementDenial::Waiting),
        "{receipt:?}"
    );
    assert_eq!(receipt.bytes_released(), 0);
    assert_eq!(receipt.displaced_extents().len(), 1);
    receipt
}

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
        Err(BlobReclaimContinuationFailure::ForeignStoreOrRuntime)
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
        BlobReclaimRetirement::Completed
    );
    assert_eq!(receipt.bytes_released(), expected);
    assert_eq!(
        serving
            .blobs()
            .unwrap()
            .continue_reclaim_retirement(&mut receipt, budget)
            .unwrap(),
        BlobReclaimRetirement::Completed
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
        BlobReclaimRetirement::Completed
    );
    assert_eq!(receipt.bytes_released(), expected);
    assert_eq!(
        serving
            .blobs()
            .unwrap()
            .continue_reclaim_retirement(&mut receipt, continuation_budget())
            .unwrap(),
        BlobReclaimRetirement::Completed
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
    assert_eq!(frontier.retirement(), BlobReclaimRetirement::Completed);
    assert_eq!(frontier.remaining_payload_records(), 2);

    let batch_request = BlobReclaimRequest::abandoned(
        token,
        &scope,
        placement(),
        deadline(),
        BlobReclaimLimits::new(
            NonZeroU64::new(128).unwrap(),
            NonZeroU64::new(8 << 20).unwrap(),
            NonZeroU16::new(2).unwrap(),
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
        BlobReclaimRetirement::Pending(PhysicalRetirementDenial::Waiting)
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
        BlobReclaimRetirement::Pending(PhysicalRetirementDenial::Waiting)
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
        BlobReclaimRetirement::Completed
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
    let limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (CHUNK * 18) as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(32).unwrap(),
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
    let request = BlobReclaimRequest::abandoned(
        token,
        &scope,
        placement(),
        deadline(),
        BlobReclaimLimits::new(
            NonZeroU64::new(128).unwrap(),
            NonZeroU64::new(16 << 20).unwrap(),
            NonZeroU16::new(32).unwrap(),
        )
        .unwrap(),
    );
    let mut receipt = blobs.reclaim(request).unwrap().wait().unwrap();
    assert!(receipt.displaced_extents().len() > 15, "{receipt:?}");
    assert_eq!(
        receipt.retirement(),
        BlobReclaimRetirement::Pending(PhysicalRetirementDenial::ArenaIndexCapacity),
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
            BlobReclaimRetirement::Pending(PhysicalRetirementDenial::ArenaIndexCapacity)
        );
        assert_eq!(receipt.bytes_released(), credited, "no invented credit");
    }
    serving.close();

    // Seventeen chunk/checkpoint pairs exceed the three-chunk crash profile's
    // rejoin envelope; this world is admitted by the multilevel profile.
    super::super::blob_crash::recover_closed_store_with_profile(
        directory.path(),
        "c11-blob-multilevel-v1",
    );
    let reopened = serving_from_open(directory.path());
    assert_eq!(
        reopened
            .blobs()
            .unwrap()
            .continue_reclaim_retirement(&mut receipt, continuation_budget()),
        Err(BlobReclaimContinuationFailure::ForeignStoreOrRuntime),
        "the old runtime's receipt cannot be reused after reopen"
    );
    assert_eq!(receipt.bytes_released(), credited);
    let (format, _, _) = super::super::fixture::configuration();
    let larger = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
        .arena_index_bytes(RecordByteLimit::new(256 * 1024).unwrap())
        .admit(format)
        .unwrap();
    let submission = reopened.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0xa7; 32]))
        .unwrap();
    let new_extent = vec![0xa7; CHUNK];
    let TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(append)) =
        submission
            .prepare_durable_append(
                RecordAppendBatch::try_from_iter([new_extent.as_slice()]).unwrap(),
                larger,
                PhysicalMutationRequest::platform_durable(key, deadline()),
            )
            .into_raw()
    else {
        panic!("fresh extent append must admit a larger allocator index");
    };
    let PhysicalMutationOutcome::Completed(appended) = append.execute() else {
        panic!("fresh extent append must publish independently");
    };
    assert_eq!(appended.persisted_records().len(), 1);
    let mut settled = false;
    for _ in 0..64 {
        match reopened.retire_displaced_segment() {
            Ok(()) => {}
            Err(PhysicalRetirementDenial::Absent) => {
                settled = true;
                break;
            }
            other => panic!("expanded fresh owner failed to settle: {other:?}"),
        }
    }
    assert!(settled, "retained native obligations did not drain");
    assert_eq!(
        reopened.retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Absent),
        "completed extents may not retire twice"
    );
    reopened.close();
}
