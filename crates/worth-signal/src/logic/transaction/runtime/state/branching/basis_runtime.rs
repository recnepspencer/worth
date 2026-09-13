use worth_proof::TransitionOutcome;

use worth_foundational::FoundationalBranchReferenceGeneration;

use crate::branch::{
    AdmittedSignalBranchBasis, SignalBranchAdmissionLease, SignalBranchObservation,
};
use crate::data::error::SignalError;
use crate::state::{SignalBranchHandle, SignalSnapshotV1, SnapshotRestoreIntent};

use super::super::runtime_state::SignalRuntime;
use super::basis::{
    materialize_branch_basis, SignalBranchBasisArtifact, SignalBranchBasisDenial,
    SignalBranchBasisIdentity, SignalBranchBasisValidationOutcome,
};
use super::basis_definition::signal_definition_basis;

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn current_branch_basis_artifact(&mut self) -> SignalBranchBasisArtifact {
        self.with_telemetry(|telemetry| telemetry.transaction.branch_basis_production_count += 1);
        let branch = self.graph.current_branch();
        materialize_branch_basis(
            branch.name.clone(),
            SignalBranchBasisIdentity::from_branch_handle(&branch),
        )
    }

    pub(crate) fn branch_basis_artifact(
        &mut self,
        branch: SignalBranchHandle,
    ) -> TransitionOutcome<SignalBranchBasisArtifact, SignalBranchBasisDenial> {
        self.with_telemetry(|telemetry| telemetry.transaction.branch_basis_production_count += 1);
        let live_branch =
            self.branches
                .branch_handle(branch.id)
                .ok_or(SignalBranchBasisDenial::UnknownBranch {
                    branch_id: branch.id,
                    branch_name: branch.name,
                });
        match live_branch {
            Ok(branch) => TransitionOutcome::success(materialize_branch_basis(
                branch.name.clone(),
                SignalBranchBasisIdentity::from_branch_handle(&branch),
            )),
            Err(denial) => {
                self.with_telemetry(|telemetry| {
                    telemetry.transaction.branch_basis_denial_count += 1
                });
                TransitionOutcome::denied(denial)
            }
        }
    }

    pub(crate) fn snapshot_restore_branch_basis_artifact(
        &mut self,
        snapshot: &SignalSnapshotV1,
        intent: SnapshotRestoreIntent,
    ) -> TransitionOutcome<SignalBranchBasisArtifact, SignalBranchBasisDenial> {
        self.with_telemetry(|telemetry| telemetry.transaction.branch_basis_production_count += 1);
        let branch_id = snapshot.meta.branch_id;
        let snapshot_id = snapshot.meta.snapshot_id;
        let Some(live_branch) = self.branches.branch_handle(branch_id) else {
            self.with_telemetry(|telemetry| telemetry.transaction.branch_basis_denial_count += 1);
            return TransitionOutcome::denied(SignalBranchBasisDenial::UnknownBranch {
                branch_id,
                branch_name: snapshot.meta.branch_name.clone(),
            });
        };

        if self
            .branches
            .snapshot_state(branch_id, snapshot_id)
            .is_none()
        {
            self.with_telemetry(|telemetry| telemetry.transaction.branch_basis_denial_count += 1);
            return TransitionOutcome::denied(SignalBranchBasisDenial::UntrackedSnapshot {
                branch_id,
                snapshot_id,
            });
        }

        TransitionOutcome::success(materialize_branch_basis(
            live_branch.name.clone(),
            SignalBranchBasisIdentity::from_snapshot_restore(snapshot, intent),
        ))
    }

    pub(crate) fn snapshot_branch_basis_artifact(
        &mut self,
        branch: SignalBranchHandle,
        snapshot: &SignalSnapshotV1,
    ) -> TransitionOutcome<SignalBranchBasisArtifact, SignalBranchBasisDenial> {
        self.with_telemetry(|telemetry| telemetry.transaction.branch_basis_production_count += 1);
        let Some(live_branch) = self.branches.branch_handle(branch.id) else {
            self.with_telemetry(|telemetry| telemetry.transaction.branch_basis_denial_count += 1);
            return TransitionOutcome::denied(SignalBranchBasisDenial::UnknownBranch {
                branch_id: branch.id,
                branch_name: branch.name,
            });
        };

        if snapshot.meta.branch_id != live_branch.id {
            self.with_telemetry(|telemetry| telemetry.transaction.branch_basis_denial_count += 1);
            return TransitionOutcome::denied(SignalBranchBasisDenial::CrossBranchMismatch {
                basis_branch_id: snapshot.meta.branch_id,
                expected_branch_id: live_branch.id,
            });
        }

        if self
            .branches
            .snapshot_state(live_branch.id, snapshot.meta.snapshot_id)
            .is_none()
        {
            self.with_telemetry(|telemetry| telemetry.transaction.branch_basis_denial_count += 1);
            return TransitionOutcome::denied(SignalBranchBasisDenial::UntrackedSnapshot {
                branch_id: live_branch.id,
                snapshot_id: snapshot.meta.snapshot_id,
            });
        }

        TransitionOutcome::success(materialize_branch_basis(
            live_branch.name.clone(),
            SignalBranchBasisIdentity::from_branch_snapshot(
                &live_branch,
                snapshot.meta.snapshot_id,
            ),
        ))
    }

    pub(crate) fn validate_branch_basis_artifact(
        &mut self,
        basis: SignalBranchBasisArtifact,
        branch: SignalBranchHandle,
    ) -> SignalBranchBasisValidationOutcome {
        self.with_telemetry(|telemetry| telemetry.transaction.branch_basis_validation_count += 1);
        if basis.payload().branch_id() != branch.id {
            self.with_telemetry(|telemetry| telemetry.transaction.branch_basis_denial_count += 1);
            return TransitionOutcome::denied(SignalBranchBasisDenial::CrossBranchMismatch {
                basis_branch_id: basis.payload().branch_id(),
                expected_branch_id: branch.id,
            });
        }

        let Some(live_branch) = self.branches.branch_handle(branch.id) else {
            self.with_telemetry(|telemetry| telemetry.transaction.branch_basis_denial_count += 1);
            return TransitionOutcome::denied(SignalBranchBasisDenial::UnknownBranch {
                branch_id: branch.id,
                branch_name: branch.name,
            });
        };

        let live_identity = SignalBranchBasisIdentity::from_branch_handle(&live_branch);
        if basis.strong_basis().value() != &live_identity {
            self.with_telemetry(|telemetry| telemetry.transaction.branch_basis_stale_count += 1);
            return TransitionOutcome::stale(basis.downgrade_to_stale_readable());
        }

        TransitionOutcome::success(basis)
    }

    /// Observe one live Signal branch through the shared Foundational
    /// reference grammar and immediately attach the owner-issued basis proof.
    /// The old numeric head remains an internal engine detail; callers receive
    /// only the exact immutable observation that can be compared or carried.
    pub fn observe_signal_branch_basis(
        &self,
        branch: SignalBranchHandle,
    ) -> Result<AdmittedSignalBranchBasis, crate::branch::SignalBranchBasisObservationDenial> {
        if self.owner_services.is_sealed() {
            return self.owner_services.observe_legacy_branch(branch);
        }
        let live_branch = self.branches.branch_handle(branch.id).ok_or(
            crate::branch::SignalBranchBasisObservationDenial::UnknownBranch {
                branch_id: branch.id,
            },
        )?;
        let observation = self
            .signal_branch_observation(&live_branch)
            .map_err(|error| {
                crate::branch::SignalBranchBasisObservationDenial::InvalidOwnerObservation { error }
            })?;
        self.admit_unsealed_canonical_basis_with_retention(observation, live_branch.id, || {
            self.branches.acquire_admitted_retention(live_branch.id)
        })
        .map_err(|denial| {
            crate::branch::SignalBranchBasisObservationDenial::RetentionUnavailable { denial }
        })
    }

    pub(super) fn admit_signal_branch_with_retention(
        &self,
        branch: SignalBranchHandle,
        retention: SignalBranchAdmissionLease,
    ) -> Result<AdmittedSignalBranchBasis, SignalError> {
        let live_branch = self
            .branches
            .branch_handle(branch.id)
            .ok_or_else(|| SignalError::unknown_branch(Some(branch.id), branch.name))?;
        let observation = self.signal_branch_observation(&live_branch)?;
        Ok(self.admit_unsealed_canonical_basis(observation, live_branch.id, retention))
    }

    pub(super) fn admit_unsealed_canonical_basis(
        &self,
        observation: SignalBranchObservation,
        branch_id: crate::state::SignalBranchId,
        retention: SignalBranchAdmissionLease,
    ) -> AdmittedSignalBranchBasis {
        self.basis_registry.admit(
            self.branches.owner_runtime_instance_id(),
            signal_definition_basis(self),
            branch_id,
            // The unsealed runtime has one manager-cell lifetime. Its actual
            // head generation already participates in the exact observation;
            // the sealed owner supplies the stronger cell incarnation axis.
            0,
            observation,
            retention,
        )
    }

    pub(super) fn admit_unsealed_canonical_basis_with_retention<Acquire>(
        &self,
        observation: SignalBranchObservation,
        branch_id: crate::state::SignalBranchId,
        acquire_retention: Acquire,
    ) -> Result<AdmittedSignalBranchBasis, crate::branch::SignalBranchRetentionAcquisitionDenial>
    where
        Acquire: FnOnce() -> Result<
            SignalBranchAdmissionLease,
            crate::branch::SignalBranchRetentionAcquisitionDenial,
        >,
    {
        self.basis_registry.admit_with_retention(
            self.branches.owner_runtime_instance_id(),
            signal_definition_basis(self),
            branch_id,
            0,
            observation,
            |_| self.validate_unsealed_canonical_basis_reuse(branch_id),
            acquire_retention,
        )
    }

    fn validate_unsealed_canonical_basis_reuse(
        &self,
        branch_id: crate::state::SignalBranchId,
    ) -> Result<(), crate::branch::SignalBranchRetentionAcquisitionDenial> {
        if self.branches.branch_handle(branch_id).is_some() {
            return Ok(());
        }
        if self.branches.branch_retirement_receipt(branch_id).is_some() {
            Err(crate::branch::SignalBranchRetentionAcquisitionDenial::RetiredBranch { branch_id })
        } else {
            Err(crate::branch::SignalBranchRetentionAcquisitionDenial::UnknownBranch { branch_id })
        }
    }

    pub(super) fn signal_branch_observation(
        &self,
        live_branch: &SignalBranchHandle,
    ) -> Result<SignalBranchObservation, SignalError> {
        let identity = SignalBranchBasisIdentity::from_branch_handle_with_restore(
            live_branch,
            self.branches.branch_restore_snapshot_id(live_branch.id),
        );
        identity
            .to_foundational_observation(
                self.branches.owner_runtime_instance_id().to_string(),
                live_branch.name.clone(),
                signal_definition_basis(self),
                FoundationalBranchReferenceGeneration::new(
                    self.branches.branch_head_generation(live_branch.id),
                ),
            )
            .map_err(|denial| {
                SignalError::invalid_input(format!(
                    "Signal branch observation construction denied: {denial:?}"
                ))
            })
    }
}
