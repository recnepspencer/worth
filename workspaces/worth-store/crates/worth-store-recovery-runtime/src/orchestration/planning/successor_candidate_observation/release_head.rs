//! Collect and verify WAL-authorized `ReleaseCustodyHeadBlock` frames for an
//! observed successor candidate so both incremental and recovery-rebuild
//! validation see the exact durable head frames.
use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};
use worth_store_recovery_physics::VerifiedSelectedReleaseHeadReplayV14;

use super::artifact_read::{read as read_artifact, retain_successor};
use super::materialization::CandidateMaterialization;
use super::resident::memory_failure;
use crate::entry::{
    PhysicalRecoverySuccessorCandidateDenial, PhysicalRecoverySuccessorCandidateMismatch,
};
use crate::orchestration::planning::manifest_entry_budget::{pays_for, ChargeToken};
use crate::progression::{PlanningResidentAllowance, RecoveryObservedCandidateArtifact};

pub(super) fn read(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    charge: &ChargeToken,
    replay: Option<&VerifiedSelectedReleaseHeadReplayV14>,
    artifacts: &mut Vec<RecoveryObservedCandidateArtifact>,
    referenced_artifacts: &mut Vec<RecordArtifactFile>,
    materialization: &mut CandidateMaterialization,
    allowance: &mut PlanningResidentAllowance,
) -> Result<(), PhysicalRecoverySuccessorCandidateDenial> {
    pays_for(charge, root.generation());
    let Some(replay) = replay else {
        return Ok(());
    };
    for write in replay.effect().node_writes() {
        let reference = write.reference();
        let artifact = RecordArtifactFile::ReleaseCustodyHeadBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        allowance
            .grow(referenced_artifacts, 1)
            .map_err(|failure| memory_failure(artifact, failure))?;
        referenced_artifacts.push(artifact);
        materialization.retain_reference();
        let source = read_artifact(discovery, artifact, format, charge, allowance)?;
        let Some(bytes) = source.into_bytes() else {
            return Err(PhysicalRecoverySuccessorCandidateDenial::MissingArtifact {
                artifact,
                generation: super::artifact_generation(artifact),
            });
        };
        if bytes.as_slice() != write.frame() {
            let retained = PlanningResidentAllowance::vector_bytes(&bytes)
                .map_err(|failure| memory_failure(artifact, failure))?;
            drop(bytes);
            allowance.release(retained);
            return Err(PhysicalRecoverySuccessorCandidateDenial::Conflict {
                artifact,
                generation: root.generation(),
                mismatch: PhysicalRecoverySuccessorCandidateMismatch::SuccessorArtifactBytes,
            });
        }
        let retained_bytes = bytes.len();
        if retain_successor(root.generation(), artifact, bytes, artifacts, allowance)? {
            materialization.retain_artifact(retained_bytes);
        }
    }
    Ok(())
}
