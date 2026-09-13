use crate::{BlobReplaySourceAdmission, BlobReplaySourceKind};
use worth_store_physical_backend::BlobPhysicalManifestValidation;

use crate::{
    BlobGeneration, BlobObjectClassification, BlobObjectId, ChunkTreeRoot, LogicalContentDigest,
};

use super::{
    BlobGenerationPublicationRecord, BlobRecoveryRecordCounterSnapshot, BlobRecoveryRecordDenial,
    BlobRecoveryRecordDenialKind, BlobRootCandidateRecord,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobReachabilityManifestRow {
    staged: BlobRecoveredReachabilityStaging,
    manifest_source: BlobReplaySourceAdmission,
    counters: BlobRecoveryRecordCounterSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobRecoveredReachabilityStaging {
    object_id: BlobObjectId,
    generation: BlobGeneration,
    chunk_tree_root: ChunkTreeRoot,
    logical_content_digest: LogicalContentDigest,
    classification: BlobObjectClassification,
}

impl BlobReachabilityManifestRow {
    pub fn from_root_candidate(
        root_candidate: &BlobRootCandidateRecord,
        manifest_source: BlobReplaySourceAdmission,
    ) -> Result<Self, BlobRecoveryRecordDenial> {
        if manifest_source.kind() != BlobReplaySourceKind::Manifest {
            return Err(BlobRecoveryRecordDenial::start(
                BlobRecoveryRecordDenialKind::MissingManifestSource,
            ));
        }
        let intent = root_candidate.candidate().intent();
        let staged = BlobRecoveredReachabilityStaging {
            object_id: intent.object_id().clone(),
            generation: intent.generation(),
            chunk_tree_root: intent.chunk_tree_root().clone(),
            logical_content_digest: intent.logical_content_digest().clone(),
            classification: intent.classification(),
        };
        Ok(Self {
            staged,
            manifest_source,
            counters: BlobRecoveryRecordCounterSnapshot::start().with_manifest_row(),
        })
    }

    pub const fn staged(&self) -> &BlobRecoveredReachabilityStaging {
        &self.staged
    }

    pub const fn manifest_source(&self) -> &BlobReplaySourceAdmission {
        &self.manifest_source
    }

    pub const fn counters(&self) -> BlobRecoveryRecordCounterSnapshot {
        self.counters
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobPlacementManifestRow {
    observation: BlobRecoveredPlacementObservation,
    manifest_source: BlobReplaySourceAdmission,
    counters: BlobRecoveryRecordCounterSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobRecoveredPlacementObservation {
    object_id: BlobObjectId,
    generation: BlobGeneration,
    chunk_tree_root: ChunkTreeRoot,
    logical_content_digest: LogicalContentDigest,
}

impl BlobPlacementManifestRow {
    pub fn from_replayed_publication(
        publication: &BlobGenerationPublicationRecord,
        manifest_source: BlobReplaySourceAdmission,
    ) -> Result<Self, BlobRecoveryRecordDenial> {
        if manifest_source.kind() != BlobReplaySourceKind::Manifest {
            return Err(BlobRecoveryRecordDenial::start(
                BlobRecoveryRecordDenialKind::MissingManifestSource,
            ));
        }
        let published = publication.published();
        let observation = BlobRecoveredPlacementObservation {
            object_id: published.object_id().clone(),
            generation: published.generation(),
            chunk_tree_root: published.chunk_tree_root().clone(),
            logical_content_digest: published.logical_content_digest().clone(),
        };
        Ok(Self {
            observation,
            manifest_source,
            counters: BlobRecoveryRecordCounterSnapshot::start().with_manifest_row(),
        })
    }

    pub const fn observation(&self) -> &BlobRecoveredPlacementObservation {
        &self.observation
    }

    pub const fn manifest_source(&self) -> &BlobReplaySourceAdmission {
        &self.manifest_source
    }

    pub const fn counters(&self) -> BlobRecoveryRecordCounterSnapshot {
        self.counters
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobManifestAgreement {
    reachability: BlobReachabilityManifestRow,
    placement: BlobPlacementManifestRow,
    validation: BlobPhysicalManifestValidation,
    counters: BlobRecoveryRecordCounterSnapshot,
}

impl BlobManifestAgreement {
    pub fn validate(
        reachability: BlobReachabilityManifestRow,
        placement: BlobPlacementManifestRow,
        validation: BlobPhysicalManifestValidation,
    ) -> Result<Self, BlobRecoveryRecordDenial> {
        if reachability.staged.object_id() != placement.observation.object_id() {
            return Err(BlobRecoveryRecordDenial::start(
                BlobRecoveryRecordDenialKind::PublicationWithoutManifestAgreement,
            ));
        }
        if reachability.staged.generation() != placement.observation.generation() {
            return Err(BlobRecoveryRecordDenial::start(
                BlobRecoveryRecordDenialKind::PublicationWithoutManifestAgreement,
            ));
        }
        if reachability.staged.chunk_tree_root() != placement.observation.chunk_tree_root()
            || reachability.staged.logical_content_digest()
                != placement.observation.logical_content_digest()
        {
            return Err(BlobRecoveryRecordDenial::start(
                BlobRecoveryRecordDenialKind::PublicationWithoutManifestAgreement,
            ));
        }
        let manifest_source_digest = reachability.manifest_source.source_digest();
        if manifest_source_digest != placement.manifest_source.source_digest() {
            return Err(BlobRecoveryRecordDenial::start(
                BlobRecoveryRecordDenialKind::PublicationWithoutManifestAgreement,
            ));
        }
        if validation.reachability_digest() != manifest_source_digest
            || validation.placement_digest() != manifest_source_digest
        {
            return Err(BlobRecoveryRecordDenial::start(
                BlobRecoveryRecordDenialKind::PublicationWithoutManifestAgreement,
            ));
        }
        if validation.reachability_generation_sequence()
            != reachability.staged.generation().sequence()
            || validation.placement_generation_sequence()
                != placement.observation.generation().sequence()
        {
            return Err(BlobRecoveryRecordDenial::start(
                BlobRecoveryRecordDenialKind::PublicationWithoutManifestAgreement,
            ));
        }
        let counters = reachability.counters().merge(placement.counters());
        Ok(Self {
            reachability,
            placement,
            validation,
            counters,
        })
    }

    pub const fn reachability(&self) -> &BlobReachabilityManifestRow {
        &self.reachability
    }

    pub const fn placement(&self) -> &BlobPlacementManifestRow {
        &self.placement
    }

    pub const fn validation(&self) -> &BlobPhysicalManifestValidation {
        &self.validation
    }

    pub const fn counters(&self) -> BlobRecoveryRecordCounterSnapshot {
        self.counters
    }
}

impl BlobRecoveredReachabilityStaging {
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
}

impl BlobRecoveredPlacementObservation {
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
}
