use crate::{BlobReplaySourceAdmission, BlobReplaySourceKind};
use worth_store_wal::PublicationDeclaration;

use crate::{
    BlobChunkSecurityMetadataWitness, BlobGeneration, BlobObjectClassification, BlobObjectId,
    BlobPublicationWalRecord, BlobReachabilityStagingIdentity, BlobRootCandidateForPublication,
    ChunkTreeRoot, LogicalContentDigest,
};

use super::{
    BlobCheckpointFrontierRecord, BlobRecoveryRecordCounterSnapshot, BlobRecoveryRecordDenial,
    BlobRecoveryRecordDenialKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobChunkAppendRecord {
    logical_content_digest: LogicalContentDigest,
    wal_source: BlobReplaySourceAdmission,
    counters: BlobRecoveryRecordCounterSnapshot,
}

impl BlobChunkAppendRecord {
    pub fn from_integrity_admission(
        logical_content_digest: LogicalContentDigest,
        wal_source: BlobReplaySourceAdmission,
    ) -> Result<Self, BlobRecoveryRecordDenial> {
        if wal_source.kind() != BlobReplaySourceKind::Wal {
            return Err(BlobRecoveryRecordDenial::start(
                BlobRecoveryRecordDenialKind::MissingWalSource,
            ));
        }
        Ok(Self {
            logical_content_digest,
            wal_source,
            counters: BlobRecoveryRecordCounterSnapshot::start().with_wal_record(),
        })
    }

    pub const fn logical_content_digest(&self) -> &LogicalContentDigest {
        &self.logical_content_digest
    }

    pub const fn wal_source(&self) -> &BlobReplaySourceAdmission {
        &self.wal_source
    }

    pub const fn counters(&self) -> BlobRecoveryRecordCounterSnapshot {
        self.counters
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobRootCandidateRecord {
    candidate: BlobRootCandidateForPublication,
    frontier: BlobCheckpointFrontierRecord,
    counters: BlobRecoveryRecordCounterSnapshot,
}

impl BlobRootCandidateRecord {
    pub fn from_checkpoint_frontier(
        frontier: BlobCheckpointFrontierRecord,
        candidate: BlobRootCandidateForPublication,
    ) -> Result<Self, BlobRecoveryRecordDenial> {
        if frontier.logical_content_digest() != candidate.intent().logical_content_digest() {
            return Err(BlobRecoveryRecordDenial::start(
                BlobRecoveryRecordDenialKind::CheckpointFrontierWithoutRootCandidate,
            ));
        }
        Ok(Self {
            candidate,
            frontier,
            counters: BlobRecoveryRecordCounterSnapshot::start().with_wal_record(),
        })
    }

    pub const fn candidate(&self) -> &BlobRootCandidateForPublication {
        &self.candidate
    }

    pub const fn chunk_tree_root(&self) -> &ChunkTreeRoot {
        self.candidate.intent().chunk_tree_root()
    }

    pub const fn frontier(&self) -> &BlobCheckpointFrontierRecord {
        &self.frontier
    }

    pub const fn counters(&self) -> BlobRecoveryRecordCounterSnapshot {
        self.counters
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobGenerationPublicationRecord {
    published: BlobRecoveredPublishedGeneration,
    root_candidate: BlobRootCandidateRecord,
    wal_source: BlobReplaySourceAdmission,
    counters: BlobRecoveryRecordCounterSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobRecoveredPublishedGeneration {
    object_id: BlobObjectId,
    generation: BlobGeneration,
    chunk_tree_root: ChunkTreeRoot,
    logical_content_digest: LogicalContentDigest,
    classification: BlobObjectClassification,
    publication_declaration: PublicationDeclaration,
    replay_classification_digest: String,
    staging_identity: BlobReachabilityStagingIdentity,
    security_metadata: BlobChunkSecurityMetadataWitness,
}

impl BlobGenerationPublicationRecord {
    pub fn from_replayed_wal_record(
        root_candidate: BlobRootCandidateRecord,
        wal_record: BlobPublicationWalRecord,
        wal_source: BlobReplaySourceAdmission,
    ) -> Result<Self, BlobRecoveryRecordDenial> {
        if wal_source.kind() != BlobReplaySourceKind::Wal {
            return Err(BlobRecoveryRecordDenial::start(
                BlobRecoveryRecordDenialKind::MissingWalSource,
            ));
        }
        if root_candidate.chunk_tree_root() != wal_record.intent().chunk_tree_root() {
            return Err(BlobRecoveryRecordDenial::start(
                BlobRecoveryRecordDenialKind::RootCandidateWithoutPublication,
            ));
        }
        let published = BlobRecoveredPublishedGeneration::from_wal_record(&wal_record);
        Ok(Self {
            published,
            root_candidate,
            wal_source,
            counters: BlobRecoveryRecordCounterSnapshot::start().with_wal_record(),
        })
    }

    pub const fn published(&self) -> &BlobRecoveredPublishedGeneration {
        &self.published
    }

    pub const fn root_candidate(&self) -> &BlobRootCandidateRecord {
        &self.root_candidate
    }

    pub const fn wal_source(&self) -> &BlobReplaySourceAdmission {
        &self.wal_source
    }

    pub const fn counters(&self) -> BlobRecoveryRecordCounterSnapshot {
        self.counters
    }
}

impl BlobRecoveredPublishedGeneration {
    fn from_wal_record(wal_record: &BlobPublicationWalRecord) -> Self {
        let intent = wal_record.intent();
        let commit = wal_record.wal_commit();
        Self {
            object_id: intent.object_id().clone(),
            generation: intent.generation(),
            chunk_tree_root: intent.chunk_tree_root().clone(),
            logical_content_digest: intent.logical_content_digest().clone(),
            classification: intent.classification(),
            publication_declaration: commit.publication_declaration().clone(),
            replay_classification_digest: commit.replay_classification_digest().to_owned(),
            staging_identity: commit.staging_identity().clone(),
            security_metadata: commit.security_metadata(),
        }
    }

    pub const fn object_id(&self) -> &BlobObjectId {
        &self.object_id
    }

    pub const fn generation(&self) -> BlobGeneration {
        self.generation
    }

    pub const fn chunk_tree_root(&self) -> &ChunkTreeRoot {
        &self.chunk_tree_root
    }

    pub const fn logical_content_digest(&self) -> &LogicalContentDigest {
        &self.logical_content_digest
    }

    pub const fn classification(&self) -> BlobObjectClassification {
        self.classification
    }

    pub const fn publication_declaration(&self) -> &PublicationDeclaration {
        &self.publication_declaration
    }

    pub fn replay_classification_digest(&self) -> &str {
        &self.replay_classification_digest
    }

    pub const fn staging_identity(&self) -> &BlobReachabilityStagingIdentity {
        &self.staging_identity
    }

    pub const fn security_metadata(&self) -> BlobChunkSecurityMetadataWitness {
        self.security_metadata
    }
}
