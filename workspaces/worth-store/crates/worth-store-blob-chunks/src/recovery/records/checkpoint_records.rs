use crate::{BlobReplaySourceAdmission, BlobReplaySourceKind};

use crate::{BlobGeneration, BlobObjectId, BlobResumabilityReceipt, LogicalContentDigest};

use super::{
    BlobChunkAppendRecord, BlobGenerationPublicationRecord, BlobRecoveryRecordCounterSnapshot,
    BlobRecoveryRecordDenial, BlobRecoveryRecordDenialKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobCheckpointFrontierRecord {
    chunk_append: BlobChunkAppendRecord,
    checkpoint_source: BlobReplaySourceAdmission,
    counters: BlobRecoveryRecordCounterSnapshot,
}

impl BlobCheckpointFrontierRecord {
    pub fn from_chunk_append(
        chunk_append: BlobChunkAppendRecord,
        checkpoint_source: BlobReplaySourceAdmission,
    ) -> Result<Self, BlobRecoveryRecordDenial> {
        if checkpoint_source.kind() != BlobReplaySourceKind::Checkpoint {
            return Err(BlobRecoveryRecordDenial::start(
                BlobRecoveryRecordDenialKind::MissingCheckpointSource,
            ));
        }
        Ok(Self {
            chunk_append,
            checkpoint_source,
            counters: BlobRecoveryRecordCounterSnapshot::start().with_checkpoint_record(),
        })
    }

    pub const fn logical_content_digest(&self) -> &LogicalContentDigest {
        self.chunk_append.logical_content_digest()
    }

    pub const fn chunk_append(&self) -> &BlobChunkAppendRecord {
        &self.chunk_append
    }

    pub const fn checkpoint_source(&self) -> &BlobReplaySourceAdmission {
        &self.checkpoint_source
    }

    pub const fn counters(&self) -> BlobRecoveryRecordCounterSnapshot {
        self.counters
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobResumeSessionCheckpointRecord {
    session: BlobRecoveredResumeSession,
    checkpoint_source: BlobReplaySourceAdmission,
    counters: BlobRecoveryRecordCounterSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobRecoveredResumeSession {
    object_id: BlobObjectId,
    generation: BlobGeneration,
    logical_content_digest: LogicalContentDigest,
}

impl BlobResumeSessionCheckpointRecord {
    pub fn from_replayed_publication(
        publication: &BlobGenerationPublicationRecord,
        resumability: BlobResumabilityReceipt,
        checkpoint_source: BlobReplaySourceAdmission,
    ) -> Result<Self, BlobRecoveryRecordDenial> {
        if checkpoint_source.kind() != BlobReplaySourceKind::Checkpoint {
            return Err(BlobRecoveryRecordDenial::start(
                BlobRecoveryRecordDenialKind::MissingCheckpointSource,
            ));
        }
        if publication.published().logical_content_digest() != resumability.logical_content_digest()
        {
            return Err(BlobRecoveryRecordDenial::start(
                BlobRecoveryRecordDenialKind::IntegrityWithoutCheckpointFrontier,
            ));
        }
        let session = BlobRecoveredResumeSession {
            object_id: publication.published().object_id().clone(),
            generation: publication.published().generation(),
            logical_content_digest: resumability.logical_content_digest().clone(),
        };
        Ok(Self {
            session,
            checkpoint_source,
            counters: BlobRecoveryRecordCounterSnapshot::start().with_checkpoint_record(),
        })
    }

    pub const fn session(&self) -> &BlobRecoveredResumeSession {
        &self.session
    }

    pub const fn checkpoint_source(&self) -> &BlobReplaySourceAdmission {
        &self.checkpoint_source
    }

    pub const fn counters(&self) -> BlobRecoveryRecordCounterSnapshot {
        self.counters
    }
}

impl BlobRecoveredResumeSession {
    pub const fn object_id(&self) -> &BlobObjectId {
        &self.object_id
    }

    pub const fn generation(&self) -> BlobGeneration {
        self.generation
    }

    pub const fn logical_content_digest(&self) -> &LogicalContentDigest {
        &self.logical_content_digest
    }
}
