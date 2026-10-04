//! Production terminal head retired, in one process. No world here reopens:
//! recovery of the retirement member belongs to the recovery slice.

use std::num::{NonZeroU16, NonZeroU32, NonZeroU64};

use worth_proof::{AdmittedBlobReleaseProof, TransitionOutcome};
use worth_store::physical_runtime::{
    certification::CertificationReleaseHeadObservation, BlobCheckpointLimit, BlobIngestDeclaration,
    BlobIngestFailure, BlobIngestSession, BlobObjectId, BlobReadLimits, BlobReclaimDisposition,
    BlobReclaimLimits, BlobReclaimReceipt, BlobReclaimRequest, BlobReclaimRetirement,
    BlobReclaimRetirementBudget, BlobResumeToken, BlobTerminalHeadRetirementDenial as Denial,
    BlobTerminalHeadRetirementFailure as Failure, BlobTerminalHeadRetirementReceipt,
    BlobTerminalHeadRetirementRequest, PhysicalCheckpointDeadline,
    PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome, PhysicalCheckpointRequest,
    PhysicalMutationDeadline, PhysicalWalObservation, ServingPhysicalRuntime,
};
use worth_store_blob_chunks::BlobChunkSize;

use super::fixture::{admitted_blob_scope, placement, serving_from_initialization};

#[path = "terminal_head_retirement/denials.rs"]
mod denials;
#[path = "terminal_head_retirement/non_reissue.rs"]
mod non_reissue;

const SCOPE: &str = "c11.blob.terminal.head.retired.scope";
const CHUNK: usize = 64 << 10;
/// The shortest horizon a declaration admits. A release completes more
/// checkpoints than this, so such a declaration never outlives its release.
const HORIZON: u64 = 1;
/// More records than one two-chunk generation owns: one drop is terminal.
const WHOLE_GENERATION: u16 = 16;

#[test]
fn retirement_removes_exactly_the_attested_head_and_the_next_checkpoint_ratchets_the_roster() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let first = publish(&serving, 0x31);
    let second = publish(&serving, 0x47);
    release_to_terminal(&serving, &first);
    release_to_terminal(&serving, &second);
    let source = observe(&serving);
    assert_eq!(source.effective_heads().0, 2);
    assert_eq!(source.checkpoint_heads(), source.effective_heads());
    assert_eq!(source.pending_retirements(), 0);
    let source_wal = wal(&serving);

    let receipt = retire(&serving, &first).expect("every dependency is settled");
    assert_eq!(receipt.object(), first.object());
    assert_eq!(receipt.generation(), first.generation());
    assert_eq!(receipt.source_root_generation(), source.root_generation());
    assert!(!receipt.head_tree_emptied());
    let retired = observe(&serving);
    assert_eq!(receipt.result_root_generation(), retired.root_generation());
    assert!(
        retired.is_head_only_successor_of(&source),
        "the result root may differ only in the head root and head block frontier:
{source:?}
{retired:?}"
    );
    assert!(retired.has_head_tree());
    assert_eq!(retired.effective_heads().0, 1);
    assert_ne!(retired.effective_heads().1, source.effective_heads().1);
    assert_eq!(retired.checkpoint_heads(), source.checkpoint_heads());
    assert_eq!(retired.pending_retirements(), 1);
    assert_eq!(retired.pending_drops(), source.pending_drops());
    assert!(wal(&serving).last_lsn_end() > source_wal.last_lsn_end());

    assert_settled_absence(&serving, &first);
    checkpoint(&serving, 0x82);
    let ratcheted = observe(&serving);
    assert_eq!(ratcheted.effective_heads(), retired.effective_heads());
    assert_eq!(ratcheted.checkpoint_heads(), retired.effective_heads());
    assert_eq!(ratcheted.pending_retirements(), 0);
    assert_settled_absence(&serving, &first);

    let last = retire(&serving, &second).expect("the other head is settled as well");
    assert!(last.head_tree_emptied());
    let emptied = observe(&serving);
    assert!(emptied.is_head_only_successor_of(&ratcheted));
    assert!(!emptied.has_head_tree());
    assert_eq!(emptied.effective_heads().0, 0);
    assert_eq!(emptied.checkpoint_heads(), ratcheted.checkpoint_heads());
    assert_settled_absence(&serving, &second);
    checkpoint(&serving, 0x83);
    let empty = observe(&serving);
    assert_eq!(empty.checkpoint_heads(), emptied.effective_heads());
    assert_eq!(empty.pending_retirements(), 0);
    assert_settled_absence(&serving, &second);
    assert_settled_absence(&serving, &first);
    serving.close();
}

/// Each retirement funds its own pending event at admission, before the WAL
/// effect. The commit after the effect only takes the funded slot, so a
/// whole roster retires inside one checkpoint window.
#[test]
fn a_roster_retired_inside_one_checkpoint_window_funds_each_pending_event_before_its_effect() {
    const SEEDS: [u8; 4] = [0x11, 0x21, 0x31, 0x41];
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let proofs = SEEDS.map(|seed| publish(&serving, seed));
    for proof in &proofs {
        release_to_terminal(&serving, proof);
    }
    let source = observe(&serving);
    assert_eq!(source.effective_heads().0, SEEDS.len() as u64);
    assert_eq!(source.checkpoint_heads(), source.effective_heads());
    assert_eq!(
        (source.pending_drops(), source.pending_retirements()),
        (0, 0)
    );

    for (retired, proof) in proofs.iter().enumerate() {
        retire(&serving, proof).expect("the admission funded this retirement's pending event");
        let window = observe(&serving);
        assert_eq!(window.pending_retirements(), retired + 1);
        assert_eq!(
            window.effective_heads().0,
            (SEEDS.len() - retired - 1) as u64
        );
        assert_eq!(window.checkpoint_heads(), source.checkpoint_heads());
    }
    assert!(!observe(&serving).has_head_tree());
    checkpoint(&serving, 0x82);
    let ratcheted = observe(&serving);
    assert_eq!(ratcheted.checkpoint_heads(), ratcheted.effective_heads());
    assert_eq!(ratcheted.checkpoint_heads().0, 0);
    assert_eq!(ratcheted.pending_retirements(), 0);
    for proof in &proofs {
        assert_settled_absence(&serving, proof);
    }
    serving.close();
}

/// After its head is retired the object stays settled: a repeated reclaim is
/// proven no effect and mints no head, and a second retirement finds no head.
fn assert_settled_absence(serving: &ServingPhysicalRuntime, proof: &AdmittedBlobReleaseProof) {
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
        BlobReclaimDisposition::ProvenNoEffect
    );
    assert!(repeated.dropped_records().is_empty());
    assert_eq!(repeated.retirement(), BlobReclaimRetirement::NotRequired);
    assert_eq!(fixed(serving), before, "a settled reclaim minted an effect");
    assert!(matches!(
        denied(serving, proof),
        Failure::Denied(Denial::NoHead)
    ));
}

/// Runs one retirement that must deny, and proves the denial left the WAL
/// end, the selected root and the release ledger exactly as they were.
fn denied(serving: &ServingPhysicalRuntime, proof: &AdmittedBlobReleaseProof) -> Failure {
    let before = fixed(serving);
    let failure = retire(serving, proof).expect_err("the retirement must deny");
    assert_eq!(
        fixed(serving),
        before,
        "a denied retirement changed the WAL, the root or the release ledger: {failure:?}"
    );
    failure
}

fn fixed(
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

fn wal(serving: &ServingPhysicalRuntime) -> PhysicalWalObservation {
    serving
        .record_submission()
        .wal_observation()
        .expect("live WAL observation")
}

fn observe(serving: &ServingPhysicalRuntime) -> CertificationReleaseHeadObservation {
    serving
        .certification_release_head_observation()
        .expect("selected release custody")
}

fn retire(
    serving: &ServingPhysicalRuntime,
    proof: &AdmittedBlobReleaseProof,
) -> Result<BlobTerminalHeadRetirementReceipt, Failure> {
    serving
        .blobs()
        .unwrap()
        .retire_terminal_head(BlobTerminalHeadRetirementRequest::new(
            again(proof),
            placement(),
            deadline(),
            limits(WHOLE_GENERATION),
        ))
}

/// Completes enough checkpoints, keyed from `first_key`, that every
/// declaration made before the call is durably expired: its session can never
/// resume, so only the release-side dependencies of its head are left.
fn expire_declarations(serving: &ServingPhysicalRuntime, first_key: u8) {
    for round in 0..=HORIZON as u8 {
        checkpoint(serving, first_key + round);
    }
}

/// Publishes one generation and durably expires its declaration.
fn publish(serving: &ServingPhysicalRuntime, seed: u8) -> AdmittedBlobReleaseProof {
    let published = publish_resumable(serving, seed, HORIZON);
    expire_declarations(serving, seed);
    published.proof
}

/// One published generation: its admitted release, and what its ended
/// session and its object can still be asked.
struct Published {
    proof: AdmittedBlobReleaseProof,
    token: BlobResumeToken,
    object: BlobObjectId,
}

fn read_limits() -> BlobReadLimits {
    BlobReadLimits::new(NonZeroU64::new(128).unwrap())
}

/// Declares one two-chunk ingest of `object`, resumable for `horizon`
/// completed checkpoints.
fn begin(
    serving: &ServingPhysicalRuntime,
    object: BlobObjectId,
    horizon: u64,
) -> Result<BlobIngestSession<'_>, BlobIngestFailure> {
    let scope = admitted_blob_scope(SCOPE);
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(horizon).unwrap(),
        deadline(),
    )
    .unwrap();
    let blobs = serving.blobs().unwrap();
    blobs.begin_ingest(declaration, placement(), CHUNK as u64, read_limits())
}

/// Publishes one two-chunk generation and admits its release proof. Its
/// session stays resumable for `horizon` completed checkpoints.
fn publish_resumable(serving: &ServingPhysicalRuntime, seed: u8, horizon: u64) -> Published {
    let blobs = serving.blobs().unwrap();
    let object = blobs.issue_object_id(read_limits()).unwrap();
    let mut ingest = begin(serving, object, horizon).unwrap();
    let token = ingest.resume_token();
    ingest.push(&vec![seed; CHUNK]).unwrap();
    ingest.push(&vec![!seed; CHUNK]).unwrap();
    let published = ingest.finish().unwrap();
    let marker = serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("published generation has a selected marker");
    let record = marker.record();
    let proof = AdmittedBlobReleaseProof::certification_admit(
        serving.store_identity().bytes(),
        object.bytes(),
        published.generation().sequence(),
        record.allocation_epoch(),
        record.ordinal(),
        marker.encoded_digest(),
        [0x71; 32],
    )
    .unwrap();
    Published {
        proof,
        token,
        object,
    }
}

/// The fixture issuer admits the same release again for a repeated request.
fn again(proof: &AdmittedBlobReleaseProof) -> AdmittedBlobReleaseProof {
    AdmittedBlobReleaseProof::certification_admit(
        proof.store(),
        proof.object(),
        proof.generation(),
        proof.publication_allocation_epoch(),
        proof.publication_record_ordinal(),
        proof.publication_frame_sha256(),
        proof.issuer_evidence_sha256(),
    )
    .unwrap()
}

fn release_request(
    proof: &AdmittedBlobReleaseProof,
    batch_records: u16,
) -> BlobReclaimRequest<'static> {
    BlobReclaimRequest::released(again(proof), placement(), deadline(), limits(batch_records))
}

/// One drop batch of the release, with its displaced extents not yet settled.
fn release_batch(
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
    assert_eq!(receipt.disposition(), BlobReclaimDisposition::Dropped);
    receipt
}

/// Drops the whole generation and settles every displaced extent, so its
/// head is terminal and the drop's own checkpoint has attested it.
fn release_to_terminal(serving: &ServingPhysicalRuntime, proof: &AdmittedBlobReleaseProof) {
    let mut receipt = release_batch(serving, proof, WHOLE_GENERATION);
    assert_eq!(receipt.remaining_payload_records(), 0);
    settle(serving, &mut receipt);
}

/// Continues the receipt's displaced-extent retirement to completion.
fn settle(serving: &ServingPhysicalRuntime, receipt: &mut BlobReclaimReceipt) {
    for _ in 0..64 {
        if receipt.retirement() == BlobReclaimRetirement::Completed {
            return;
        }
        serving
            .blobs()
            .unwrap()
            .continue_reclaim_retirement(
                receipt,
                BlobReclaimRetirementBudget::new(NonZeroU32::new(64).unwrap(), deadline()),
            )
            .unwrap();
    }
    panic!("the displaced extents never settled: {receipt:?}");
}

fn checkpoint(serving: &ServingPhysicalRuntime, key: u8) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([key; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let handle = match serving.checkpoints().start(request).into_raw() {
        TransitionOutcome::Success(handle) => handle,
        TransitionOutcome::Failed(failure) => panic!("checkpoint {key:#x} denied: {failure:?}"),
        _ => panic!("checkpoint {key:#x} did not start"),
    };
    let outcome = handle.wait();
    assert!(
        matches!(outcome, PhysicalCheckpointOutcome::Completed(_)),
        "checkpoint {key:#x} must complete: {outcome:?}"
    );
}

fn limits(batch_records: u16) -> BlobReclaimLimits {
    BlobReclaimLimits::new(
        NonZeroU64::new(256).unwrap(),
        NonZeroU64::new(64 << 20).unwrap(),
        NonZeroU16::new(batch_records).unwrap(),
    )
    .unwrap()
}

fn deadline() -> PhysicalMutationDeadline {
    PhysicalMutationDeadline::after_milliseconds(30_000).unwrap()
}
