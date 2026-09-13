use worth_store_physical_backend::BackendTargetProfile;
use worth_store_physical_format::{
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId, PhysicalRecordSlot,
    PhysicalReferenceAuthority, PhysicalSegmentId,
};
use worth_store_physical_isolation::{
    BlobOrphanReclaimProof, CurrentGenerationPhysicalReference, GenerationCountedPhysicalReference,
    ReclaimEligibilityProof,
};
use worth_store_security::StoreTenantScope;
use worth_store_wal::{
    BlobWalRecordEnvelope, BlobWalRecordIdentity, BlobWalRecordKind, LogSequenceNumber,
    PublicationDeclaration, WalFramePublicationScope, WalLsnRange, WalSegmentGeneration,
    WalSegmentId,
};

use super::super::replay_source::{BlobReplaySourceIdentity, BlobReplaySourceIdentityKind};
use crate::lifecycle::generation_registry_test_support::current_authority;
use crate::publication::test_support::publication_inputs_with_bytes_and_chunk_size;
use crate::test_support::{admitted_multichunk_sequence_for_scope, blob_scope};
use crate::{
    BlobChunkSize, BlobChunkingRuleAdmission, BlobInterruptedIngestRecovery,
    BlobReplaySourceAdmission, BlobResumeCheckpointStateKind, BlobResumeDenial,
    BlobResumeReadmissionAuthority, BlobResumeReplay, BlobResumeReplayOutcome,
    BlobResumeReplayReadmission, BlobResumeRootPublicationReady, BlobResumeSessionAbandoned,
    BlobResumeSessionAdmitted, BlobResumeSessionDeclaration, BlobResumeStoreAuthority,
    BlobResumeUnfinishedState, BlobStreamingContentFrontier,
};

#[path = "orphan_reclaim_tests.rs"]
mod orphan_reclaim_tests;

#[test]
fn interrupted_resume_replay_localizes_each_unfinished_state() {
    let lane = resume_lane("phase12-replay", b"aaaabbbbcccc", 12, 12);
    assert_unfinished(
        lane.durable.export_checkpoint(),
        &lane,
        BlobResumeUnfinishedState::ChunkBytesWithoutChecksumAdmission,
    );
    assert_unfinished(
        lane.integrity.export_checkpoint(),
        &lane,
        BlobResumeUnfinishedState::ChecksumAdmissionWithoutDurableFrontier,
    );
    assert_unfinished(
        lane.checkpointed.export_checkpoint(),
        &lane,
        BlobResumeUnfinishedState::DurableFrontierWithoutRootNode,
    );
    assert_unfinished(
        lane.root_candidate.export_checkpoint(),
        &lane,
        BlobResumeUnfinishedState::RootNodeWithoutReachabilityStaging,
    );
}

#[test]
fn root_publication_ready_replay_preserves_final_blob_identity() {
    let lane = resume_lane("phase12-ready", b"aaaabbbbcccc", 12, 12);
    let checkpoint = lane.ready.export_checkpoint();
    let readmission = readmission_authority_for_checkpoint(&checkpoint, &lane);
    let recovery =
        BlobInterruptedIngestRecovery::from_persisted_checkpoint(checkpoint, readmission)
            .expect("ready replay");
    let BlobResumeReplayOutcome::RootPublicationReady(ready) = recovery.outcome() else {
        panic!("ready replay should resume root publication");
    };
    let staged = lane.ready.reachability_staging().staging_identity();
    assert_eq!(
        ready.chunk_tree_root_digest(),
        staged.chunk_tree_root().digest().as_str()
    );
    assert_eq!(
        ready.logical_content_digest(),
        staged.logical_content_digest().digest().as_str()
    );
}

#[test]
fn stale_wrong_scope_wrong_authority_and_missing_tail_checkpoints_cannot_resume() {
    let stale = resume_lane("phase12-stale", b"aaaabbbbcccc", 12, 12);
    let stale_result = BlobInterruptedIngestRecovery::from_persisted_checkpoint(
        stale
            .checkpointed
            .export_checkpoint()
            .mark_stale_for_replay_test(),
        readmission_authority_for_digest("checkpoint:test", &stale),
    );
    assert_eq!(stale_result, Err(BlobResumeDenial::StaleSessionId));

    let wrong_authority = BlobInterruptedIngestRecovery::from_persisted_checkpoint(
        stale.checkpointed.export_checkpoint(),
        readmission_authority("phase12-other-authority"),
    );
    assert_eq!(wrong_authority, Err(BlobResumeDenial::WrongStoreAuthority));

    let wrong_scope = resume_lane("phase12-scope", b"aaaabbbbcccc", 12, 12);
    let other_scope = blob_scope(
        "phase12-other-scope",
        StoreTenantScope::MultiTenantPhysicalBoundary,
    );
    let other_sequence = admitted_multichunk_sequence_for_scope(other_scope, b"aaaabbbbcccc", 12);
    let other_metadata = other_sequence.proof_frontier().ordered_leaves()[0].security_metadata();
    let wrong_scope_checkpoint = wrong_scope.checkpointed.export_checkpoint();
    let wrong_scope_readmission =
        readmission_authority_for_checkpoint(&wrong_scope_checkpoint, &wrong_scope);
    let scope_result = BlobResumeReplay::readmit_checkpoint_for_security_scope(
        wrong_scope_checkpoint,
        wrong_scope_readmission,
        other_metadata,
    );
    assert_eq!(scope_result, Err(BlobResumeDenial::WrongSecurityScope));

    let missing_tail = resume_lane("phase12-missing-tail", b"aaaabbbbcccc", 12, 20);
    assert_unfinished(
        missing_tail.checkpointed.export_checkpoint(),
        &missing_tail,
        BlobResumeUnfinishedState::MissingChunkTail,
    );
}

#[test]
fn replay_reports_distinct_active_session_lifecycle_outcomes() {
    let lane = resume_lane("phase12-distinct-active", b"aaaabbbbcccc", 12, 12);
    assert_unfinished(
        lane.declaration
            .export_checkpoint(
                lane.store_authority.clone(),
                wal_record(
                    BlobWalRecordKind::SessionCheckpoint,
                    10,
                    "phase12-distinct-active",
                ),
            )
            .unwrap(),
        &lane,
        BlobResumeUnfinishedState::SessionDeclaredWithoutAdmission,
    );
    assert_unfinished(
        lane.admitted
            .export_checkpoint(wal_record(
                BlobWalRecordKind::SessionCheckpoint,
                11,
                "phase12-distinct-active",
            ))
            .unwrap(),
        &lane,
        BlobResumeUnfinishedState::SessionAdmittedWithoutChunkAppend,
    );
    assert_unfinished(
        lane.append_started
            .export_checkpoint(wal_record(
                BlobWalRecordKind::SessionCheckpoint,
                12,
                "phase12-distinct-active",
            ))
            .unwrap(),
        &lane,
        BlobResumeUnfinishedState::ChunkAppendWithoutDurableBytes,
    );
    assert_unfinished(
        lane.ready
            .export_checkpoint()
            .with_state(BlobResumeCheckpointStateKind::BlobPublished),
        &lane,
        BlobResumeUnfinishedState::BlobPublishedAwaitingSessionCloseout,
    );
}

#[test]
fn replay_reports_distinct_terminal_session_lifecycle_outcomes() {
    let lane = resume_lane("phase12-distinct-terminal", b"aaaabbbbcccc", 12, 12);
    let closed = lane
        .ready
        .clone()
        .close_session(wal_record(
            BlobWalRecordKind::SessionCloseout,
            13,
            "phase12-distinct-terminal",
        ))
        .unwrap();
    assert_unfinished(
        closed.export_checkpoint(),
        &lane,
        BlobResumeUnfinishedState::SessionClosed,
    );

    let abandoned =
        BlobResumeSessionAbandoned::abandon(lane.checkpointed.export_checkpoint()).unwrap();
    assert_unfinished(
        abandoned.into_checkpoint(),
        &lane,
        BlobResumeUnfinishedState::SessionAbandonedAwaitingReclaim,
    );

    let reclaim_lane = resume_lane("phase12-distinct-reclaimed", b"mmmmoooopppp", 12, 12);
    let abandoned =
        BlobResumeSessionAbandoned::abandon(reclaim_lane.checkpointed.export_checkpoint()).unwrap();
    let coverage = abandoned
        .reclaim_barrier()
        .clone()
        .admit_reclaim_coverage(reclaim_evidence_for_barrier(&abandoned))
        .unwrap();
    let proof = BlobOrphanReclaimProof::from_reclaim_coverage(coverage);
    let reclaimed = crate::BlobResumeSessionReclaimed::reclaim(abandoned, proof).unwrap();
    assert_unfinished(
        reclaimed.into_checkpoint(),
        &reclaim_lane,
        BlobResumeUnfinishedState::SessionReclaimed,
    );
}

struct ResumeLane {
    declaration: BlobResumeSessionDeclaration,
    admitted: BlobResumeSessionAdmitted,
    append_started: crate::BlobResumeChunkAppendStarted,
    durable: crate::BlobResumeChunkBytesDurable,
    integrity: crate::BlobResumeChunkIntegrityAdmitted,
    checkpointed: crate::BlobResumeFrontierCheckpointed,
    root_candidate: crate::BlobResumeRootCandidateBuilt,
    ready: BlobResumeRootPublicationReady,
    store_authority: BlobResumeStoreAuthority,
    authority_case: String,
}

fn resume_lane(case: &str, bytes: &[u8], chunk_size: u64, declared_total: u64) -> ResumeLane {
    let scope = blob_scope(case, StoreTenantScope::TenantPhysicalBoundary);
    let sequence = admitted_multichunk_sequence_for_scope(scope, bytes, chunk_size);
    let leaf = sequence.proof_frontier().ordered_leaves()[0].clone();
    let frontier = BlobStreamingContentFrontier::from_sequence(&sequence);
    let rule =
        BlobChunkingRuleAdmission::fixed_size(BlobChunkSize::from_bytes(chunk_size).unwrap())
            .unwrap();
    let authority_case = format!("{case}.resume-authority");
    let authority = current_authority(&authority_case, "resume");
    let physical_reference = physical_reference(case.len() as u16 + 1);
    let store_authority = BlobResumeStoreAuthority::from_current_store_authority(authority);
    let declaration =
        BlobResumeSessionDeclaration::new(leaf.security_metadata(), rule, declared_total).unwrap();
    let admitted = BlobResumeSessionAdmitted::admit(declaration.clone(), store_authority.clone());
    let append_started = admitted.clone().start_chunk_append(leaf.ordinal());
    let durable = append_started
        .clone()
        .record_chunk_bytes_durable(
            wal_record(BlobWalRecordKind::ChunkAppend, 1, case),
            bytes.len() as u64,
            physical_reference,
        )
        .unwrap();
    let integrity = durable.clone().admit_chunk_integrity(leaf).unwrap();
    let checkpointed = integrity
        .clone()
        .checkpoint_frontier(
            frontier,
            wal_record(BlobWalRecordKind::SessionCheckpoint, 2, case),
        )
        .unwrap();
    let (candidate, reachability, _) =
        publication_inputs_with_bytes_and_chunk_size(case, bytes, chunk_size);
    let root_candidate = checkpointed
        .clone()
        .build_root_candidate(candidate)
        .unwrap();
    let ready = root_candidate
        .clone()
        .stage_reachability(reachability)
        .unwrap();
    ResumeLane {
        declaration,
        admitted,
        append_started,
        durable,
        integrity,
        checkpointed,
        root_candidate,
        ready,
        store_authority,
        authority_case,
    }
}

fn reclaim_evidence_for_barrier(abandoned: &BlobResumeSessionAbandoned) -> ReclaimEligibilityProof {
    ReclaimEligibilityProof::for_certification_reference(
        abandoned.reclaim_barrier().orphan().physical_reference(),
    )
}

fn physical_reference(slot: u16) -> CurrentGenerationPhysicalReference {
    let generations = PhysicalGenerationAuthority::for_canonical_physical_format();
    let references = PhysicalReferenceAuthority::for_canonical_physical_format();
    let generation = PhysicalGeneration::from_raw(7).expect("generation");
    let cell = generations
        .slot_cell(
            PhysicalSegmentId::from_raw(1).expect("segment"),
            PhysicalPageId::from_raw(1).expect("page"),
            PhysicalRecordSlot::from_raw(slot).expect("slot"),
        )
        .with_slot_generation(generation);
    GenerationCountedPhysicalReference::from_admitted_reference(references.admit_page_slot(cell))
        .require_current_generation(generation)
        .expect("current generation reference")
}

fn assert_unfinished(
    checkpoint: crate::BlobResumeCheckpoint,
    lane: &ResumeLane,
    expected: BlobResumeUnfinishedState,
) {
    let readmission = readmission_authority_for_checkpoint(&checkpoint, lane);
    let recovery =
        BlobInterruptedIngestRecovery::from_persisted_checkpoint(checkpoint, readmission)
            .expect("checkpoint should readmit");
    let BlobResumeReplayOutcome::Unfinished { state, .. } = recovery.outcome() else {
        panic!("checkpoint should localize unfinished state");
    };
    assert_eq!(*state, expected);
}

fn readmission_authority(case: &str) -> BlobResumeReadmissionAuthority {
    let source = replay_source_for_checkpoint_digest("wrong-authority-checkpoint");
    let authority = current_authority(&format!("{case}.resume-authority"), "resume");
    BlobResumeReadmissionAuthority::from_recovery_readmission(
        BlobResumeReplayReadmission::from_checkpoint_source(&source, authority)
            .expect("checkpoint source readmission"),
    )
}

fn readmission_authority_for_checkpoint(
    checkpoint: &crate::BlobResumeCheckpoint,
    lane: &ResumeLane,
) -> BlobResumeReadmissionAuthority {
    readmission_authority_for_digest(checkpoint.checkpoint_identity().as_str(), lane)
}

fn readmission_authority_for_digest(
    checkpoint_digest: &str,
    lane: &ResumeLane,
) -> BlobResumeReadmissionAuthority {
    let source = replay_source_for_checkpoint_digest(checkpoint_digest);
    let authority = current_authority(&lane.authority_case, "resume");
    BlobResumeReadmissionAuthority::from_recovery_readmission(
        BlobResumeReplayReadmission::from_checkpoint_source(&source, authority)
            .expect("checkpoint source readmission"),
    )
}

fn replay_source_for_checkpoint_digest(checkpoint_digest: &str) -> BlobReplaySourceAdmission {
    let identity = BlobReplaySourceIdentity::new(
        BlobReplaySourceIdentityKind::Checkpoint,
        BackendTargetProfile::SimulatedStrictDurable,
        checkpoint_digest,
        1,
        2,
    )
    .unwrap();
    BlobReplaySourceAdmission::from_checkpoint_replay_identity(&identity)
        .expect("checkpoint replay source")
}

fn wal_record(kind: BlobWalRecordKind, sequence: u64, case: &str) -> BlobWalRecordEnvelope {
    let payload = format!("{case}:{kind:?}:{sequence}");
    let scope = WalFramePublicationScope::new(
        WalSegmentId::new(9).unwrap(),
        WalSegmentGeneration::new(1).unwrap(),
        WalLsnRange::new(
            LogSequenceNumber::new(sequence),
            LogSequenceNumber::new(sequence + 1),
        )
        .unwrap(),
        &payload,
        64,
    )
    .expect("wal scope");
    BlobWalRecordEnvelope::new(
        BlobWalRecordIdentity::new(sequence, kind).expect("wal identity"),
        PublicationDeclaration::wal_frame(scope),
        payload,
    )
    .expect("wal envelope")
}
