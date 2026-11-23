//! Open and release the exact active registration of a retained observation.
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use crate::runtime::state::subsystems::{
    history::HistorySubsystem, visibility::VisibilitySubsystem,
};
use crate::visibility::cache_state::{bump_active_snapshot_ref, VisibilityCacheInputs};
use crate::visibility::snapshot_admission::RelationalSnapshotAdmissionDenial;
use crate::visibility::snapshot_release::{
    RelationalSnapshotReleaseDenial, RelationalSnapshotReleaseReceipt,
};

/// Read and registration custody lent by the observation's installed owner.
/// Construction binds the actual native directory, capacity policy, and history.
#[derive(Clone, Copy)]
pub struct SnapshotObservationInputs<'a> {
    pub(crate) runtime_identity: u64,
    pub(crate) symbols: &'a RuntimeSymbolTable,
    pub(crate) visibility: &'a VisibilitySubsystem,
    pub(crate) history: &'a HistorySubsystem,
    pub(crate) maximum_active_snapshots: Option<usize>,
    pub(crate) cache: VisibilityCacheInputs<'a>,
}

impl SnapshotObservationInputs<'_> {
    pub(in crate::visibility) fn open_active_snapshot_for_basis(
        &self,
        basis: crate::visibility::snapshot_states::VisibilitySnapshotBasis,
        read_policy: crate::snapshots::data::SnapshotReadPolicy,
    ) -> Result<crate::snapshots::data::SnapshotHandle, RelationalSnapshotAdmissionDenial> {
        let maximum_active_snapshots = self.maximum_active_snapshots;
        if let Some(maximum_active_snapshots) = maximum_active_snapshots
            .filter(|maximum| self.visibility.active_snapshot_count() >= *maximum)
        {
            return Err(
                RelationalSnapshotAdmissionDenial::ActiveSnapshotCapacityExhausted {
                    maximum_active_snapshots,
                },
            );
        }
        let snapshot_id = self
            .visibility
            .allocate_snapshot_id()
            .ok_or(RelationalSnapshotAdmissionDenial::SnapshotIdentityExhausted)?;
        let handle = crate::snapshots::data::SnapshotHandle {
            runtime_instance_id: self.runtime_identity,
            branch_id: basis.branch_id().clone(),
            snapshot_id,
            version_id: basis.version_id(),
            read_policy,
        };
        // The binding carries the exact immutable owner root and its retained
        // observation. That O(1) obligation is the snapshot's storage fence;
        // materialized visibility remains requested read work.
        self.visibility.insert_active_handle(
            handle.snapshot_id,
            crate::runtime::state::subsystems::visibility::SnapshotHandleBinding::new(
                basis.clone(),
                handle.read_policy,
            ),
        );
        bump_active_snapshot_ref(self.cache, &basis, 1);
        Ok(handle)
    }

    pub fn snapshot_for_observation(
        &self,
        observation: &crate::mvcc::observation::RelationalBranchObservation,
    ) -> Result<crate::snapshots::data::SnapshotHandle, RelationalSnapshotAdmissionDenial> {
        if observation.identity().runtime_instance_id() != self.runtime_identity {
            return Err(RelationalSnapshotAdmissionDenial::ForeignRuntime {
                expected_runtime_instance_id: self.runtime_identity,
                actual_runtime_instance_id: observation.identity().runtime_instance_id(),
            });
        }
        let basis = crate::visibility::snapshot_states::VisibilitySnapshotBasis::from_observation(
            observation,
        );
        let handle = self.open_active_snapshot_for_basis(
            basis,
            crate::snapshots::data::SnapshotReadPolicy::ImmutablePinnedNoLazyMutation,
        )?;
        self.history
            .record_snapshot_root_read(observation.identity().branch_id());
        Ok(handle)
    }

    pub fn release_snapshot(
        &self,
        handle: &crate::snapshots::data::SnapshotHandle,
    ) -> Result<RelationalSnapshotReleaseReceipt, RelationalSnapshotReleaseDenial> {
        if handle.runtime_instance_id() != self.runtime_identity {
            return Err(RelationalSnapshotReleaseDenial::ForeignRuntime {
                expected_runtime_instance_id: self.runtime_identity,
                actual_runtime_instance_id: handle.runtime_instance_id(),
            });
        }
        let active_binding = self.visibility.active_handle_binding(handle.snapshot_id());
        let published_binding = self
            .visibility
            .published_snapshot_binding(handle.snapshot_id());
        let binding = active_binding
            .or(published_binding)
            .ok_or(RelationalSnapshotReleaseDenial::UnknownSnapshot)?;
        if binding.branch_id != *handle.branch_id()
            || binding.version_id != handle.version_id()
            || binding.read_policy != handle.read_policy()
        {
            return Err(RelationalSnapshotReleaseDenial::BindingMismatch);
        }
        if let Some(binding) = self.visibility.remove_active_handle(handle.snapshot_id()) {
            bump_active_snapshot_ref(self.cache, &binding.basis, -1);
            return Ok(RelationalSnapshotReleaseReceipt::new(handle.snapshot_id()));
        }
        self.visibility
            .remove_published_handle(handle.snapshot_id())
            .map(|_| RelationalSnapshotReleaseReceipt::new(handle.snapshot_id()))
            .ok_or(RelationalSnapshotReleaseDenial::UnknownSnapshot)
    }
}
