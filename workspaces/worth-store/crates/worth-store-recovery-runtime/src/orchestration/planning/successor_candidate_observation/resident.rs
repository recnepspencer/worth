//! The successor observation shares the planning resident window. These
//! helpers preserve the artifact that caused a memory or allocator denial.

use worth_store_physical_format::RecordArtifactFile;

use super::artifact_generation;
use crate::entry::PhysicalRecoverySuccessorCandidateDenial;
use crate::integrity_ingress::RecoveryIntegrityIngressTrace;
use crate::progression::{PlanningMemoryDenial, PlanningResidentAllowance};

pub(super) fn memory_failure(
    artifact: RecordArtifactFile,
    allowance: &PlanningResidentAllowance,
    failure: PlanningMemoryDenial,
) -> PhysicalRecoverySuccessorCandidateDenial {
    let generation = artifact_generation(artifact);
    match failure {
        PlanningMemoryDenial::RecoveryMemoryBytes { observed } => {
            PhysicalRecoverySuccessorCandidateDenial::RecoveryMemoryBytes {
                artifact,
                generation,
                observed,
                admitted: allowance.maximum(),
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
            allowance,
            PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX },
        )
    })?;
    if requested == 0 {
        return Ok(());
    }
    allowance
        .transient(requested)
        .map_err(|failure| memory_failure(artifact, allowance, failure))?;
    let old = trace.owned_heap_bytes().ok_or_else(|| {
        memory_failure(
            artifact,
            allowance,
            PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX },
        )
    })?;
    let target = trace.next_observation_capacity(1).ok_or_else(|| {
        memory_failure(
            artifact,
            allowance,
            PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX },
        )
    })?;
    trace
        .try_reserve_observation_capacity(target)
        .map_err(|cause| {
            memory_failure(
                artifact,
                allowance,
                PlanningMemoryDenial::Allocation {
                    requested_bytes: requested,
                    cause,
                },
            )
        })?;
    let actual = trace.owned_heap_bytes().ok_or_else(|| {
        memory_failure(
            artifact,
            allowance,
            PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX },
        )
    })?;
    let growth = actual.checked_sub(old).ok_or_else(|| {
        memory_failure(
            artifact,
            allowance,
            PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX },
        )
    })?;
    allowance
        .retain(growth)
        .map_err(|failure| memory_failure(artifact, allowance, failure))
}
