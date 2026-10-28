use super::{
    BlobPersistedResumeCheckpointSource, BlobResumeCheckpoint, BlobResumeCheckpointReadmission,
    BlobResumeCheckpointStateKind, BlobResumeCounterSnapshot, BlobResumeDenial,
    BlobResumeReadmissionAuthority, BlobResumeUnfinishedState,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobResumeRootPublicationReadyReadmitted {
    source: BlobPersistedResumeCheckpointSource,
    reachability_staging: crate::BlobReachabilityStaging,
    counters: BlobResumeCounterSnapshot,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BlobResumeReplayOutcome {
    RootPublicationReady(Box<BlobResumeRootPublicationReadyReadmitted>),
    Unfinished {
        state: BlobResumeUnfinishedState,
        checkpoint: Box<BlobResumeCheckpoint>,
        counters: BlobResumeCounterSnapshot,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobResumeReplay;

impl BlobResumeReplay {
    pub fn readmit_checkpoint(
        checkpoint: BlobResumeCheckpoint,
        authority: BlobResumeReadmissionAuthority,
    ) -> Result<BlobResumeReplayOutcome, BlobResumeDenial> {
        let readmission = readmission_ticket(checkpoint, &authority)?;
        readmit_checkpoint(readmission, None)
    }

    pub fn readmit_checkpoint_for_security_scope(
        checkpoint: BlobResumeCheckpoint,
        authority: BlobResumeReadmissionAuthority,
        expected: crate::BlobChunkSecurityMetadataWitness,
    ) -> Result<BlobResumeReplayOutcome, BlobResumeDenial> {
        let readmission = readmission_ticket(checkpoint, &authority)?;
        readmit_checkpoint(readmission, Some(expected))
    }

    pub fn readmission_ticket(
        checkpoint: BlobResumeCheckpoint,
        authority: &BlobResumeReadmissionAuthority,
    ) -> Result<BlobResumeCheckpointReadmission, BlobResumeDenial> {
        checkpoint
            .readmit(authority)
            .ok_or(BlobResumeDenial::CopiedCheckpointAuthority)
    }
}

impl BlobResumeRootPublicationReadyReadmitted {
    pub const fn source(&self) -> &BlobPersistedResumeCheckpointSource {
        &self.source
    }

    pub const fn reachability_staging(&self) -> &crate::BlobReachabilityStaging {
        &self.reachability_staging
    }

    pub fn session_digest(&self) -> &str {
        self.source.session_id().as_str()
    }

    pub fn checkpoint_digest(&self) -> &str {
        self.source.checkpoint_identity().as_str()
    }

    pub fn chunk_tree_root_digest(&self) -> &str {
        self.reachability_staging
            .staging_identity()
            .chunk_tree_root()
            .digest()
            .as_str()
    }

    pub fn logical_content_digest(&self) -> &str {
        self.reachability_staging
            .staging_identity()
            .logical_content_digest()
            .digest()
            .as_str()
    }

    pub const fn counters(&self) -> BlobResumeCounterSnapshot {
        self.counters
    }
}

fn readmit_checkpoint(
    readmission: BlobResumeCheckpointReadmission,
    expected: Option<crate::BlobChunkSecurityMetadataWitness>,
) -> Result<BlobResumeReplayOutcome, BlobResumeDenial> {
    let (checkpoint, persisted_source) = readmission.into_parts();
    if checkpoint.stale() {
        return Err(BlobResumeDenial::StaleSessionId);
    }
    if expected.is_some_and(|expected| expected != checkpoint.security_metadata()) {
        return Err(BlobResumeDenial::WrongSecurityScope);
    }
    if let Some(frontier) = checkpoint.frontier()
        && frontier.proof_frontier().total_bytes() < checkpoint.declared_total_bytes()
        && checkpoint.state() != BlobResumeCheckpointStateKind::RootPublicationReady
    {
        return Ok(unfinished(
            BlobResumeUnfinishedState::MissingChunkTail,
            checkpoint,
        ));
    }
    match checkpoint.state() {
        BlobResumeCheckpointStateKind::SessionDeclared => Ok(unfinished(
            BlobResumeUnfinishedState::SessionDeclaredWithoutAdmission,
            checkpoint,
        )),
        BlobResumeCheckpointStateKind::SessionAdmitted => Ok(unfinished(
            BlobResumeUnfinishedState::SessionAdmittedWithoutChunkAppend,
            checkpoint,
        )),
        BlobResumeCheckpointStateKind::ChunkAppendStarted => Ok(unfinished(
            BlobResumeUnfinishedState::ChunkAppendWithoutDurableBytes,
            checkpoint,
        )),
        BlobResumeCheckpointStateKind::ChunkBytesDurable => Ok(unfinished(
            BlobResumeUnfinishedState::ChunkBytesWithoutChecksumAdmission,
            checkpoint,
        )),
        BlobResumeCheckpointStateKind::ChunkIntegrityAdmitted => Ok(unfinished(
            BlobResumeUnfinishedState::ChecksumAdmissionWithoutDurableFrontier,
            checkpoint,
        )),
        BlobResumeCheckpointStateKind::FrontierCheckpointed => Ok(unfinished(
            BlobResumeUnfinishedState::DurableFrontierWithoutRootNode,
            checkpoint,
        )),
        BlobResumeCheckpointStateKind::RootCandidateBuilt => Ok(unfinished(
            BlobResumeUnfinishedState::RootNodeWithoutReachabilityStaging,
            checkpoint,
        )),
        BlobResumeCheckpointStateKind::SessionClosedWithOrphanChunks => Ok(unfinished(
            BlobResumeUnfinishedState::ClosedSessionWithOrphanChunks,
            checkpoint,
        )),
        BlobResumeCheckpointStateKind::RootPublicationReady => {
            let staging = checkpoint
                .reachability_staging()
                .ok_or(BlobResumeDenial::RootCandidateMismatch)?;
            Ok(BlobResumeReplayOutcome::RootPublicationReady(Box::new(
                BlobResumeRootPublicationReadyReadmitted {
                    source: persisted_source,
                    reachability_staging: staging.clone(),
                    counters: checkpoint.counters().replayed(),
                },
            )))
        }
        BlobResumeCheckpointStateKind::BlobPublished => Ok(unfinished(
            BlobResumeUnfinishedState::BlobPublishedAwaitingSessionCloseout,
            checkpoint,
        )),
        BlobResumeCheckpointStateKind::SessionClosed => Ok(unfinished(
            BlobResumeUnfinishedState::SessionClosed,
            checkpoint,
        )),
        BlobResumeCheckpointStateKind::SessionAbandoned => Ok(unfinished(
            BlobResumeUnfinishedState::SessionAbandonedAwaitingReclaim,
            checkpoint,
        )),
        BlobResumeCheckpointStateKind::SessionReclaimed => Ok(unfinished(
            BlobResumeUnfinishedState::SessionReclaimed,
            checkpoint,
        )),
    }
}

fn readmission_ticket(
    checkpoint: BlobResumeCheckpoint,
    authority: &BlobResumeReadmissionAuthority,
) -> Result<BlobResumeCheckpointReadmission, BlobResumeDenial> {
    if checkpoint.stale() {
        return Err(BlobResumeDenial::StaleSessionId);
    }
    if checkpoint.authority_digest() != authority.authority_digest() {
        return Err(BlobResumeDenial::WrongStoreAuthority);
    }
    BlobResumeReplay::readmission_ticket(checkpoint, authority)
}

fn unfinished(
    state: BlobResumeUnfinishedState,
    checkpoint: BlobResumeCheckpoint,
) -> BlobResumeReplayOutcome {
    BlobResumeReplayOutcome::Unfinished {
        state,
        counters: checkpoint.counters().replayed(),
        checkpoint: Box::new(checkpoint),
    }
}
