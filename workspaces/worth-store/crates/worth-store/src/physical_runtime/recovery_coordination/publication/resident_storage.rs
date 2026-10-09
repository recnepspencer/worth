//! Heap backing retained by publication settlement evidence, not candidate
//! construction buffers which were released before this boundary.

use super::{
    CompletedPhysicalRecoveryPublicationCandidate, CompletedPhysicalRecoveryPublicationCommand,
    PhysicalRecoveryPublicationCandidateMaterialization, PhysicalRecoveryPublicationCommandDenial,
    PhysicalRecoveryPublicationCommandIndeterminate,
};

fn candidates_bytes(candidates: &[CompletedPhysicalRecoveryPublicationCandidate]) -> Option<u64> {
    candidates.iter().try_fold(
        u64::try_from(std::mem::size_of_val(candidates)).ok()?,
        |bytes, candidate| bytes.checked_add(candidate.owned_heap_bytes()?),
    )
}

impl CompletedPhysicalRecoveryPublicationCandidate {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        self.materialization()
            .owned_heap_bytes()?
            .checked_add(self.synchronization().owned_heap_bytes()?)
    }
}

impl PhysicalRecoveryPublicationCandidateMaterialization {
    fn owned_heap_bytes(&self) -> Option<u64> {
        self.physical().owned_heap_bytes()
    }
}

impl CompletedPhysicalRecoveryPublicationCommand {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        candidates_bytes(&self.candidates)?
            .checked_add(self.root_protocol.owned_heap_bytes()?)?
            .checked_add(self.record_namespace.owned_heap_bytes()?)
    }
}

impl PhysicalRecoveryPublicationCommandDenial {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        candidates_bytes(&self.candidates)?
            .checked_add(
                self.candidate_materialization
                    .as_ref()
                    .map_or(Some(0), |value| value.owned_heap_bytes())?,
            )?
            .checked_add(
                self.root_protocol
                    .as_ref()
                    .map_or(Some(0), |value| value.owned_heap_bytes())?,
            )
    }
}

impl PhysicalRecoveryPublicationCommandIndeterminate {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        match self {
            Self::CandidateMaterialization {
                physical,
                completed,
                ..
            } => candidates_bytes(completed)?.checked_add(physical.owned_heap_bytes()?),
            Self::CandidateSynchronization {
                physical,
                materialization,
                completed,
                ..
            } => candidates_bytes(completed)?
                .checked_add(physical.owned_heap_bytes()?)?
                .checked_add(materialization.owned_heap_bytes()?),
            Self::CandidateSynchronizationSettlement {
                physical,
                materialization,
                completed,
                ..
            }
            | Self::CandidateSynchronizationYieldpoint {
                physical,
                materialization,
                completed,
                ..
            } => candidates_bytes(completed)?
                .checked_add(physical.owned_heap_bytes()?)?
                .checked_add(materialization.owned_heap_bytes()?),
            Self::CandidateMaterializationSettlement {
                physical,
                completed,
                ..
            }
            | Self::CandidateMaterializationYieldpoint {
                physical,
                completed,
                ..
            } => candidates_bytes(completed)?.checked_add(physical.owned_heap_bytes()?),
            Self::Media {
                physical,
                candidates,
                root_protocol,
                ..
            } => candidates_bytes(candidates)?
                .checked_add(physical.owned_heap_bytes()?)?
                .checked_add(
                    root_protocol
                        .as_ref()
                        .map_or(Some(0), |value| value.owned_heap_bytes())?,
                ),
            Self::Scheduler {
                physical,
                candidates,
                root_protocol,
                ..
            }
            | Self::Signal {
                physical,
                candidates,
                root_protocol,
                ..
            }
            | Self::Yieldpoint {
                physical,
                candidates,
                root_protocol,
                ..
            } => candidates_bytes(candidates)?
                .checked_add(physical.owned_heap_bytes()?)?
                .checked_add(
                    root_protocol
                        .as_ref()
                        .map_or(Some(0), |value| value.owned_heap_bytes())?,
                ),
        }
    }
}
