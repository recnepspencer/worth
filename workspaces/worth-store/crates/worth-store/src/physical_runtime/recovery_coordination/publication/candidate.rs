use sha2::{Digest, Sha256};
use worth_store_physical_backend::CompletedRecoveryStagingWrite;
use worth_store_physical_format::{RecordArtifactFile, RootSelectorRole};

use crate::physical_runtime::recovery_coordination::{
    PerformedRecoveryPhysicalEffect, RecoveryPublicationCandidateMaterializationAction,
    RecoveryPublicationCandidateSynchronizationAction,
};

use super::RecoveryRootProtocolPublicationPlan;

pub struct PhysicalRecoveryPublicationCandidate {
    artifact: RecordArtifactFile,
    bytes: Box<[u8]>,
    payload_digest: [u8; 32],
}

pub enum PhysicalRecoveryPublicationCandidateMaterialization {
    Created(PerformedRecoveryPhysicalEffect<RecoveryPublicationCandidateMaterializationAction>),
    AlreadyMaterialized(CompletedRecoveryStagingWrite),
    CompletedFromExactPrefix(
        PerformedRecoveryPhysicalEffect<RecoveryPublicationCandidateMaterializationAction>,
    ),
}

pub struct CompletedPhysicalRecoveryPublicationCandidate {
    materialization: PhysicalRecoveryPublicationCandidateMaterialization,
    synchronization:
        PerformedRecoveryPhysicalEffect<RecoveryPublicationCandidateSynchronizationAction>,
}

impl PhysicalRecoveryPublicationCandidate {
    pub fn new(
        artifact: RecordArtifactFile,
        bytes: Box<[u8]>,
        payload_digest: [u8; 32],
    ) -> Option<Self> {
        (!bytes.is_empty() && Sha256::digest(&bytes).as_slice() == payload_digest).then_some(Self {
            artifact,
            bytes,
            payload_digest,
        })
    }

    pub const fn artifact(&self) -> RecordArtifactFile {
        self.artifact
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub const fn payload_digest(&self) -> [u8; 32] {
        self.payload_digest
    }
}

impl CompletedPhysicalRecoveryPublicationCandidate {
    pub(super) const fn new(
        materialization: PhysicalRecoveryPublicationCandidateMaterialization,
        synchronization: PerformedRecoveryPhysicalEffect<
            RecoveryPublicationCandidateSynchronizationAction,
        >,
    ) -> Self {
        Self {
            materialization,
            synchronization,
        }
    }

    pub const fn materialization(&self) -> &PhysicalRecoveryPublicationCandidateMaterialization {
        &self.materialization
    }
    pub const fn synchronization(
        &self,
    ) -> &PerformedRecoveryPhysicalEffect<RecoveryPublicationCandidateSynchronizationAction> {
        &self.synchronization
    }
}

impl PhysicalRecoveryPublicationCandidateMaterialization {
    pub fn physical(&self) -> &CompletedRecoveryStagingWrite {
        match self {
            Self::Created(performed) | Self::CompletedFromExactPrefix(performed) => {
                match performed.occurrence() {
                super::super::RecoveryPhysicalEffectOccurrence::PublicationCandidateMaterialization(
                    occurrence,
                ) => occurrence.physical(),
                _ => unreachable!("publication-candidate evidence has its exact action"),
                }
            }
            Self::AlreadyMaterialized(physical) => physical,
        }
    }
}

pub(super) fn is_complete_and_canonical(
    candidates: &[PhysicalRecoveryPublicationCandidate],
    generation: u64,
    protocol: RecoveryRootProtocolPublicationPlan,
) -> bool {
    let mut previous = None;
    let mut root = false;
    let mut previous_selector = false;
    let mut current_selector = false;
    let mut catalog = false;
    for candidate in candidates {
        if previous.is_some_and(|artifact| artifact >= candidate.artifact) {
            return false;
        }
        previous = Some(candidate.artifact);
        match candidate.artifact {
            RecordArtifactFile::RootManifest {
                generation: observed,
            }
            | RecordArtifactFile::RootRoutingBlock {
                generation: observed,
                ..
            }
            | RecordArtifactFile::SegmentMembershipBlock {
                generation: observed,
                ..
            }
            | RecordArtifactFile::FreeSpaceManifest {
                generation: observed,
            }
            | RecordArtifactFile::FreeSpaceMembershipBlock {
                generation: observed,
                ..
            } if observed == generation => {
                root |= matches!(candidate.artifact, RecordArtifactFile::RootManifest { .. });
            }
            RecordArtifactFile::RootSelectorCandidate {
                role: RootSelectorRole::Previous,
                publication,
            } if publication == protocol.publication() => previous_selector = true,
            RecordArtifactFile::RootSelectorCandidate {
                role: RootSelectorRole::Current,
                publication,
            } if publication == protocol.publication() => current_selector = true,
            RecordArtifactFile::CatalogCandidate { publication }
                if publication == protocol.publication() =>
            {
                catalog = true
            }
            _ => return false,
        }
    }
    root && previous_selector && current_selector && catalog
}
