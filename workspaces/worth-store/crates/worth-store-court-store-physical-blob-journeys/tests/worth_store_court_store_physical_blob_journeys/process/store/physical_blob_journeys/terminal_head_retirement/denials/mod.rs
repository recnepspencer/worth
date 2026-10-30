//! Every dependency of a terminal head retired, live one at a time. Each
//! denial precedes any WAL, root or release-ledger effect, and the same
//! retirement succeeds once the dependency is gone.

#[allow(unused_imports)]
use crate::physical_blob_journeys::terminal_head_retirement::denials::*;
use {std::num::NonZeroU64, std::thread, std::time::Duration, std::time::Instant};

use {worth_proof::AdmittedBlobReleaseProof, worth_proof::TransitionOutcome};
use {
    worth_store::physical_runtime::production::PhysicalMutationCheckpoint,
    worth_store::physical_runtime::BlobAppendFailure,
    worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimRetirement,
    worth_store::physical_runtime::CertificationScopedAllocation,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::PhysicalOperationAllocationScope,
    worth_store::physical_runtime::PhysicalReadProtectionDenial,
    worth_store::physical_runtime::PhysicalRetirementDenial,
    worth_store::physical_runtime::PreparedPhysicalMutation,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

use {
    crate::physical_blob_journeys::terminal_head_retirement::checkpoint,
    crate::physical_blob_journeys::terminal_head_retirement::deadline,
    crate::physical_blob_journeys::terminal_head_retirement::denied,
    crate::physical_blob_journeys::terminal_head_retirement::publish,
    crate::physical_blob_journeys::terminal_head_retirement::release_batch,
    crate::physical_blob_journeys::terminal_head_retirement::release_request,
    crate::physical_blob_journeys::terminal_head_retirement::release_to_terminal,
    crate::physical_blob_journeys::terminal_head_retirement::retire,
    crate::physical_blob_journeys::terminal_head_retirement::settle,
    crate::physical_blob_journeys::terminal_head_retirement::WHOLE_GENERATION,
    worth_store::physical_runtime::BlobTerminalHeadRetirementDenial as Denial,
    worth_store::physical_runtime::BlobTerminalHeadRetirementFailure as Failure,
};
use {
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
};

#[test]
fn an_external_reader_of_the_source_root_denies_before_effects() {
    let (_directory, serving, proof) = attested_world();
    let held = serving.records().unwrap();
    assert!(matches!(
        denied(&serving, &proof),
        worth_store::physical_runtime::BlobTerminalHeadRetirementFailure::Denied(
            worth_store::physical_runtime::BlobTerminalHeadRetirementDenial::ProtectedReader
        )
    ));
    drop(held);
    retire(&serving, &proof).expect("the reader is gone");
    serving.close();
}

#[test]
fn a_release_proof_of_another_store_is_refused_before_effects() {
    let (_directory, serving, proof) = attested_world();
    let foreign = worth_proof::AdmittedBlobReleaseProof::certification_admit(
        [0x5a; 16],
        proof.object(),
        proof.generation(),
        proof.publication_allocation_epoch(),
        proof.publication_record_ordinal(),
        proof.publication_frame_sha256(),
        proof.issuer_evidence_sha256(),
    )
    .unwrap();
    assert!(matches!(
        denied(&serving, &foreign),
        worth_store::physical_runtime::BlobTerminalHeadRetirementFailure::ForeignStore
    ));
    retire(&serving, &proof).expect("the Store's own proof still retires the head");
    serving.close();
}

#[test]
fn a_retirement_in_flight_fences_readers_until_it_commits() {
    let (_directory, serving, proof) = attested_world();
    let gate = serving.pause_physical_mutation_at(
        worth_store::physical_runtime::production::PhysicalMutationCheckpoint::BeforeWalAppend,
    );
    std::thread::scope(|workers| {
        let retiring = workers.spawn(|| retire(&serving, &proof));
        let until = std::time::Instant::now() + std::time::Duration::from_secs(60);
        while !gate.await_arrival() {
            assert!(
                std::time::Instant::now() < until,
                "the retirement never reached its WAL effect"
            );
        }
        let fenced = matches!(
            serving.records(),
            Err(worth_store::physical_runtime::PhysicalReadProtectionDenial::ReclaimFenced)
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
    let gate = serving.pause_physical_mutation_at(
        worth_store::physical_runtime::production::PhysicalMutationCheckpoint::BeforeWalAppend,
    );
    std::thread::scope(|workers| {
        let appender = workers.spawn(|| prepared.execute());
        let until = std::time::Instant::now() + std::time::Duration::from_secs(60);
        while !gate.await_arrival() {
            assert!(
                std::time::Instant::now() < until,
                "the append never registered its pending publication"
            );
        }
        let failure = denied(&serving, &proof);
        gate.release();
        assert!(
            matches!(failure, worth_store::physical_runtime::BlobTerminalHeadRetirementFailure::Denied(worth_store::physical_runtime::BlobTerminalHeadRetirementDenial::PendingPublication)),
            "{failure:?}"
        );
        assert!(matches!(
            appender.join().unwrap(),
            worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_)
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
            worth_store::physical_runtime::BlobTerminalHeadRetirementFailure::ReadProtection(
                worth_store::physical_runtime::PhysicalReadProtectionDenial::ReclaimFenced
            )
        ),
        "{failure:?}"
    );
    assert_eq!(
        live.cancel(),
        worth_store::physical_runtime::BlobReclaimDisposition::ProvenNoEffect
    );
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
            worth_store::physical_runtime::BlobTerminalHeadRetirementFailure::Denied(worth_store::physical_runtime::BlobTerminalHeadRetirementDenial::UnresolvedIdempotencyBinding)
        ),
        "{failure:?}"
    );
    assert!(
        matches!(
            prepared.execute(),
            worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_)
        ),
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
    let worth_store::physical_runtime::BlobTerminalHeadRetirementFailure::Allocation(entry) =
        denied(&serving, &proof)
    else {
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
            worth_store::physical_runtime::BlobTerminalHeadRetirementFailure::Publication(
                worth_store::physical_runtime::BlobAppendFailure::Preparation(_)
            )
        ),
        "{failure:?}"
    );
    drop(held);
    retire(&serving, &proof).expect("the pool is free again");
    serving.close();
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
            worth_store::physical_runtime::BlobTerminalHeadRetirementFailure::Denied(worth_store::physical_runtime::BlobTerminalHeadRetirementDenial::IdentityPublicationSelected)
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
        worth_store::physical_runtime::BlobTerminalHeadRetirementFailure::Denied(
            worth_store::physical_runtime::BlobTerminalHeadRetirementDenial::NonterminalHead
        )
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
        worth_store::physical_runtime::BlobReclaimRetirement::Pending(
            worth_store::physical_runtime::PhysicalRetirementDenial::Checkpoint
        )
    );
    assert!(receipt.displaced_extents().len() > 1);
    let failure = denied(&serving, &proof);
    assert!(
        matches!(failure, worth_store::physical_runtime::BlobTerminalHeadRetirementFailure::Denied(worth_store::physical_runtime::BlobTerminalHeadRetirementDenial::NotCheckpointAttested)),
        "{failure:?}"
    );
    checkpoint(&serving, 0x91);
    let failure = denied(&serving, &proof);
    assert!(
        matches!(
            failure,
            worth_store::physical_runtime::BlobTerminalHeadRetirementFailure::Denied(
                worth_store::physical_runtime::BlobTerminalHeadRetirementDenial::PendingPublication
            )
        ),
        "the first displaced extent's release is a prepared publication: {failure:?}"
    );
    serving
        .retire_displaced_segment()
        .expect("the checkpoint admits the first displaced extent's release");
    let failure = denied(&serving, &proof);
    assert!(
        matches!(failure, worth_store::physical_runtime::BlobTerminalHeadRetirementFailure::Denied(worth_store::physical_runtime::BlobTerminalHeadRetirementDenial::DisplacedExtentOutstanding)),
        "the attested head still waits for its displaced extents: {failure:?}"
    );
    for _ in 0..64 {
        match serving.retire_displaced_segment() {
            Ok(()) => {}
            Err(worth_store::physical_runtime::PhysicalRetirementDenial::Absent) => break,
            Err(denial) => panic!("a displaced extent must retire: {denial:?}"),
        }
    }
    retire(&serving, &proof).expect("attested and settled");
    serving.close();
}
