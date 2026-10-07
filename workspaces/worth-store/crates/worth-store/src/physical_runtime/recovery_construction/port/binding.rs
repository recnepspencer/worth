//! Exact construction authority and selected-root occurrence binding.

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_format::RecordArtifactFile;

use super::super::RecoveredPhysicalRuntimeConstructionDenial;
use crate::physical_runtime::{
    CompletedPhysicalRecoveryFreshReopen, PhysicalRecoveryAllocationAdmission,
    PhysicalRecoveryCoordination, RuntimeIdentity,
};

pub(super) fn validate_construction_binding(
    coordination: &PhysicalRecoveryCoordination,
    media: &AdmittedRecoveryFilesystemMedia,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
) -> Result<PhysicalRecoveryAllocationAdmission, RecoveredPhysicalRuntimeConstructionDenial> {
    if !coordination.construction_authority().matches(
        media.store_identity(),
        media.media_generation(),
        coordination.session_identity(),
    ) {
        return Err(RecoveredPhysicalRuntimeConstructionDenial::ConstructionAuthorityMismatch);
    }
    let allocation = coordination.recovery_allocation_admission().ok_or(
        RecoveredPhysicalRuntimeConstructionDenial::ResidentAdmission(
            crate::physical_runtime::PhysicalRecoveryRejoinResidentAdmissionDenial::MissingAllocation,
        ),
    )?;
    if allocation.store_identity() != media.store_identity() {
        return Err(
            RecoveredPhysicalRuntimeConstructionDenial::AllocationStoreMismatch {
                allocation: allocation.store_identity(),
                selected: media.store_identity(),
            },
        );
    }
    let occurrence = reopen.fresh_reopen_occurrence();
    let expected_root = RecordArtifactFile::RootManifest {
        generation: reopen.root().generation(),
    };
    if occurrence.session() != coordination.session_identity()
        || occurrence.generation() != reopen.root().generation()
        || occurrence.selector().artifact() != RecordArtifactFile::CurrentRootSelector
        || occurrence.root().artifact() != expected_root
        || occurrence.selector().bytes().is_empty()
        || occurrence.root().bytes().is_empty()
    {
        return Err(RecoveredPhysicalRuntimeConstructionDenial::BindingMismatch);
    }
    Ok(allocation)
}

pub(super) fn fresh_runtime_identity(previous: RuntimeIdentity) -> Option<RuntimeIdentity> {
    loop {
        let candidate = RuntimeIdentity::generate()?;
        if candidate != previous {
            return Some(candidate);
        }
    }
}
