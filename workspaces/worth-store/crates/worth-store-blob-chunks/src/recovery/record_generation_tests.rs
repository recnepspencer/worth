use super::replay_source::{BlobReplaySourceIdentity, BlobReplaySourceIdentityKind};
use crate::lifecycle::generation_registry_test_support::{
    lifecycle_receipt_for_publication_with_identity, registry_authority,
    root_publication_with_bytes,
};
use crate::publication::test_support::{durable_wal_publication, replayable_wal_classification};
use crate::{
    BlobAdmittedRecoveryRecords, BlobAuthorityClassification, BlobCheckpointFrontierRecord,
    BlobChunkAppendRecord, BlobChunkReachabilityProofSet, BlobGenerationPublicationRecord,
    BlobGenerationRegistry, BlobGenerationRegistryAdmission, BlobManifestAgreement,
    BlobObjectClassificationAdmission, BlobPlacementManifestRow, BlobPublicationWalCommit,
    BlobPublicationWalPayload, BlobPublicationWalRecord, BlobReachabilityManifestRow,
    BlobReachabilityStaging, BlobRecoveryRecordDenialKind, BlobRecoveryRecordSet,
    BlobReplaySourceAdmission, BlobResumabilityReceipt, BlobResumeSessionCheckpointRecord,
    BlobRootCandidateForPublication, BlobRootCandidateRecord,
};
use worth_store_physical_backend::{
    BackendTargetProfile, BlobPhysicalManifestObservation, BlobPhysicalManifestValidation,
};

#[test]
fn stale_resume_checkpoint_for_same_object_generation_is_denied() {
    let object_case = "phase7-stale-resume-object";
    let stale = replay_fixture_for_object_generation(
        object_case,
        1,
        "phase7-stale-resume-old",
        b"oldgen000001",
    );
    let current = replay_fixture_for_object_generation(
        object_case,
        2,
        "phase7-stale-resume-current",
        b"curgen000002",
    );

    let denial = BlobRecoveryRecordSet::from_admitted_replay_records(
        BlobAdmittedRecoveryRecords::new()
            .with_chunk_append(current.chunk)
            .with_checkpoint_frontier(current.frontier)
            .with_root_candidate(current.root)
            .with_publication(current.publication)
            .with_resume_session(stale.resume)
            .with_manifest(current.manifest),
    )
    .expect_err("stale resume checkpoint must not join current publication");
    assert_eq!(
        denial.kind(),
        BlobRecoveryRecordDenialKind::PublicationWithoutManifestAgreement
    );
}

#[test]
fn unrelated_replay_chain_members_are_denied() {
    let published = replay_fixture_for_object_generation(
        "phase7-chain-published-object",
        1,
        "phase7-chain-published",
        b"chainpub0001",
    );
    let unrelated = replay_fixture_for_object_generation(
        "phase7-chain-unrelated-object",
        1,
        "phase7-chain-unrelated",
        b"chainunrel01",
    );

    assert_eq!(
        BlobRecoveryRecordSet::from_admitted_replay_records(
            BlobAdmittedRecoveryRecords::new()
                .with_chunk_append(unrelated.chunk.clone())
                .with_checkpoint_frontier(published.frontier.clone())
                .with_root_candidate(published.root.clone())
                .with_publication(published.publication.clone())
                .with_resume_session(published.resume.clone())
                .with_manifest(published.manifest.clone()),
        )
        .expect_err("checkpoint frontier must bind the supplied chunk append")
        .kind(),
        BlobRecoveryRecordDenialKind::IntegrityWithoutCheckpointFrontier
    );
    assert_eq!(
        BlobRecoveryRecordSet::from_admitted_replay_records(
            BlobAdmittedRecoveryRecords::new()
                .with_chunk_append(unrelated.chunk.clone())
                .with_checkpoint_frontier(unrelated.frontier.clone())
                .with_root_candidate(published.root.clone())
                .with_publication(published.publication.clone())
                .with_resume_session(published.resume.clone())
                .with_manifest(published.manifest.clone()),
        )
        .expect_err("root candidate must bind the supplied checkpoint frontier")
        .kind(),
        BlobRecoveryRecordDenialKind::CheckpointFrontierWithoutRootCandidate
    );
    assert_eq!(
        BlobRecoveryRecordSet::from_admitted_replay_records(
            BlobAdmittedRecoveryRecords::new()
                .with_chunk_append(unrelated.chunk)
                .with_checkpoint_frontier(unrelated.frontier)
                .with_root_candidate(unrelated.root)
                .with_publication(published.publication)
                .with_resume_session(published.resume)
                .with_manifest(published.manifest),
        )
        .expect_err("publication must bind the supplied root candidate")
        .kind(),
        BlobRecoveryRecordDenialKind::RootCandidateWithoutPublication
    );
}

struct GenerationReplayFixture {
    chunk: BlobChunkAppendRecord,
    frontier: BlobCheckpointFrontierRecord,
    root: BlobRootCandidateRecord,
    publication: BlobGenerationPublicationRecord,
    resume: BlobResumeSessionCheckpointRecord,
    manifest: BlobManifestAgreement,
}

fn replay_fixture_for_object_generation(
    object_case: &str,
    generation_sequence: u64,
    content_case: &str,
    bytes: &[u8],
) -> GenerationReplayFixture {
    let (candidate, reachability, resumability) =
        root_candidate_for_object_generation(object_case, generation_sequence, content_case, bytes);
    replay_fixture_from_candidate(content_case, candidate, reachability, resumability)
}

fn root_candidate_for_object_generation(
    object_case: &str,
    generation_sequence: u64,
    content_case: &str,
    bytes: &[u8],
) -> (
    BlobRootCandidateForPublication,
    BlobChunkReachabilityProofSet,
    BlobResumabilityReceipt,
) {
    let (root, stored_digest) = root_publication_with_bytes(content_case, bytes);
    let receipt = lifecycle_receipt_for_publication_with_identity(
        crate::lifecycle::generation_registry_test_support::PublicationIdentityCase::new(
            content_case,
            object_case,
        ),
        generation_sequence,
        root.chunk_tree_root().clone(),
        root.logical_content_digest().clone(),
        stored_digest,
        BlobAuthorityClassification::StoreOwnedPhysicalBlob,
        bytes,
    );
    let object_id = receipt.declaration().object_id().clone();
    let generation = receipt.declaration().generation();
    let mut registry = BlobGenerationRegistry::new();
    let classification = BlobObjectClassificationAdmission::from_executed_lifecycle(&receipt);
    BlobGenerationRegistryAdmission::from_executed_lifecycle(root.clone(), receipt, classification)
        .publish(&mut registry, registry_authority(content_case))
        .expect("registry publication should admit");
    let observation = registry
        .observe_registered_generation(&object_id, generation)
        .expect("registered generation should observe");
    let reachability = observation.lifecycle_receipt().reachability().clone();
    let resumability = observation.lifecycle_receipt().resumability_receipt();
    let candidate = BlobRootCandidateForPublication::from_registry_observation(observation, root)
        .expect("root candidate should bind registry observation");
    (candidate, reachability, resumability)
}

fn replay_fixture_from_candidate(
    case: &str,
    candidate: BlobRootCandidateForPublication,
    reachability: BlobChunkReachabilityProofSet,
    resumability: BlobResumabilityReceipt,
) -> GenerationReplayFixture {
    let logical = candidate.intent().logical_content_digest().clone();
    let staged = BlobReachabilityStaging::stage(candidate.clone(), reachability)
        .expect("stage reachability");
    let payload = BlobPublicationWalPayload::from_staged_reachability(&staged);
    let durable = durable_wal_publication(payload.frame_digest());
    let classification = replayable_wal_classification(payload.frame_digest());
    let wal_commit = BlobPublicationWalCommit::from_replayable_wal_record(
        staged,
        payload,
        durable,
        &classification,
    )
    .expect("publication wal commit should admit");
    let wal_record = BlobPublicationWalRecord::append(wal_commit);
    let wal_source = BlobReplaySourceAdmission::from_replayable_wal_report(&classification)
        .expect("wal replay source should admit");
    let checkpoint_source = checkpoint_source(case);
    let manifest_source = manifest_source(case);
    let manifest_digest = manifest_source.source_digest().to_owned();
    let chunk = BlobChunkAppendRecord::from_integrity_admission(logical, wal_source)
        .expect("integrity-admitted chunk should produce append record");
    let frontier =
        BlobCheckpointFrontierRecord::from_chunk_append(chunk.clone(), checkpoint_source.clone())
            .expect("checkpoint frontier should admit chunk append");
    let root = BlobRootCandidateRecord::from_checkpoint_frontier(frontier.clone(), candidate)
        .expect("root candidate should bind frontier");
    let publication = BlobGenerationPublicationRecord::from_replayed_wal_record(
        root.clone(),
        wal_record,
        BlobReplaySourceAdmission::from_replayable_wal_report(&classification)
            .expect("publication wal source should admit"),
    )
    .expect("replayed wal publication should admit");
    let resume = BlobResumeSessionCheckpointRecord::from_replayed_publication(
        &publication,
        resumability,
        checkpoint_source,
    )
    .expect("resume checkpoint should admit");
    let reachability_row =
        BlobReachabilityManifestRow::from_root_candidate(&root, manifest_source.clone())
            .expect("reachability row should admit");
    let placement_row =
        BlobPlacementManifestRow::from_replayed_publication(&publication, manifest_source)
            .expect("placement row should admit");
    let validation = BlobPhysicalManifestValidation::validate_observation(
        BlobPhysicalManifestObservation::for_certification_test_authority(
            manifest_digest.clone(),
            publication.published().generation().sequence(),
            manifest_digest,
            publication.published().generation().sequence(),
            publication.published().security_metadata().identity(),
            true,
        )
        .expect("backend manifest observation should admit"),
    )
    .expect("physical manifest should validate");
    let manifest = BlobManifestAgreement::validate(reachability_row, placement_row, validation)
        .expect("manifest agrees");
    GenerationReplayFixture {
        chunk,
        frontier,
        root,
        publication,
        resume,
        manifest,
    }
}

fn checkpoint_source(case: &str) -> BlobReplaySourceAdmission {
    let identity = BlobReplaySourceIdentity::new(
        BlobReplaySourceIdentityKind::Checkpoint,
        BackendTargetProfile::SimulatedStrictDurable,
        format!("{case}.checkpoint"),
        1,
        2,
    )
    .unwrap();
    BlobReplaySourceAdmission::from_checkpoint_replay_identity(&identity)
        .expect("checkpoint replay source should admit")
}

fn manifest_source(case: &str) -> BlobReplaySourceAdmission {
    let identity = BlobReplaySourceIdentity::new(
        BlobReplaySourceIdentityKind::Manifest,
        BackendTargetProfile::SimulatedStrictDurable,
        format!("{case}.manifest"),
        1,
        2,
    )
    .unwrap();
    BlobReplaySourceAdmission::from_manifest_replay_identity(&identity)
        .expect("manifest replay source should admit")
}
