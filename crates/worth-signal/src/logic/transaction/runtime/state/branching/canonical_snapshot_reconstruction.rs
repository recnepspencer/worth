use crate::branch::{
    AdmittedSignalBranchBasis, AdmittedSignalBranchSnapshot, SignalBranchAdmissionLease,
    SignalBranchSnapshotReconstructionDenial, SignalBranchSnapshotReconstructionOutcome,
};
use crate::state::{SignalBranchHandle, SignalSnapshotV1};

use super::super::runtime_state::SignalRuntime;
use super::branches::{SignalBranchSnapshotStorageDenial, SnapshotBranchState};

struct SignalBranchSnapshotReconstructionPreflight {
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
    /// Admit a portable snapshot only while constructing a pristine runtime.
    ///
    /// This is intentionally distinct from ordinary restore: it validates the
    /// current empty owner reference before importing external snapshot state,
    /// then returns an owner-bound snapshot accepted by canonical restore.
    pub fn reconstruct_signal_branch_snapshot(
        &mut self,
        expected: &AdmittedSignalBranchBasis,
        snapshot: &SignalSnapshotV1,
    ) -> Result<SignalBranchSnapshotReconstructionOutcome, SignalBranchSnapshotReconstructionDenial>
    {
        if self.owner_services.is_sealed() {
            return Err(SignalBranchSnapshotReconstructionDenial::NonPristineRuntime);
        }
        let preflight = self.preflight_signal_branch_snapshot_reconstruction(expected, snapshot)?;
        self.restore_branch_snapshot(preflight.branch.clone(), snapshot)
            .map_err(
                |error| SignalBranchSnapshotReconstructionDenial::OwnerDeniedNoMovement { error },
            )?;

        let mut branch_state = self
            .capture_heavy_branch_state()
            .expect("restored pristine branch must remain transferable");
        branch_state
            .mutation_ledger_mut()
            .clear_all(Some(snapshot.meta.snapshot_id));
        self.branches.insert_snapshot(
            SnapshotBranchState::from_branch_state(&branch_state).packet(snapshot.meta.snapshot_id),
        );
        self.branches.synchronize_snapshot_identity_high_water(
            snapshot.meta.snapshot_id.0.saturating_add(1),
        );
        self.branches.observe_active_branch_state(&branch_state);

        let admitted_snapshot = AdmittedSignalBranchSnapshot::owner_issued(
            self.branches.owner_runtime_instance_id(),
            snapshot.clone(),
            preflight.snapshot_retention,
        );
        let reconstructed_basis = self
            .admit_signal_branch_with_retention(preflight.branch, preflight.basis_retention)
            .expect("retained reconstructed branch must remain admissible");
        Ok(SignalBranchSnapshotReconstructionOutcome::owner_issued(
            admitted_snapshot,
            reconstructed_basis,
        ))
    }

    fn preflight_signal_branch_snapshot_reconstruction(
        &self,
        expected: &AdmittedSignalBranchBasis,
        snapshot: &SignalSnapshotV1,
    ) -> Result<SignalBranchSnapshotReconstructionPreflight, SignalBranchSnapshotReconstructionDenial>
    {
        let branch_id = expected.owner_branch_id();
        let branch = self
            .branches
            .branch_handle(branch_id)
            .ok_or(SignalBranchSnapshotReconstructionDenial::UnknownBranch { branch_id })?;
        let live = self
            .signal_branch_observation(&branch)
            .map_err(|_| SignalBranchSnapshotReconstructionDenial::UnknownBranch { branch_id })?;
        if let Err(mismatch) = live.compare(expected.observation()) {
            return Err(SignalBranchSnapshotReconstructionDenial::BasisMismatch {
                axes: mismatch.axes().to_vec(),
            });
        }
        if branch.head_snapshot_id.is_some() || self.branches.branch_head_generation(branch_id) != 0
        {
            return Err(SignalBranchSnapshotReconstructionDenial::NonPristineBranch { branch_id });
        }
        if self.graph.current_branch().id != branch_id {
            return Err(SignalBranchSnapshotReconstructionDenial::InactiveBranch { branch_id });
        }
        if snapshot.meta.branch_id != branch_id {
            return Err(
                SignalBranchSnapshotReconstructionDenial::CrossBranchSnapshot {
                    branch_id,
                    snapshot_branch_id: snapshot.meta.branch_id,
                },
            );
        }
        if !self
            .branches
            .snapshot_reconstruction_runtime_is_pristine(branch_id)
            || self
                .graph
                .diagnostics_state()
                .branch_snapshot_allocator_state()
                .0
                != 0
        {
            return Err(SignalBranchSnapshotReconstructionDenial::NonPristineRuntime);
        }
        self.branches
            .ensure_snapshot_storage_available()
            .map_err(|denial| match denial {
                SignalBranchSnapshotStorageDenial::CapacityExhausted {
                    maximum_stored_snapshots,
                } => SignalBranchSnapshotReconstructionDenial::SnapshotCapacityExhausted {
                    maximum_stored_snapshots,
                },
            })?;
        let basis_retention = self
            .branches
            .acquire_admitted_retention(branch_id)
            .map_err(
                |denial| SignalBranchSnapshotReconstructionDenial::RetentionUnavailable { denial },
            )?;
        let snapshot_retention = self
            .branches
            .acquire_admitted_retention(branch_id)
            .map_err(
                |denial| SignalBranchSnapshotReconstructionDenial::RetentionUnavailable { denial },
            )?;
        Ok(SignalBranchSnapshotReconstructionPreflight {
            branch,
            basis_retention,
            snapshot_retention,
        })
    }
}
