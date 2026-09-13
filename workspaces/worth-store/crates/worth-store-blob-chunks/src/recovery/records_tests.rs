use super::replay_source::{BlobReplaySourceIdentity, BlobReplaySourceIdentityKind};
use crate::publication::test_support::{
    durable_wal_publication, publication_inputs, replayable_wal_classification,
};
use crate::BlobAdmittedRecoveryRecords;
use crate::{
    BlobCheckpointFrontierRecord, BlobChunkAppendRecord, BlobGenerationPublicationRecord,
    BlobManifestAgreement, BlobPlacementManifestRow, BlobPublicationWalCommit,
    BlobPublicationWalPayload, BlobPublicationWalRecord, BlobReachabilityManifestRow,
    BlobReachabilityStaging, BlobRecoveryOutcome, BlobRecoveryRecordDenialKind,
    BlobRecoveryRecordSet, BlobRecoveryReplay, BlobReplaySourceAdmission,
    BlobResumeSessionCheckpointRecord, BlobRootCandidateRecord,
};
use worth_store_physical_backend::{
    BackendTargetProfile, BlobPhysicalManifestObservation, BlobPhysicalManifestValidation,
};

#[test]
fn admitted_wal_checkpoint_manifest_replay_reconstructs_blob_facts() {
    let fixture = replay_fixture("phase7-replay");
    let record_counters = fixture.records.counters();
    assert_eq!(
        (
            record_counters.wal_records(),
            record_counters.checkpoint_records(),
            record_counters.manifest_rows(),
            record_counters.replayed_outcomes(),
        ),
        (3, 2, 2, 0)
    );

    let replay = BlobRecoveryReplay::reconstruct(fixture.records);

    assert_eq!(
        replay.outcome(),
        BlobRecoveryOutcome::ClosedResumeSessionPublishedGeneration
    );
    assert_eq!(
        replay.published_generation().object_id(),
        &fixture.object_id
    );
    assert_eq!(replay.resume_session().object_id(), &fixture.object_id);
    assert_eq!(
        replay.reachability_staging().object_id(),
        &fixture.object_id
    );
    assert_eq!(
        replay.placement_observation().object_id(),
        &fixture.object_id
    );
    let replay_counters = replay.counters();
    assert_eq!(
        (
            replay_counters.wal_records(),
            replay_counters.checkpoint_records(),
            replay_counters.manifest_rows(),
            replay_counters.replayed_outcomes(),
        ),
        (3, 2, 2, 1)
    );
}

#[test]
fn missing_prerequisites_produce_distinct_typed_denials() {
    let fixture = replay_fixture("phase7-denial");

    assert_eq!(
        BlobRecoveryRecordSet::from_admitted_replay_records(BlobAdmittedRecoveryRecords::new())
            .expect_err("chunk bytes alone must deny before chunk append admission")
            .kind(),
        BlobRecoveryRecordDenialKind::ChunkBytesWithoutIntegrityAdmission
    );
    assert_eq!(
        BlobRecoveryRecordSet::from_admitted_replay_records(
            BlobAdmittedRecoveryRecords::new().with_chunk_append(fixture.chunk.clone()),
        )
        .expect_err("integrity without frontier must deny")
        .kind(),
        BlobRecoveryRecordDenialKind::IntegrityWithoutCheckpointFrontier
    );
    assert_eq!(
        BlobRecoveryRecordSet::from_admitted_replay_records(
            BlobAdmittedRecoveryRecords::new()
                .with_chunk_append(fixture.chunk.clone())
                .with_checkpoint_frontier(fixture.frontier.clone()),
        )
        .expect_err("frontier without root candidate must deny")
        .kind(),
        BlobRecoveryRecordDenialKind::CheckpointFrontierWithoutRootCandidate
    );
    assert_eq!(
        BlobRecoveryRecordSet::from_admitted_replay_records(
            BlobAdmittedRecoveryRecords::new()
                .with_chunk_append(fixture.chunk.clone())
                .with_checkpoint_frontier(fixture.frontier.clone())
                .with_root_candidate(fixture.root.clone()),
        )
        .expect_err("root candidate without publication must deny")
        .kind(),
        BlobRecoveryRecordDenialKind::RootCandidateWithoutPublication
    );
    assert_eq!(
        BlobRecoveryRecordSet::from_admitted_replay_records(
            BlobAdmittedRecoveryRecords::new()
                .with_chunk_append(fixture.chunk.clone())
                .with_checkpoint_frontier(fixture.frontier.clone())
                .with_root_candidate(fixture.root.clone())
                .with_publication(fixture.publication.clone()),
        )
        .expect_err("publication without closed resume session must deny")
        .kind(),
        BlobRecoveryRecordDenialKind::PublicationWithoutClosedResumeSession
    );
    assert_eq!(
        BlobRecoveryRecordSet::from_admitted_replay_records(
            BlobAdmittedRecoveryRecords::new()
                .with_chunk_append(fixture.chunk.clone())
                .with_checkpoint_frontier(fixture.frontier.clone())
                .with_root_candidate(fixture.root.clone())
                .with_publication(fixture.publication.clone())
                .with_resume_session(fixture.resume.clone()),
        )
        .expect_err("publication without manifest agreement must deny")
        .kind(),
        BlobRecoveryRecordDenialKind::PublicationWithoutManifestAgreement
    );
}

#[test]
fn manifest_agreement_denies_wrong_physical_validation_digest() {
    let fixture = replay_fixture("phase7-wrong-manifest-validation");
    let manifest_source = manifest_source("phase7-wrong-manifest-validation");
    let reachability_row =
        BlobReachabilityManifestRow::from_root_candidate(&fixture.root, manifest_source.clone())
            .expect("reachability row should admit");
    let placement_row =
        BlobPlacementManifestRow::from_replayed_publication(&fixture.publication, manifest_source)
            .expect("placement row should admit");
    let generation = fixture.publication.published().generation().sequence();
    let wrong_validation = BlobPhysicalManifestValidation::validate_observation(
        BlobPhysicalManifestObservation::for_certification_test_authority(
            "phase7-other-manifest",
            generation,
            "phase7-other-manifest",
            generation,
            fixture
                .publication
                .published()
                .security_metadata()
                .identity(),
            true,
        )
        .expect("wrong manifest observation should still be physically valid"),
    )
    .expect("physical validation is valid for the wrong manifest");

    let denial = BlobManifestAgreement::validate(reachability_row, placement_row, wrong_validation)
        .expect_err("physical validation from another manifest must not agree");
    assert_eq!(
        denial.kind(),
        BlobRecoveryRecordDenialKind::PublicationWithoutManifestAgreement
    );
}

struct ReplayFixture {
    records: BlobRecoveryRecordSet,
    object_id: crate::BlobObjectId,
    chunk: BlobChunkAppendRecord,
    frontier: BlobCheckpointFrontierRecord,
    root: BlobRootCandidateRecord,
    publication: BlobGenerationPublicationRecord,
    resume: BlobResumeSessionCheckpointRecord,
}

fn replay_fixture(case: &str) -> ReplayFixture {
    let (candidate, reachability, resumability) = publication_inputs(case);
    let logical = candidate.intent().logical_content_digest().clone();
    let staged = BlobReachabilityStaging::stage(candidate.clone(), reachability)
        .expect("stage reachability");
    let payload = BlobPublicationWalPayload::from_staged_reachability(&staged);
    let durable = durable_wal_publication(payload.frame_digest());
    let classification = replayable_wal_classification(payload.frame_digest());
    let wal_commit = BlobPublicationWalCommit::from_replayable_wal_record(
        staged.clone(),
        payload.clone(),
        durable.clone(),
        &classification,
    )
    .expect("publication wal commit should admit");
    let wal_record = BlobPublicationWalRecord::append(wal_commit);
    let object_id = candidate.intent().object_id().clone();
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
    let root =
        BlobRootCandidateRecord::from_checkpoint_frontier(frontier.clone(), candidate.clone())
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
    let records = BlobRecoveryRecordSet::from_admitted_replay_records(
        BlobAdmittedRecoveryRecords::new()
            .with_chunk_append(chunk.clone())
            .with_checkpoint_frontier(frontier.clone())
            .with_root_candidate(root.clone())
            .with_publication(publication.clone())
            .with_resume_session(resume.clone())
            .with_manifest(manifest.clone()),
    )
    .unwrap();
    ReplayFixture {
        records,
        object_id,
        chunk,
        frontier,
        root,
        publication,
        resume,
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
