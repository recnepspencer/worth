use worth_proof::TransitionOutcome;

use worth_foundational::FoundationalBranchReferenceGeneration;

use crate::branch::{
    admit_runtime_signal_branch_observation, AdmittedSignalBranchBasis, SignalBranchAdmissionLease,
    SignalBranchObservation,
};
use crate::data::error::SignalError;
#[cfg(test)]
use crate::state::SnapshotRestoreIntent;
use crate::state::{SignalBranchHandle, SignalSnapshotV1};

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
    #[cfg(test)]
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

    #[cfg(test)]
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
        let retention = self
            .branches
            .acquire_admitted_retention(live_branch.id)
            .map_err(|denial| {
                crate::branch::SignalBranchBasisObservationDenial::RetentionUnavailable { denial }
            })?;
        Ok(admit_runtime_signal_branch_observation(
            observation,
            live_branch.id,
            retention,
        ))
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
        Ok(admit_runtime_signal_branch_observation(
            observation,
            live_branch.id,
            retention,
        ))
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
