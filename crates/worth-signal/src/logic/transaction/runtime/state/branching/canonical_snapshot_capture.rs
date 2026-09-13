use crate::branch::{
    AdmittedSignalBranchBasis, AdmittedSignalBranchSnapshot, SignalBranchAdmissionLease,
    SignalBranchSnapshotCaptureDenial, SignalBranchSnapshotCaptureOutcome,
};
use crate::state::SignalBranchHandle;

use super::super::runtime_state::SignalRuntime;
use super::branches::SignalBranchSnapshotStorageDenial;

struct SignalBranchSnapshotCapturePreflight {
    branch: SignalBranchHandle,
    basis_retention: SignalBranchAdmissionLease,
    snapshot_retention: SignalBranchAdmissionLease,
}

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn capture_signal_branch_snapshot(
        &mut self,
        expected: &AdmittedSignalBranchBasis,
    ) -> Result<SignalBranchSnapshotCaptureOutcome, SignalBranchSnapshotCaptureDenial> {
        if let Some((_, mutation, _)) = self.sealed_owner_port_slots() {
            let cancellation = crate::branch::SignalOwnerCancellationSource::new();
            return mutation.capture_exact(expected, &cancellation.token());
        }
        let preflight = self.preflight_signal_branch_snapshot_capture(expected)?;
        let snapshot = self
            .capture_branch_snapshot(preflight.branch.clone())
            .map_err(|error| SignalBranchSnapshotCaptureDenial::OwnerDeniedNoMovement { error })?;
        let captured_basis = self
            .admit_signal_branch_with_retention(preflight.branch, preflight.basis_retention)
            .expect("performed snapshot capture must retain its canonical branch basis");
        let snapshot = AdmittedSignalBranchSnapshot::owner_issued(
            self.branches.owner_runtime_instance_id(),
            snapshot,
            preflight.snapshot_retention,
        );
        Ok(SignalBranchSnapshotCaptureOutcome::owner_issued(
            snapshot,
            captured_basis,
        ))
    }

    fn preflight_signal_branch_snapshot_capture(
        &mut self,
        expected: &AdmittedSignalBranchBasis,
    ) -> Result<SignalBranchSnapshotCapturePreflight, SignalBranchSnapshotCaptureDenial> {
        let branch_id = expected.owner_branch_id();
        let branch = self
            .branches
            .branch_handle(branch_id)
            .ok_or(SignalBranchSnapshotCaptureDenial::UnknownBranch { branch_id })?;
        let live = self
            .signal_branch_observation(&branch)
            .map_err(|_| SignalBranchSnapshotCaptureDenial::UnknownBranch { branch_id })?;
        if let Err(mismatch) = live.compare(expected.observation()) {
            return Err(SignalBranchSnapshotCaptureDenial::BasisMismatch {
                axes: mismatch.axes().to_vec(),
            });
        }
        self.branches
            .ensure_snapshot_storage_available()
            .map_err(|denial| match denial {
                SignalBranchSnapshotStorageDenial::CapacityExhausted {
                    maximum_stored_snapshots,
                } => SignalBranchSnapshotCaptureDenial::SnapshotCapacityExhausted {
                    maximum_stored_snapshots,
                },
            })?;
        let target_graph = self
            .branches
            .replay_graph(branch_id, self.graph.current_branch().id, &self.graph)
            .ok_or(SignalBranchSnapshotCaptureDenial::UnknownBranch { branch_id })?;
        let (next_snapshot_id, _) = target_graph
            .diagnostics_state()
            .branch_snapshot_allocator_state();
        self.branches
            .synchronize_snapshot_identity_high_water(next_snapshot_id);
        self.branches
            .snapshot_identity_available()
            .map_err(|next_snapshot_id| {
                SignalBranchSnapshotCaptureDenial::SnapshotIdentityExhausted { next_snapshot_id }
            })?;
        let basis_retention = self
            .branches
            .acquire_admitted_retention(branch_id)
            .map_err(|denial| SignalBranchSnapshotCaptureDenial::RetentionUnavailable { denial })?;
        let snapshot_retention = self
            .branches
            .acquire_admitted_retention(branch_id)
            .map_err(|denial| SignalBranchSnapshotCaptureDenial::RetentionUnavailable { denial })?;
        Ok(SignalBranchSnapshotCapturePreflight {
            branch,
            basis_retention,
            snapshot_retention,
        })
    }
}
