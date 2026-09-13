use crate::{OfflinePhysicalArtifactFamily, VerifiedRootManifestArtifact};
use worth_store_wal::artifact_store::BoundedWalSegmentObservation;

use crate::backup_verification::BoundedCheckpointBackupObservation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObservedRecoveryFrontier {
    RootManifest {
        root_reference: u64,
        generation: u64,
    },
    Checkpoint {
        checkpoint_identity_digest: [u8; 32],
        manifest_generation: u64,
        durable_checkpoint_lsn: u64,
        root_reference: u64,
        root_generation: u64,
        covered_lsn_start: u64,
        covered_lsn_end_exclusive: u64,
        redo_lsn: u64,
    },
    WalSegment {
        segment_id: u64,
        generation: u64,
        start_lsn: u64,
        end_exclusive_lsn: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryCandidateObservation {
    family: OfflinePhysicalArtifactFamily,
    frontier: ObservedRecoveryFrontier,
    content_digest: [u8; 32],
}

impl RecoveryCandidateObservation {
    pub fn from_verified_root_manifest(verified: VerifiedRootManifestArtifact) -> Self {
        let root = verified.root();
        Self {
            family: OfflinePhysicalArtifactFamily::Manifest,
            frontier: ObservedRecoveryFrontier::RootManifest {
                root_reference: root.root_reference().get(),
                generation: root.generation().get(),
            },
            content_digest: verified.content_digest(),
        }
    }

    pub fn from_verified_checkpoint(verified: BoundedCheckpointBackupObservation) -> Self {
        Self {
            family: OfflinePhysicalArtifactFamily::Manifest,
            frontier: ObservedRecoveryFrontier::Checkpoint {
                checkpoint_identity_digest: verified.checkpoint_identity_digest(),
                manifest_generation: verified.manifest_generation(),
                durable_checkpoint_lsn: verified.durable_checkpoint_lsn(),
                root_reference: verified.root_reference(),
                root_generation: verified.root_generation(),
                covered_lsn_start: verified.covered_lsn().0,
                covered_lsn_end_exclusive: verified.covered_lsn().1,
                redo_lsn: verified.redo_lsn(),
            },
            content_digest: verified.artifact_digest(),
        }
    }

    pub fn from_verified_wal_segment(verified: BoundedWalSegmentObservation) -> Self {
        let (start_lsn, end_exclusive_lsn) = verified.lsn_interval();
        Self {
            family: OfflinePhysicalArtifactFamily::Wal,
            frontier: ObservedRecoveryFrontier::WalSegment {
                segment_id: verified.segment_id(),
                generation: verified.generation(),
                start_lsn,
                end_exclusive_lsn,
            },
            content_digest: verified.artifact_digest(),
        }
    }
}

#[cfg(test)]
pub(crate) fn synthetic_observation_for_test(digest_byte: u8) -> RecoveryCandidateObservation {
    RecoveryCandidateObservation {
        family: OfflinePhysicalArtifactFamily::Manifest,
        frontier: ObservedRecoveryFrontier::RootManifest {
            root_reference: 7,
            generation: 3,
        },
        content_digest: [digest_byte; 32],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryCandidateConfidence {
    OwnerAndIntegrityConfirmed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryCandidate {
    family: OfflinePhysicalArtifactFamily,
    frontier: ObservedRecoveryFrontier,
    content_digest: [u8; 32],
    confidence: RecoveryCandidateConfidence,
}

impl RecoveryCandidate {
    pub const fn family(&self) -> OfflinePhysicalArtifactFamily {
        self.family
    }
    pub const fn frontier(&self) -> ObservedRecoveryFrontier {
        self.frontier
    }
    pub const fn content_digest(&self) -> [u8; 32] {
        self.content_digest
    }
    pub const fn confidence(&self) -> RecoveryCandidateConfidence {
        self.confidence
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryCandidateSet {
    candidates: Vec<RecoveryCandidate>,
}

impl RecoveryCandidateSet {
    pub fn candidates(&self) -> &[RecoveryCandidate] {
        &self.candidates
    }

    pub fn owned_allocation_bytes(&self) -> Option<u64> {
        u64::try_from(self.candidates.capacity())
            .ok()?
            .checked_mul(std::mem::size_of::<RecoveryCandidate>() as u64)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryCandidateDiscoveryDenial {
    AllocationFailed,
    ConflictingFrontierEvidence,
}

pub fn discover_recovery_candidates(
    observations: impl IntoIterator<Item = RecoveryCandidateObservation>,
) -> Result<RecoveryCandidateSet, RecoveryCandidateDiscoveryDenial> {
    let observations = observations.into_iter();
    let mut candidates = Vec::new();
    candidates
        .try_reserve_exact(observations.size_hint().0)
        .map_err(|_| RecoveryCandidateDiscoveryDenial::AllocationFailed)?;
    for observation in observations {
        if candidates.len() == candidates.capacity() {
            candidates
                .try_reserve(1)
                .map_err(|_| RecoveryCandidateDiscoveryDenial::AllocationFailed)?;
        }
        candidates.push(RecoveryCandidate {
            family: observation.family,
            frontier: observation.frontier,
            content_digest: observation.content_digest,
            confidence: RecoveryCandidateConfidence::OwnerAndIntegrityConfirmed,
        });
    }
    candidates.sort_by(|left, right| {
        left.frontier
            .cmp(&right.frontier)
            .then_with(|| left.content_digest.cmp(&right.content_digest))
    });
    if candidates.windows(2).any(|pair| {
        pair[0].frontier == pair[1].frontier && pair[0].content_digest != pair[1].content_digest
    }) {
        return Err(RecoveryCandidateDiscoveryDenial::ConflictingFrontierEvidence);
    }
    candidates.dedup_by(|right, left| {
        right.frontier == left.frontier && right.content_digest == left.content_digest
    });
    Ok(RecoveryCandidateSet { candidates })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_owner_evidence_is_deduplicated_but_conflicting_frontiers_fail_closed() {
        let first = observation(1);
        let duplicate = observation(1);
        let deduplicated = discover_recovery_candidates([first, duplicate]).expect("dedupe");
        assert_eq!(deduplicated.candidates().len(), 1);

        let denial = discover_recovery_candidates([observation(1), observation(2)])
            .expect_err("one frontier cannot carry two physical digests");
        assert_eq!(
            denial,
            RecoveryCandidateDiscoveryDenial::ConflictingFrontierEvidence
        );
    }

    fn observation(digest_byte: u8) -> RecoveryCandidateObservation {
        RecoveryCandidateObservation {
            family: OfflinePhysicalArtifactFamily::Manifest,
            frontier: ObservedRecoveryFrontier::RootManifest {
                root_reference: 7,
                generation: 3,
            },
            content_digest: [digest_byte; 32],
        }
    }
}
