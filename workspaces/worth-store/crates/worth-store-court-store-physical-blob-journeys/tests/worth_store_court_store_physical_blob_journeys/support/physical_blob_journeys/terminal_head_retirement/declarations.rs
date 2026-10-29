//! Production terminal head retired, in one process. No world here reopens:
//! recovery of the retirement member belongs to the recovery slice.

pub(crate) use {std::num::NonZeroU16, std::num::NonZeroU32, std::num::NonZeroU64};

pub(crate) use {worth_proof::AdmittedBlobReleaseProof, worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::certification::CertificationReleaseHeadObservation, worth_store::physical_runtime::BlobReclaimDisposition, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimReceipt, worth_store::physical_runtime::BlobReclaimRequest, worth_store::physical_runtime::BlobReclaimRetirement, worth_store::physical_runtime::BlobReclaimRetirementBudget, worth_store::physical_runtime::BlobTerminalHeadRetirementDenial as Denial, worth_store::physical_runtime::BlobTerminalHeadRetirementFailure as Failure, worth_store::physical_runtime::BlobTerminalHeadRetirementReceipt, worth_store::physical_runtime::BlobTerminalHeadRetirementRequest, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalWalObservation, worth_store::physical_runtime::ServingPhysicalRuntime};

pub(crate) use {worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization};
pub(crate) use {crate::physical_blob_journeys::terminal_head_retirement::publication::again, crate::physical_blob_journeys::terminal_head_retirement::publication::begin, crate::physical_blob_journeys::terminal_head_retirement::publication::publish_chunks, crate::physical_blob_journeys::terminal_head_retirement::publication::publish_resumable, crate::physical_blob_journeys::terminal_head_retirement::publication::Published};






pub(crate) const SCOPE: &str = "c11.blob.terminal.head.retired.scope";
pub(crate) const CHUNK: usize = 64 << 10;
/// The shortest horizon a declaration admits. A release completes more
/// checkpoints than this, so such a declaration never outlives its release.
pub(crate) const HORIZON: u64 = 1;
/// More records than one two-chunk generation owns: one drop is terminal.
pub(crate) const WHOLE_GENERATION: u16 = 16;

/// After its head is retired the object stays settled: a repeated reclaim is
/// proven no effect and mints no head, and a second retirement finds no head.
pub(crate) fn assert_settled_absence(serving: &ServingPhysicalRuntime, proof: &AdmittedBlobReleaseProof) {
    let before = fixed(serving);
    let repeated = serving
        .blobs()
        .unwrap()
        .reclaim(release_request(proof, WHOLE_GENERATION))
        .expect("a settled release still admits its no-effect answer")
        .wait()
        .expect("a settled release answers without effect");
    assert_eq!(
        repeated.disposition(),
        worth_store::physical_runtime::BlobReclaimDisposition::ProvenNoEffect
    );
    assert!(repeated.dropped_records().is_empty());
    assert_eq!(repeated.retirement(), worth_store::physical_runtime::BlobReclaimRetirement::NotRequired);
    assert_eq!(fixed(serving), before, "a settled reclaim minted an effect");
    assert!(matches!(
        denied(serving, proof),
        worth_store::physical_runtime::BlobTerminalHeadRetirementFailure::Denied(worth_store::physical_runtime::BlobTerminalHeadRetirementDenial::NoHead)
    ));
}

/// Runs one retirement that must deny, and proves the denial left the WAL
/// end, the selected root and the release ledger exactly as they were.
pub(crate) fn denied(serving: &ServingPhysicalRuntime, proof: &AdmittedBlobReleaseProof) -> Failure {
    let before = fixed(serving);
    let failure = retire(serving, proof).expect_err("the retirement must deny");
    assert_eq!(
        fixed(serving),
        before,
        "a denied retirement changed the WAL, the root or the release ledger: {failure:?}"
    );
    failure
}

pub(crate) fn fixed(
    serving: &ServingPhysicalRuntime,
) -> (
    PhysicalWalObservation,
    Option<CertificationReleaseHeadObservation>,
) {
    (
        wal(serving),
        serving.certification_release_head_observation(),
    )
}

pub(crate) fn wal(serving: &ServingPhysicalRuntime) -> PhysicalWalObservation {
    serving
        .record_submission()
        .wal_observation()
        .expect("live WAL observation")
}

pub(crate) fn observe(serving: &ServingPhysicalRuntime) -> CertificationReleaseHeadObservation {
    serving
        .certification_release_head_observation()
        .expect("selected release custody")
}

pub(crate) fn retire(
    serving: &ServingPhysicalRuntime,
    proof: &AdmittedBlobReleaseProof,
) -> Result<BlobTerminalHeadRetirementReceipt, Failure> {
    serving
        .blobs()
        .unwrap()
        .retire_terminal_head(worth_store::physical_runtime::BlobTerminalHeadRetirementRequest::new(
            again(proof),
            placement(),
            deadline(),
            limits(WHOLE_GENERATION),
        ))
}

/// Completes enough checkpoints, keyed from `first_key`, that every
/// declaration made before the call is durably expired: its session can never
/// resume, so only the release-side dependencies of its head are left.
pub(crate) fn expire_declarations(serving: &ServingPhysicalRuntime, first_key: u8) {
    for round in 0..=HORIZON as u8 {
        checkpoint(serving, first_key + round);
    }
}

/// Publishes one generation and durably expires its declaration.
pub(crate) fn publish(serving: &ServingPhysicalRuntime, seed: u8) -> AdmittedBlobReleaseProof {
    let published = publish_resumable(serving, seed, HORIZON);
    expire_declarations(serving, seed);
    published.proof
}

pub(crate) fn release_request(
    proof: &AdmittedBlobReleaseProof,
    batch_records: u16,
) -> BlobReclaimRequest<'static> {
    worth_store::physical_runtime::BlobReclaimRequest::released(again(proof), placement(), deadline(), limits(batch_records))
}

/// One drop batch of the release, with its displaced extents not yet settled.
pub(crate) fn release_batch(
    serving: &ServingPhysicalRuntime,
    proof: &AdmittedBlobReleaseProof,
    batch_records: u16,
) -> BlobReclaimReceipt {
    let receipt = serving
        .blobs()
        .unwrap()
        .reclaim(release_request(proof, batch_records))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(receipt.disposition(), worth_store::physical_runtime::BlobReclaimDisposition::Dropped);
    receipt
}

/// Drops the whole generation and settles every displaced extent, so its
/// head is terminal and the drop's own checkpoint has attested it.
pub(crate) fn release_to_terminal(serving: &ServingPhysicalRuntime, proof: &AdmittedBlobReleaseProof) {
    let mut receipt = release_batch(serving, proof, WHOLE_GENERATION);
    assert_eq!(receipt.remaining_payload_records(), 0);
    settle(serving, &mut receipt);
}

/// Continues the receipt's displaced-extent retirement to completion.
pub(crate) fn settle(serving: &ServingPhysicalRuntime, receipt: &mut BlobReclaimReceipt) {
    for _ in 0..64 {
        if receipt.retirement() == worth_store::physical_runtime::BlobReclaimRetirement::Completed {
            return;
        }
        serving
            .blobs()
            .unwrap()
            .continue_reclaim_retirement(
                receipt,
                worth_store::physical_runtime::BlobReclaimRetirementBudget::new(std::num::NonZeroU32::new(64).unwrap(), deadline()),
            )
            .unwrap();
    }
    panic!("the displaced extents never settled: {receipt:?}");
}

pub(crate) fn checkpoint(serving: &ServingPhysicalRuntime, key: u8) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([key; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let handle = match serving.checkpoints().start(request).into_raw() {
        worth_proof::TransitionOutcome::Success(handle) => handle,
        worth_proof::TransitionOutcome::Failed(failure) => panic!("checkpoint {key:#x} denied: {failure:?}"),
        _ => panic!("checkpoint {key:#x} did not start"),
    };
    let outcome = handle.wait();
    assert!(
        matches!(outcome, worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)),
        "checkpoint {key:#x} must complete: {outcome:?}"
    );
}

pub(crate) fn limits(batch_records: u16) -> BlobReclaimLimits {
    worth_store::physical_runtime::BlobReclaimLimits::new(
        std::num::NonZeroU64::new(256).unwrap(),
        std::num::NonZeroU64::new(64 << 20).unwrap(),
        std::num::NonZeroU16::new(batch_records).unwrap(),
    )
    .unwrap()
}

pub(crate) fn deadline() -> PhysicalMutationDeadline {
    worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap()
}