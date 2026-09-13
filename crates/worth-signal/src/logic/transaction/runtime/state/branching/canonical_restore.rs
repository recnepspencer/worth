use crate::branch::{
    AdmittedSignalBranchBasis, AdmittedSignalBranchSnapshot, SignalBranchRestoreDenial,
};

use super::super::runtime_state::SignalRuntime;

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn restore_signal_branch(
        &mut self,
        expected: &AdmittedSignalBranchBasis,
        admitted_snapshot: &AdmittedSignalBranchSnapshot,
    ) -> Result<AdmittedSignalBranchBasis, SignalBranchRestoreDenial> {
        if let Some((_, mutation, _)) = self.sealed_owner_port_slots() {
            let cancellation = crate::branch::SignalOwnerCancellationSource::new();
            return mutation.restore_exact(expected, admitted_snapshot, &cancellation.token());
        }
        let expected_runtime_instance_id = self.branches.owner_runtime_instance_id();
        let observed_runtime_instance_id = admitted_snapshot.owner_runtime_instance_id();
        if observed_runtime_instance_id != expected_runtime_instance_id {
            return Err(SignalBranchRestoreDenial::ForeignSnapshotOwner {
                expected_runtime_instance_id,
                observed_runtime_instance_id,
            });
        }
        let snapshot = admitted_snapshot.snapshot();
        let branch_id = expected.owner_branch_id();
        let branch = self
            .branches
            .branch_handle(branch_id)
            .ok_or(SignalBranchRestoreDenial::UnknownBranch { branch_id })?;
        let live = self
            .signal_branch_observation(&branch)
            .map_err(|_| SignalBranchRestoreDenial::UnknownBranch { branch_id })?;
        if let Err(mismatch) = live.compare(expected.observation()) {
            return Err(SignalBranchRestoreDenial::BasisMismatch {
                axes: mismatch.axes().to_vec(),
            });
        }
        if snapshot.meta.branch_id != branch_id {
            return Err(SignalBranchRestoreDenial::CrossBranchSnapshot {
                branch_id,
                snapshot_branch_id: snapshot.meta.branch_id,
            });
        }
        if self
            .branches
            .snapshot_state(branch_id, snapshot.meta.snapshot_id)
            .is_none()
        {
            return Err(SignalBranchRestoreDenial::UnavailableSnapshot {
                branch_id,
                snapshot_id: snapshot.meta.snapshot_id,
            });
        }
        let retention = self
            .branches
            .acquire_admitted_retention(branch_id)
            .map_err(|denial| SignalBranchRestoreDenial::RetentionUnavailable { denial })?;
        self.restore_branch_snapshot(branch.clone(), snapshot)
            .map_err(|error| SignalBranchRestoreDenial::OwnerDeniedNoMovement { error })?;
        Ok(self
            .admit_signal_branch_with_retention(branch, retention)
            .expect("retained live branch must remain admissible after canonical restore"))
    }
}
