use worth_store::physical_runtime::{
    AbsentCheckpointWitness, ObservedRecoveryArtifact, PhysicalRecoveryCoordination,
    SelectedCheckpointInstallationDenial, SharedRecoveryCheckpoint,
    StoreRecoveryCheckpointBindingBasis,
};
use worth_store_recovery_physics::{PhysicalCheckpointBase, SelectedPhysicalRoot};

use crate::entry::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDeclaration,
    PhysicalRecoveryRootProtocolArtifact, PhysicalRecoverySourceDenial,
    PhysicalRecoverySourceReadAllocationBoundary as AllocationBoundary,
    PhysicalRecoverySourceReadAllocationDenial as AllocationCause,
};
use crate::integrity_ingress::{
    admit_observed_root_manifest, IntegrityAdmittedRecoveryArtifact, OwnerCheckpointProjection,
    RecoveryIntegrityIngressTrace,
};
use crate::orchestration::{source_memory_limit, CheckpointDiscovery};
use crate::progression::PhysicalRecoveryDiscoveryCounters;

use super::failure::SelectionFailure;

pub(in crate::progression::discovered) enum CheckpointInstallation {
    Absent(ObservedRecoveryArtifact),
    Selected {
        base: PhysicalCheckpointBase,
        shared: SharedRecoveryCheckpoint,
        binding: StoreRecoveryCheckpointBindingBasis,
    },
}

impl CheckpointInstallation {
    /// Installs into the owner; an absence also yields its witness.
    pub(in crate::progression::discovered) fn install(
        self,
        coordination: &mut PhysicalRecoveryCoordination,
    ) -> Result<Option<AbsentCheckpointWitness>, SelectedCheckpointInstallationDenial> {
        match self {
            Self::Absent(observed) => coordination.install_absent_checkpoint(observed).map(Some),
            Self::Selected {
                base,
                shared,
                binding,
            } => coordination
                .install_selected_checkpoint(base, shared, binding)
                .map(|()| None),
        }
    }
}

pub(super) fn select_checkpoint(
    root: &SelectedPhysicalRoot,
    checkpoint: CheckpointDiscovery,
    counters: PhysicalRecoveryDiscoveryCounters,
    limits: &PhysicalRecoveryLimitDeclaration,
    trace: &mut RecoveryIntegrityIngressTrace,
    coordination: &mut PhysicalRecoveryCoordination,
) -> Result<(Option<PhysicalCheckpointBase>, CheckpointInstallation), SelectionFailure> {
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
        CheckpointDiscovery::Absent(observed) => {
            Ok((None, CheckpointInstallation::Absent(observed)))
        }
        CheckpointDiscovery::Rejected { denial, limit } => {
            let blocked = failure(PhysicalRecoverySourceDenial::CheckpointIntegrity(denial));
            Err(match limit {
                Some(limit) => blocked.with_limit(limit),
                None => blocked,
            })
        }
        CheckpointDiscovery::Admitted {
            projection,
            source_root,
        } => {
            let OwnerCheckpointProjection {
                checkpoint,
                binding_basis,
            } = projection;
            let generation = checkpoint.facts().source().root().generation();
            let root_failure =
                |rejection: crate::integrity_ingress::RecoveryIntegrityIngressRejection| {
                    failure(PhysicalRecoverySourceDenial::RootProtocol {
                        artifact: PhysicalRecoveryRootProtocolArtifact::CheckpointSourceRoot {
                            generation,
                        },
                        denial: rejection.diagnostic(),
                    })
                };
            let artifact =
                PhysicalRecoveryRootProtocolArtifact::CheckpointSourceRoot { generation };
            // A refused memory grant is recovery memory's limit.
            let allocation_failure = |boundary, requested, cause: AllocationCause| {
                let limit = source_memory_limit(limits, &cause);
                let blocked = failure(PhysicalRecoverySourceDenial::SourceReadAllocation {
                    artifact,
                    boundary,
                    requested,
                    cause,
                });
                match limit {
                    Some(limit) => blocked.with_limit(limit),
                    None => blocked,
                }
            };
            let mut allocation = coordination
                .begin_source_read_allocation()
                .map_err(|cause| {
                    allocation_failure(
                        AllocationBoundary::WindowAdmission,
                        0,
                        AllocationCause::Admission(cause),
                    )
                })?;
            let scratch = worth_store_physical_format::DurablePhysicalRootManifest::maximum_encoding_scratch_bytes() as u64;
            if source_root.observed().bytes().is_some() {
                allocation.reserve_total(scratch).map_err(|cause| {
                    allocation_failure(
                        AllocationBoundary::CanonicalValidation,
                        scratch,
                        AllocationCause::Residency(cause),
                    )
                })?;
            }
            let attempt = admit_observed_root_manifest(
                source_root.observed(),
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
                .bind_checkpoint_base(root, checkpoint.stream(), trace.counters_mut())
                .map(|base| {
                    (
                        Some(base.clone()),
                        CheckpointInstallation::Selected {
                            base,
                            shared: checkpoint,
                            binding: binding_basis,
                        },
                    )
                })
                .map_err(|denial| failure(PhysicalRecoverySourceDenial::CheckpointBinding(denial)))
        }
    }
}
