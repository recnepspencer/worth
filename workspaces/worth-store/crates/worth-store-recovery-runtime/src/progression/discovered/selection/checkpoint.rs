use worth_store::physical_runtime::StoreRecoveryCheckpointBindingBasis;
use worth_store_recovery_physics::{PhysicalCheckpointBase, SelectedPhysicalRoot};

use crate::entry::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryRootProtocolArtifact, PhysicalRecoverySourceDenial,
};
use crate::integrity_ingress::{
    admit_observed_root_manifest, IntegrityAdmittedRecoveryArtifact, OwnerCheckpointProjection,
    RecoveryIntegrityIngressTrace,
};
use crate::orchestration::CheckpointDiscovery;
use crate::progression::PhysicalRecoveryDiscoveryCounters;

use super::failure::SelectionFailure;

pub(super) fn select_checkpoint(
    root: &SelectedPhysicalRoot,
    checkpoint: CheckpointDiscovery,
    counters: PhysicalRecoveryDiscoveryCounters,
    trace: &mut RecoveryIntegrityIngressTrace,
) -> Result<
    (
        Option<PhysicalCheckpointBase>,
        Option<StoreRecoveryCheckpointBindingBasis>,
    ),
    SelectionFailure,
> {
    let failure = |denial| {
        SelectionFailure::new(
            PhysicalRecoveryBlockKind::Checkpoint,
            counters,
            "families/checkpoint.current",
        )
        .with_generation(root.selected().selector().root_generation())
        .with_source_denials(vec![denial])
    };
    match checkpoint {
        CheckpointDiscovery::Absent => Ok((None, None)),
        CheckpointDiscovery::Rejected(denial) => Err(failure(
            PhysicalRecoverySourceDenial::CheckpointIntegrity(denial),
        )),
        CheckpointDiscovery::Admitted {
            projection,
            source_root,
        } => {
            let OwnerCheckpointProjection {
                checkpoint,
                binding_basis,
            } = projection;
            let generation = checkpoint.source().root().generation();
            let root_failure =
                |rejection: crate::integrity_ingress::RecoveryIntegrityIngressRejection| {
                    failure(PhysicalRecoverySourceDenial::RootProtocol {
                        artifact: PhysicalRecoveryRootProtocolArtifact::CheckpointSourceRoot {
                            generation,
                        },
                        denial: rejection.diagnostic(),
                    })
                };
            let attempt = admit_observed_root_manifest(
                &source_root,
                root.selected().selector().store_identity(),
                root.selected().selector().format(),
                generation,
                trace.counters_mut(),
            )
            .map_err(root_failure)?;
            trace.retain(attempt.observation());
            let admitted = match attempt.into_outcome() {
                Ok(IntegrityAdmittedRecoveryArtifact::RootManifest(admitted)) => admitted,
                Ok(_) => unreachable!("checkpoint source observes only the exact root manifest"),
                Err(rejection) => return Err(root_failure(rejection)),
            };
            admitted
                .bind_checkpoint_base(root, checkpoint, trace.counters_mut())
                .map(|checkpoint| (Some(checkpoint), Some(binding_basis)))
                .map_err(|denial| failure(PhysicalRecoverySourceDenial::CheckpointBinding(denial)))
        }
    }
}
