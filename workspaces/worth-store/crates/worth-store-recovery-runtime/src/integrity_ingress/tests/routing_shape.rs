use super::super::{IntegrityAdmittedRecoveryArtifact, RecoveryIntegrityIngressCounters};

// Compile-time exhaustiveness contract, deliberately not counted as a runtime
// test: adding an admitted family requires its projection to be classified.
const _: fn(IntegrityAdmittedRecoveryArtifact<'static>, &mut RecoveryIntegrityIngressCounters) =
    project_every_current_recovery_family;

fn project_every_current_recovery_family(
    artifact: IntegrityAdmittedRecoveryArtifact<'_>,
    counters: &mut RecoveryIntegrityIngressCounters,
) {
    match artifact {
        IntegrityAdmittedRecoveryArtifact::BootstrapCatalog(value) => {
            let _ = value.project(counters);
        }
        IntegrityAdmittedRecoveryArtifact::RootManifest(value) => {
            let _ = value.project();
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
        IntegrityAdmittedRecoveryArtifact::CheckpointStreamHeader(_)
        | IntegrityAdmittedRecoveryArtifact::CheckpointDirtyBasis(_)
        | IntegrityAdmittedRecoveryArtifact::CheckpointBindingCompaction(_)
        | IntegrityAdmittedRecoveryArtifact::CheckpointBinding(_)
        | IntegrityAdmittedRecoveryArtifact::CheckpointFooter(_) => {
            // Checkpoint members project only through the verified stream.
        }
        IntegrityAdmittedRecoveryArtifact::FreeSpaceHeader(value) => {
            let _ = value.project(counters);
        }
        IntegrityAdmittedRecoveryArtifact::FreeSpaceMembershipBlock(value) => {
            let _ = value.project(counters);
        }
    }
}
