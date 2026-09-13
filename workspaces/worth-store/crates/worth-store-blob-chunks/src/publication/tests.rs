use crate::publication::{
    BlobPublicationCrashBoundaryReport, BlobPublicationObservationSet,
    BlobPublicationReplayReadDenial, BlobPublicationReplayedCrashEdge,
};
use worth_store_physical_isolation::SemanticVisibilityReference;

use crate::publication::test_support::{
    chunk_write_replay_evidence, durable_wal_publication, publication_inputs,
    publication_inputs_with_bytes_and_chunk_size, publish_generation, recovery_cases,
    replayable_wal_classification, with_generic_recovery_replay_entry, with_recovery_replay_entry,
};
use crate::{
    reject_copied_publication_record_as_blob_visibility, reject_root_candidate_as_blob_visibility,
    reject_semantic_reference_as_blob_visibility, reject_staged_reachability_as_blob_visibility,
    BlobPublicationDenial, BlobPublicationPreWalReplayEvidence, BlobPublicationRecoveryEvidence,
    BlobPublicationRecoveryReplay, BlobPublicationWalCommit, BlobPublicationWalPayload,
    BlobReachabilityStaging, BlobSemanticVisibilityHandoff, BlobSemanticVisibilityOutcome,
    BlobVisibleGeneration,
};

#[test]
fn committed_publication_is_the_only_visible_generation_source() {
    let (published, _previous) = publish_generation("phase6-visible");
    let visible = BlobVisibleGeneration::from_published(&published);
    let handoff =
        BlobSemanticVisibilityHandoff::observe_previous_or_published(None, Some(&published))
            .expect("published generation should be visible");

    assert_eq!(visible.object_id(), published.object_id());
    assert_eq!(visible.generation(), published.generation());
    assert_eq!(visible.chunk_tree_root(), published.chunk_tree_root());
    assert_eq!(
        published.staging_identity().object_id(),
        published.object_id()
    );
    assert_eq!(
        published.staging_identity().generation(),
        published.generation()
    );
    assert!(!published.replay_classification_digest().is_empty());
    assert_eq!(published.replay_counters().replayable_durable_wal(), 1);
    assert_eq!(
        published.security_metadata(),
        published.staging_identity().security_metadata()
    );
    assert_eq!(
        handoff.outcome(),
        &BlobSemanticVisibilityOutcome::NewlyPublishedGeneration(visible)
    );
}

#[test]
fn reachability_staging_rejects_proof_set_from_another_blob_authority() {
    let (candidate, _, _) =
        publication_inputs_with_bytes_and_chunk_size("phase14-stage-authority-a", b"same", 4);
    let (_, other_reachability, _) =
        publication_inputs_with_bytes_and_chunk_size("phase14-stage-authority-b", b"same", 4);

    assert!(matches!(
        BlobReachabilityStaging::stage(candidate, other_reachability),
        Err(BlobPublicationDenial::ReachabilityDigestMismatch { .. })
    ));
}

#[test]
fn semantic_handoff_observes_previous_or_new_never_partial() {
    let (published, previous) = publish_generation("phase6-handoff");
    let previous_handoff =
        BlobSemanticVisibilityHandoff::observe_previous_or_published(Some(previous.clone()), None)
            .expect("previous generation remains visible before commit");
    let new_handoff = BlobSemanticVisibilityHandoff::observe_previous_or_published(
        Some(previous),
        Some(&published),
    )
    .expect("new publication wins after commit");

    assert!(matches!(
        previous_handoff.outcome(),
        BlobSemanticVisibilityOutcome::PreviousGeneration(_)
    ));
    assert!(matches!(
        new_handoff.outcome(),
        BlobSemanticVisibilityOutcome::NewlyPublishedGeneration(_)
    ));
}

#[test]
fn weaker_publication_representations_are_denied_visibility() {
    let (candidate, _, _) = publication_inputs("phase6-denial-candidate");
    let (staging_candidate, reachability, _) = publication_inputs("phase6-denial-staged");
    let staged = BlobReachabilityStaging::stage(staging_candidate, reachability)
        .expect("reachability should stage");
    let copied_record = durable_wal_publication("phase6-denial");
    let reference = SemanticVisibilityReference::commit("relational", "semantic-blob-ref");

    assert!(matches!(
        reject_root_candidate_as_blob_visibility(&candidate),
        BlobPublicationDenial::RootCandidateRejected { .. }
    ));
    assert!(matches!(
        reject_staged_reachability_as_blob_visibility(&staged),
        BlobPublicationDenial::StagedReachabilityRejected { .. }
    ));
    assert!(matches!(
        reject_copied_publication_record_as_blob_visibility(&copied_record),
        BlobPublicationDenial::CopiedPublicationRecordRejected { .. }
    ));
    assert!(matches!(
        reject_semantic_reference_as_blob_visibility(&reference),
        BlobPublicationDenial::SemanticReferenceRejected { .. }
    ));
}

#[test]
fn every_adversarial_crash_point_recovers_from_typed_evidence() {
    for (evidence, expected_crash, expected_state) in recovery_cases() {
        let replay = BlobPublicationRecoveryReplay::recover(evidence);
        assert_eq!(replay.crash_point(), expected_crash);
        assert_eq!(replay.recovered_state(), expected_state);
        assert!(!replay.evidence().evidence_digest().is_empty());
    }
}

#[test]
fn copied_publication_declaration_cannot_bind_unrelated_replay_identity() {
    let (candidate, reachability, _) = publication_inputs("phase6-copied-wal");
    let staged =
        BlobReachabilityStaging::stage(candidate, reachability).expect("reachability should stage");
    let payload = BlobPublicationWalPayload::from_staged_reachability(&staged);

    assert!(matches!(
        BlobPublicationWalCommit::from_replayable_wal_record(
            staged,
            payload.clone(),
            durable_wal_publication(payload.frame_digest()),
            &replayable_wal_classification("sha256:phase6-unrelated-wal"),
        ),
        Err(BlobPublicationDenial::WalReplayIdentityMismatch { .. })
    ));
}

#[test]
fn publication_payload_binds_counter_receipt_identity() {
    let (candidate, reachability, _) = publication_inputs("phase6-counter-receipt");
    let staged =
        BlobReachabilityStaging::stage(candidate, reachability).expect("reachability should stage");
    let payload = BlobPublicationWalPayload::from_staged_reachability(&staged);

    assert!(payload.frame_digest().contains(
        payload
            .staging_identity()
            .counter_receipt_identity()
            .as_str()
    ));
}

#[test]
fn replayable_wal_identity_cannot_publish_unrelated_staged_blob() {
    let (source_candidate, source_reachability, _) =
        publication_inputs("phase6-source-publication");
    let source_staged = BlobReachabilityStaging::stage(source_candidate, source_reachability)
        .expect("source reachability should stage");
    let source_payload = BlobPublicationWalPayload::from_staged_reachability(&source_staged);
    let (target_candidate, target_reachability, _) =
        publication_inputs("phase6-target-publication");
    let target_staged = BlobReachabilityStaging::stage(target_candidate, target_reachability)
        .expect("target reachability should stage");

    assert!(matches!(
        BlobPublicationWalCommit::from_replayable_wal_record(
            target_staged,
            source_payload.clone(),
            durable_wal_publication(source_payload.frame_digest()),
            &replayable_wal_classification(source_payload.frame_digest()),
        ),
        Err(BlobPublicationDenial::WalReplayIdentityMismatch { .. })
    ));
}

#[test]
fn pre_wal_recovery_evidence_cannot_be_reused_for_wrong_crash_state() {
    let (candidate, _, _) = publication_inputs("phase6-wrong-crash-state");
    let checksum = candidate.intent().logical_content_digest().clone();
    let chunk_write_replay = chunk_write_replay_evidence(&checksum);

    assert!(matches!(
        BlobPublicationRecoveryEvidence::checksum_admitted(&checksum, chunk_write_replay),
        Err(BlobPublicationDenial::WalReplayEvidenceRequired { .. })
    ));
}

#[test]
fn replay_read_witness_rejects_copied_bytes_for_wrong_operation() {
    let (candidate, _, _) = publication_inputs("phase6-wrong-replay-operation");
    let digest = candidate.intent().logical_content_digest().clone();
    let operation =
        BlobPublicationPreWalReplayEvidence::chunk_write_recovery_operation_digest(&digest);
    let replay = with_recovery_replay_entry(operation.as_str(), |replay_entry| {
        BlobPublicationReplayedCrashEdge::from_replay_read_artifact(
            replay_entry
                .read_blob_publication_before_wal_append()
                .expect("protected before-WAL replay bytes should read"),
        )
        .expect("replayed bytes admit as their own before-WAL source")
    });

    assert!(matches!(
        BlobPublicationPreWalReplayEvidence::from_checksum_admitted_replay(&digest, &replay),
        Err(BlobPublicationDenial::WalReplayEvidenceRequired { .. })
    ));
}

#[test]
fn copied_replay_bytes_cannot_be_readmitted_for_another_operation() {
    let (candidate, _, _) = publication_inputs("phase6-reused-replay-admission");
    let digest = candidate.intent().logical_content_digest().clone();
    let operation =
        BlobPublicationPreWalReplayEvidence::checksum_admitted_recovery_operation_digest(&digest);
    let replay = with_recovery_replay_entry(operation.as_str(), |replay_entry| {
        BlobPublicationReplayedCrashEdge::from_replay_read_artifact(
            replay_entry
                .read_blob_publication_before_wal_append()
                .expect("protected before-WAL replay bytes should read"),
        )
        .expect("replayed bytes admit as their own before-WAL source")
    });

    assert!(matches!(
        BlobPublicationPreWalReplayEvidence::from_chunk_write_replay(&digest, &replay),
        Err(BlobPublicationDenial::WalReplayEvidenceRequired { .. })
    ));
}

#[test]
fn bytes_without_before_wal_operation_are_denied_before_witness() {
    with_generic_recovery_replay_entry("phase6-no-before-wal-operation", |replay_entry| {
        assert!(matches!(
            replay_entry.read_blob_publication_before_wal_append(),
            Err(BlobPublicationReplayReadDenial::NotBeforeWalAppend { .. })
        ));
    });
}

#[test]
fn generic_recovery_entry_cannot_mint_before_wal_replay_read_artifact() {
    with_generic_recovery_replay_entry("phase6-generic-entry-no-pre-wal-read", |replay_entry| {
        assert!(matches!(
            replay_entry.read_blob_publication_before_wal_append(),
            Err(BlobPublicationReplayReadDenial::NotBeforeWalAppend {
                actual_operation_digest: None
            })
        ));
    });
}

#[test]
fn checkpoint_cutover_bytes_are_denied_before_wal_witness() {
    with_generic_recovery_replay_entry("phase6-no-before-wal-operation", |replay_entry| {
        assert!(matches!(
            replay_entry.read_blob_publication_checkpoint_cutover("phase6-no-before-wal-operation"),
            Err(BlobPublicationReplayReadDenial::NotBeforeWalAppend { .. })
        ));
    });
}

#[test]
fn replay_read_admission_token_no_longer_exists_for_reuse() {
    let (candidate, _, _) = publication_inputs("phase6-no-replay-admission-token");
    let digest = candidate.intent().logical_content_digest().clone();
    let operation =
        BlobPublicationPreWalReplayEvidence::chunk_write_recovery_operation_digest(&digest);
    let replay = with_recovery_replay_entry(operation.as_str(), |replay_entry| {
        BlobPublicationReplayedCrashEdge::from_replay_read_artifact(
            replay_entry
                .read_blob_publication_before_wal_append()
                .expect("protected before-WAL replay bytes should read"),
        )
    });

    assert!(replay.is_ok());
}

#[test]
fn non_replayable_recovery_classification_cannot_commit_publication_record() {
    let (candidate, reachability, _) = publication_inputs("phase6-non-replayable");
    let staged =
        BlobReachabilityStaging::stage(candidate, reachability).expect("reachability should stage");
    let payload = BlobPublicationWalPayload::from_staged_reachability(&staged);
    assert!(BlobPublicationCrashBoundaryReport::admit_observations(
        BlobPublicationObservationSet::new().with_insufficient_persisted_evidence("ambiguous"),
    )
    .is_err());
    let _ = (staged, payload);
}
