use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;

use super::super::{IntegrityAdmittedRecoveryArtifact, RecoveryIntegrityIngressCounters};

// Compile-time exhaustiveness contract, deliberately not counted as a runtime
// test: adding an admitted family requires its projection to be classified.
#[allow(dead_code)]
fn routing_is_exhaustive_over_every_current_recovery_family() {
    fn family(artifact: &IntegrityAdmittedRecoveryArtifact<'_>) -> PhysicalIntegrityArtifactFamily {
        match artifact {
            IntegrityAdmittedRecoveryArtifact::BootstrapCatalog(value) => {
                value.scope().artifact_family()
            }
            IntegrityAdmittedRecoveryArtifact::CurrentSelector(value) => {
                value.scope().artifact_family()
            }
            IntegrityAdmittedRecoveryArtifact::PreviousSelector(value) => {
                value.scope().artifact_family()
            }
            IntegrityAdmittedRecoveryArtifact::RootManifest(value) => {
                value.scope().artifact_family()
            }
            IntegrityAdmittedRecoveryArtifact::RootRoutingBlock(value) => {
                value.scope().artifact_family()
            }
            IntegrityAdmittedRecoveryArtifact::SegmentMembershipBlock(value) => {
                value.scope().artifact_family()
            }
            IntegrityAdmittedRecoveryArtifact::PageFrame(value) => value.scope().artifact_family(),
            IntegrityAdmittedRecoveryArtifact::ExtentManifest(value) => {
                value.scope().artifact_family()
            }
            IntegrityAdmittedRecoveryArtifact::ExtentChunk(value) => {
                value.scope().artifact_family()
            }
            IntegrityAdmittedRecoveryArtifact::WalFrame(value) => value.scope().artifact_family(),
            IntegrityAdmittedRecoveryArtifact::CheckpointStreamHeader(value) => {
                value.scope().artifact_family()
            }
            IntegrityAdmittedRecoveryArtifact::CheckpointDirtyBasis(value) => {
                value.scope().artifact_family()
            }
            IntegrityAdmittedRecoveryArtifact::CheckpointBindingCompaction(value) => {
                value.scope().artifact_family()
            }
            IntegrityAdmittedRecoveryArtifact::CheckpointBinding(value) => {
                value.scope().artifact_family()
            }
            IntegrityAdmittedRecoveryArtifact::CheckpointFooter(value) => {
                value.scope().artifact_family()
            }
            IntegrityAdmittedRecoveryArtifact::FreeSpaceHeader(value) => {
                value.scope().artifact_family()
            }
            IntegrityAdmittedRecoveryArtifact::FreeSpaceMembershipBlock(value) => {
                value.scope().artifact_family()
            }
        }
    }
    let _ = family;

    fn project(
        artifact: IntegrityAdmittedRecoveryArtifact<'_>,
        counters: &mut RecoveryIntegrityIngressCounters,
    ) {
        match artifact {
            IntegrityAdmittedRecoveryArtifact::BootstrapCatalog(value) => {
                let _ = value.project(counters);
            }
            IntegrityAdmittedRecoveryArtifact::CurrentSelector(value) => {
                let _ = value.project_for_recovery(counters);
            }
            IntegrityAdmittedRecoveryArtifact::PreviousSelector(value) => {
                let _ = value.project_for_recovery(counters);
            }
            IntegrityAdmittedRecoveryArtifact::RootManifest(value) => {
                let _ = value.project_for_recovery(counters);
            }
            IntegrityAdmittedRecoveryArtifact::RootRoutingBlock(value) => {
                let _ = value.project(counters);
            }
            IntegrityAdmittedRecoveryArtifact::SegmentMembershipBlock(value) => {
                let _ = value.project(counters);
            }
            IntegrityAdmittedRecoveryArtifact::PageFrame(value) => {
                let projection = value.project(counters);
                let _ = (projection.page_lsn, projection.encoded_digest);
            }
            IntegrityAdmittedRecoveryArtifact::ExtentManifest(value) => {
                let _ = value.project(counters);
            }
            IntegrityAdmittedRecoveryArtifact::ExtentChunk(value) => {
                let projection = value.project(counters);
                let _ = (projection.page_lsn, projection.encoded_digest);
            }
            IntegrityAdmittedRecoveryArtifact::WalFrame(value) => {
                let _ = value.into_owner_redo_projection(counters);
            }
            IntegrityAdmittedRecoveryArtifact::CheckpointStreamHeader(value) => {
                let _ = value.scope();
            }
            IntegrityAdmittedRecoveryArtifact::CheckpointDirtyBasis(value) => {
                let _ = value.scope();
            }
            IntegrityAdmittedRecoveryArtifact::CheckpointBindingCompaction(value) => {
                let _ = value.scope();
            }
            IntegrityAdmittedRecoveryArtifact::CheckpointBinding(value) => {
                let _ = value.scope();
            }
            IntegrityAdmittedRecoveryArtifact::CheckpointFooter(value) => {
                let _ = value.scope();
            }
            IntegrityAdmittedRecoveryArtifact::FreeSpaceHeader(value) => {
                let _ = value.project(counters);
            }
            IntegrityAdmittedRecoveryArtifact::FreeSpaceMembershipBlock(value) => {
                let _ = value.project(counters);
            }
        }
    }
    let _ = project;
}
