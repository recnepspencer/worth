use worth_store::physical_runtime::{
    ArtifactCeiling, BoundedRecoveryFilesystemDiscovery, GrantedRead, GrantedReadStop,
    ObservedRecoveryArtifact, PageAddress,
};
use worth_store_physical_format::{PhysicalRecordFormatDeclaration, RecordArtifactFile};

use super::artifact_generation;
use super::resident::memory_failure;
use crate::entry::PhysicalRecoverySuccessorCandidateDenial;
use crate::orchestration::planning::manifest_entry_budget::ChargeToken;
use crate::progression::{
    PlanningMemoryDenial, PlanningResidentAllowance, RecoveryObservedCandidateArtifact,
};

#[cfg(test)]
#[path = "artifact_read/tests.rs"]
mod tests;

/// Read only within both the artifact's own ceiling and the still-live
/// planning allocation window. Each manifest artifact of a candidate is at
/// most one page of `format`, so a larger one is damage and no budget stands
/// in for that ceiling. The backend checks file length before allocating its
/// exact-length byte vector; an absent optional artifact costs no resident
/// bytes even when the window has no space left.
///
/// `_paid` is the charge of the candidate root whose page this is. One charge
/// pays for every page of its root, so the wrapper that holds it lends it to
/// each read, and the root's last read spends it.
pub(super) fn read(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    artifact: RecordArtifactFile,
    format: PhysicalRecordFormatDeclaration,
    _paid: &ChargeToken,
    allowance: &mut PlanningResidentAllowance,
) -> Result<ObservedRecoveryArtifact, PhysicalRecoverySuccessorCandidateDenial> {
    let address = match artifact {
        RecordArtifactFile::RootManifest { generation } => PageAddress::RootManifest { generation },
        RecordArtifactFile::RootRoutingBlock { generation, block } => {
            PageAddress::RootRoutingBlock { generation, block }
        }
        RecordArtifactFile::SegmentMembershipBlock { generation, block } => {
            PageAddress::SegmentMembershipBlock { generation, block }
        }
        RecordArtifactFile::FreeSpaceManifest { generation } => {
            PageAddress::FreeSpaceManifest { generation }
        }
        RecordArtifactFile::FreeSpaceMembershipBlock { generation, block } => {
            PageAddress::FreeSpaceMembershipBlock { generation, block }
        }
        _ => {
            return Err(PhysicalRecoverySuccessorCandidateDenial::InvalidArtifact {
                artifact,
                generation: artifact_generation(artifact),
            });
        }
    };
    let read = discovery
        .read(
            ArtifactCeiling::page(format, address),
            allowance.grant_read(),
        )
        .granted();
    match read {
        Ok(observed) => {
            let retained = observed.owned_heap_bytes().ok_or_else(|| {
                memory_failure(
                    artifact,
                    PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX },
                )
            })?;
            allowance
                .retain(retained)
                .map_err(|failure| memory_failure(artifact, failure))?;
            Ok(observed)
        }
        Err(GrantedReadStop::PastGrant(overrun)) => {
            Err(memory_failure(artifact, allowance.refuse_read(overrun)))
        }
        // Past its own ceiling the artifact is damage, as is every refusal
        // that is not the window's own.
        Err(GrantedReadStop::Unread(failure)) => {
            Err(PhysicalRecoverySuccessorCandidateDenial::Discovery {
                artifact,
                generation: artifact_generation(artifact),
                failure,
            })
        }
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
            .map_err(|failure| memory_failure(artifact, failure))?;
        artifacts.push(observed(artifact, bytes, allowance)?);
        Ok(true)
    } else {
        let retained = PlanningResidentAllowance::vector_bytes(&bytes)
            .map_err(|failure| memory_failure(artifact, failure))?;
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
        .map_err(|failure| memory_failure(artifact, failure))?;
    Ok(RecoveryObservedCandidateArtifact { artifact, bytes })
}
