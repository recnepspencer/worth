use worth_store::physical_runtime::{
    BoundedRecoveryFilesystemDiscovery, ObservedRecoveryArtifact, RecoveryDiscoveryByteLimitScope,
    RecoveryDiscoveryFailure,
};
use worth_store_physical_format::RecordArtifactFile;

use super::artifact_generation;
use super::resident::memory_failure;
use crate::entry::PhysicalRecoverySuccessorCandidateDenial;
use crate::progression::{
    PlanningMemoryDenial, PlanningResidentAllowance, RecoveryObservedCandidateArtifact,
};

#[cfg(test)]
#[path = "artifact_read/tests.rs"]
mod tests;

/// Read only within both the discovery request and the still-live planning
/// allocation window. The backend checks file length before allocating its
/// exact-length byte vector; an absent optional artifact costs no resident
/// bytes even when the window has no space left.
pub(super) fn read(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    artifact: RecordArtifactFile,
    byte_limit: u64,
    allowance: &mut PlanningResidentAllowance,
) -> Result<ObservedRecoveryArtifact, PhysicalRecoverySuccessorCandidateDenial> {
    let resident_limit = allowance.remaining();
    let requested = byte_limit.min(resident_limit);
    let result = match artifact {
        RecordArtifactFile::RootManifest { generation } => {
            discovery.read_root_manifest(generation, requested)
        }
        RecordArtifactFile::RootRoutingBlock { generation, block } => {
            discovery.read_root_routing_block(generation, block, requested)
        }
        RecordArtifactFile::SegmentMembershipBlock { generation, block } => {
            discovery.read_segment_membership_block(generation, block, requested)
        }
        RecordArtifactFile::FreeSpaceManifest { generation } => {
            discovery.read_free_space_manifest(generation, requested)
        }
        RecordArtifactFile::FreeSpaceMembershipBlock { generation, block } => {
            discovery.read_free_space_membership_block(generation, block, requested)
        }
        _ => {
            return Err(PhysicalRecoverySuccessorCandidateDenial::InvalidArtifact {
                artifact,
                generation: artifact_generation(artifact),
            });
        }
    };
    match result {
        Ok(observed) => {
            let retained = observed.owned_heap_bytes().ok_or_else(|| {
                memory_failure(
                    artifact,
                    allowance,
                    PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX },
                )
            })?;
            allowance
                .retain(retained)
                .map_err(|failure| memory_failure(artifact, allowance, failure))?;
            Ok(observed)
        }
        Err(RecoveryDiscoveryFailure::ByteLimitExceeded {
            observed,
            scope: RecoveryDiscoveryByteLimitScope::Requested,
            ..
        }) if resident_limit < byte_limit && observed <= byte_limit => Err(memory_failure(
            artifact,
            allowance,
            PlanningMemoryDenial::RecoveryMemoryBytes {
                observed: allowance.used().checked_add(observed).unwrap_or(u64::MAX),
            },
        )),
        Err(failure) => Err(PhysicalRecoverySuccessorCandidateDenial::Discovery {
            artifact,
            generation: artifact_generation(artifact),
            failure,
        }),
    }
}

pub(super) fn retain_successor(
    successor: u64,
    artifact: RecordArtifactFile,
    bytes: Vec<u8>,
    artifacts: &mut Vec<RecoveryObservedCandidateArtifact>,
    allowance: &mut PlanningResidentAllowance,
) -> Result<bool, PhysicalRecoverySuccessorCandidateDenial> {
    let generation = match artifact {
        RecordArtifactFile::RootRoutingBlock { generation, .. }
        | RecordArtifactFile::SegmentMembershipBlock { generation, .. }
        | RecordArtifactFile::FreeSpaceMembershipBlock { generation, .. } => generation,
        _ => 0,
    };
    if generation == successor {
        allowance
            .grow(artifacts, 1)
            .map_err(|failure| memory_failure(artifact, allowance, failure))?;
        artifacts.push(observed(artifact, bytes, allowance)?);
        Ok(true)
    } else {
        let retained = PlanningResidentAllowance::vector_bytes(&bytes)
            .map_err(|failure| memory_failure(artifact, allowance, failure))?;
        drop(bytes);
        allowance.release(retained);
        Ok(false)
    }
}

pub(super) fn observed(
    artifact: RecordArtifactFile,
    bytes: Vec<u8>,
    allowance: &mut PlanningResidentAllowance,
) -> Result<RecoveryObservedCandidateArtifact, PhysicalRecoverySuccessorCandidateDenial> {
    let bytes = allowance
        .into_box(bytes)
        .map_err(|failure| memory_failure(artifact, allowance, failure))?;
    Ok(RecoveryObservedCandidateArtifact { artifact, bytes })
}
