//! Every dependency of a terminal head retired, live one at a time. Each
//! denial precedes any WAL, root or release-ledger effect, and the same
//! retirement succeeds once the dependency is gone.

use std::{
    num::NonZeroU64,
    thread,
    time::{Duration, Instant},
};

use worth_proof::{AdmittedBlobReleaseProof, TransitionOutcome};
use worth_store::physical_runtime::{
    production::PhysicalMutationCheckpoint, BlobAppendFailure, BlobReclaimDisposition,
    BlobReclaimRetirement, CertificationScopedAllocation, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationOutcome, PhysicalMutationPreparationSuccess, PhysicalMutationRequest,
    PhysicalOperationAllocationScope, PhysicalReadProtectionDenial, PhysicalRetirementDenial,
    PreparedPhysicalMutation, RecordAppendBatch, ServingPhysicalRuntime,
};

use super::{
    checkpoint, deadline, denied, publish, release_batch, release_request, release_to_terminal,
    retire, settle, Denial, Failure, WHOLE_GENERATION,
};
use crate::fixture::{placement, serving_from_initialization};

/// One Store whose only release head is terminal and checkpoint-attested.
fn attested_world() -> (
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

fn prepared_append(serving: &ServingPhysicalRuntime, seed: u8) -> PreparedPhysicalMutation {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([seed; 32]))
        .unwrap();
    let TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .prepare_durable_append(
                RecordAppendBatch::try_from_iter([b"terminal-head-contender".as_slice()]).unwrap(),
                placement(),
                PhysicalMutationRequest::platform_durable(key, deadline()),
            )
            .into_raw()
    else {
        panic!("an ordinary append must prepare beside a settled release");
    };
    prepared
}

#[test]
fn an_external_reader_of_the_source_root_denies_before_effects() {
    let (_directory, serving, proof) = attested_world();
    let held = serving.records().unwrap();
    assert!(matches!(
        denied(&serving, &proof),
        Failure::Denied(Denial::ProtectedReader)
    ));
    drop(held);
    retire(&serving, &proof).expect("the reader is gone");
    serving.close();
}

#[test]
fn a_release_proof_of_another_store_is_refused_before_effects() {
    let (_directory, serving, proof) = attested_world();
    let foreign = AdmittedBlobReleaseProof::certification_admit(
        [0x5a; 16],
        proof.object(),
        proof.generation(),
        proof.publication_allocation_epoch(),
        proof.publication_record_ordinal(),
        proof.publication_frame_sha256(),
        proof.issuer_evidence_sha256(),
    )
    .unwrap();
    assert!(matches!(denied(&serving, &foreign), Failure::ForeignStore));
    retire(&serving, &proof).expect("the Store's own proof still retires the head");
    serving.close();
}

#[test]
fn a_retirement_in_flight_fences_readers_until_it_commits() {
    let (_directory, serving, proof) = attested_world();
    let gate = serving.pause_physical_mutation_at(PhysicalMutationCheckpoint::BeforeWalAppend);
    thread::scope(|workers| {
        let retiring = workers.spawn(|| retire(&serving, &proof));
        let until = Instant::now() + Duration::from_secs(60);
        while !gate.await_arrival() {
            assert!(
                Instant::now() < until,
                "the retirement never reached its WAL effect"
            );
        }
        let fenced = matches!(
            serving.records(),
            Err(PhysicalReadProtectionDenial::ReclaimFenced)
        );
        gate.release();
        assert!(fenced, "a reader entered beside the retirement's effect");
        retiring
            .join()
            .unwrap()
            .expect("the fenced retirement commits");
    });
    assert!(serving.records().is_ok(), "the commit releases the fence");
    serving.close();
}

#[test]
fn a_pending_publication_denies_before_effects() {
    let (_directory, serving, proof) = attested_world();
    let prepared = prepared_append(&serving, 0xa1);
    let gate = serving.pause_physical_mutation_at(PhysicalMutationCheckpoint::BeforeWalAppend);
    thread::scope(|workers| {
        let appender = workers.spawn(|| prepared.execute());
        let until = Instant::now() + Duration::from_secs(60);
        while !gate.await_arrival() {
            assert!(
                Instant::now() < until,
                "the append never registered its pending publication"
            );
        }
        let failure = denied(&serving, &proof);
        gate.release();
        assert!(
            matches!(failure, Failure::Denied(Denial::PendingPublication)),
            "{failure:?}"
        );
        assert!(matches!(
            appender.join().unwrap(),
            PhysicalMutationOutcome::Completed(_)
        ));
    });
    retire(&serving, &proof).expect("the publication settled");
    serving.close();
}

#[test]
fn a_live_reclaim_claim_denies_before_effects() {
    let (_directory, serving, proof) = attested_world();
    let other = publish(&serving, 0x47);
    let live = serving
        .blobs()
        .unwrap()
        .reclaim(release_request(&other, WHOLE_GENERATION))
        .unwrap();
    let failure = denied(&serving, &proof);
    assert!(
        matches!(
            failure,
            Failure::ReadProtection(PhysicalReadProtectionDenial::ReclaimFenced)
        ),
        "{failure:?}"
    );
    assert_eq!(live.cancel(), BlobReclaimDisposition::ProvenNoEffect);
    retire(&serving, &proof).expect("the competing reclaim was cancelled");
    serving.close();
}

#[test]
fn an_unresolved_idempotency_binding_denies_before_effects_and_releases_the_fence() {
    let (_directory, serving, proof) = attested_world();
    let prepared = prepared_append(&serving, 0xa2);
    let failure = denied(&serving, &proof);
    assert!(
        matches!(
            failure,
            Failure::Denied(Denial::UnresolvedIdempotencyBinding)
        ),
        "{failure:?}"
    );
    assert!(
        matches!(prepared.execute(), PhysicalMutationOutcome::Completed(_)),
        "the denied retirement must not leave its fence behind"
    );
    retire(&serving, &proof).expect("the settled binding no longer blocks");
    serving.close();
}

/// In one process the head's own drop already funded the release envelope a
/// retirement needs, and that grant never shrinks. Pool pressure therefore
/// denies at the entry's working allocation or at the publication's
/// preparation, both before the WAL effect.
#[test]
fn pool_pressure_denies_the_retirement_before_the_wal_effect() {
    let (_directory, serving, proof) = attested_world();
    let held = hold_recovery_bytes_leaving(&serving, 1);
    let Failure::Allocation(entry) = denied(&serving, &proof) else {
        panic!("one spare byte cannot fund the entry's own working allocation");
    };
    let entry_bytes = entry
        .pressure()
        .expect("the entry was denied by pool pressure")
        .requested();
    drop(held);
    // Exactly the entry's working allocation is left: the retained release
    // envelope still funds the head, and the publication cannot prepare.
    let held = hold_recovery_bytes_leaving(&serving, entry_bytes);
    let failure = denied(&serving, &proof);
    assert!(
        matches!(
            failure,
            Failure::Publication(BlobAppendFailure::Preparation(_))
        ),
        "{failure:?}"
    );
    drop(held);
    retire(&serving, &proof).expect("the pool is free again");
    serving.close();
}

/// Holds every Recovery byte the pool would still grant except `spare`. The
/// bound is found against the real pool.
fn hold_recovery_bytes_leaving(
    serving: &ServingPhysicalRuntime,
    spare: u64,
) -> CertificationScopedAllocation {
    let residency = serving.certification_physical_residency();
    let admit = |bytes: u64| {
        residency.admit_operation_scope(
            PhysicalOperationAllocationScope::Recovery,
            NonZeroU64::new(bytes).unwrap(),
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

/// The identity scan answers before the head lookup: the publication of a
/// generation that was never released is still selected.
#[test]
fn a_generation_that_was_never_released_denies_for_its_selected_publication() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let proof = publish(&serving, 0x31);
    let failure = denied(&serving, &proof);
    assert!(
        matches!(
            failure,
            Failure::Denied(Denial::IdentityPublicationSelected)
        ),
        "{failure:?}"
    );
    serving.close();
}

#[test]
fn a_partially_released_generation_keeps_its_nonterminal_head() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let proof = publish(&serving, 0x31);
    let mut receipt = release_batch(&serving, &proof, 1);
    assert!(receipt.remaining_payload_records() > 0);
    settle(&serving, &mut receipt);
    assert!(matches!(
        denied(&serving, &proof),
        Failure::Denied(Denial::NonterminalHead)
    ));
    serving.close();
}

#[test]
fn a_terminal_head_no_checkpoint_has_attested_denies_until_that_checkpoint() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let proof = publish(&serving, 0x31);
    serving.certification_fail_next_checkpoint_admission();
    let receipt = release_batch(&serving, &proof, WHOLE_GENERATION);
    assert_eq!(receipt.remaining_payload_records(), 0);
    assert_eq!(
        receipt.retirement(),
        BlobReclaimRetirement::Pending(PhysicalRetirementDenial::Checkpoint)
    );
    assert!(receipt.displaced_extents().len() > 1);
    let failure = denied(&serving, &proof);
    assert!(
        matches!(failure, Failure::Denied(Denial::NotCheckpointAttested)),
        "{failure:?}"
    );
    checkpoint(&serving, 0x91);
    let failure = denied(&serving, &proof);
    assert!(
        matches!(failure, Failure::Denied(Denial::PendingPublication)),
        "the first displaced extent's release is a prepared publication: {failure:?}"
    );
    serving
        .retire_displaced_segment()
        .expect("the checkpoint admits the first displaced extent's release");
    let failure = denied(&serving, &proof);
    assert!(
        matches!(failure, Failure::Denied(Denial::DisplacedExtentOutstanding)),
        "the attested head still waits for its displaced extents: {failure:?}"
    );
    for _ in 0..64 {
        match serving.retire_displaced_segment() {
            Ok(()) => {}
            Err(PhysicalRetirementDenial::Absent) => break,
            Err(denial) => panic!("a displaced extent must retire: {denial:?}"),
        }
    }
    retire(&serving, &proof).expect("attested and settled");
    serving.close();
}
