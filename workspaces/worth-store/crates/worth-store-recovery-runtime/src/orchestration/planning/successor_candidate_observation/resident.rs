//! The successor observation shares the planning resident window. These
//! helpers preserve the artifact that caused a memory or allocator denial.

use worth_store_physical_format::RecordArtifactFile;

use super::artifact_generation;
use crate::entry::PhysicalRecoverySuccessorCandidateDenial;
use crate::integrity_ingress::RecoveryIntegrityIngressTrace;
use crate::progression::{PlanningMemoryDenial, PlanningResidentAllowance};

/// The candidate's denial. A refused charge leaves its need with the window,
/// which holds the counts for the block's cause; a count that overflowed
/// leaves none.
pub(super) fn memory_failure(
    artifact: RecordArtifactFile,
    failure: PlanningMemoryDenial,
) -> PhysicalRecoverySuccessorCandidateDenial {
    let generation = artifact_generation(artifact);
    match failure {
        PlanningMemoryDenial::RecoveryMemoryBytes { .. } => {
            PhysicalRecoverySuccessorCandidateDenial::RecoveryMemoryBytes {
                artifact,
                generation,
            }
        }
        PlanningMemoryDenial::Allocation {
            requested_bytes,
            cause,
        } => PhysicalRecoverySuccessorCandidateDenial::Allocation {
            artifact,
            generation,
            requested_bytes,
            cause,
        },
    }
}

/// Admit observation-trace growth while its old backing is still live. The
/// ingress path may append exactly one observation after this returns.
pub(super) fn trace_slots(
    artifact: RecordArtifactFile,
    trace: &mut RecoveryIntegrityIngressTrace,
    allowance: &mut PlanningResidentAllowance,
) -> Result<(), PhysicalRecoverySuccessorCandidateDenial> {
    let requested = trace.observation_reservation_bytes(1).ok_or_else(|| {
        memory_failure(
            artifact,
            PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX },
        )
    })?;
    if requested == 0 {
        return Ok(());
    }
    allowance
        .transient(requested)
        .map_err(|failure| memory_failure(artifact, failure))?;
    let old = trace.owned_heap_bytes().ok_or_else(|| {
        memory_failure(
            artifact,
            PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX },
        )
    })?;
    let target = trace.next_observation_capacity(1).ok_or_else(|| {
        memory_failure(
            artifact,
            PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX },
        )
    })?;
    trace
        .try_reserve_observation_capacity(target)
        .map_err(|cause| {
            memory_failure(
                artifact,
                PlanningMemoryDenial::Allocation {
                    requested_bytes: requested,
                    cause,
                },
            )
        })?;
    let actual = trace.owned_heap_bytes().ok_or_else(|| {
        memory_failure(
            artifact,
            PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX },
        )
    })?;
    let growth = actual.checked_sub(old).ok_or_else(|| {
        memory_failure(
            artifact,
            PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX },
        )
    })?;
    allowance
        .retain(growth)
        .map_err(|failure| memory_failure(artifact, failure))
}
