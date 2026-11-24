//! Open and release the exact active registration of a retained observation.
#[path = "snapshot_observation_registration.rs"]
mod registration;

use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use crate::runtime::state::subsystems::{
    history::HistorySubsystem, visibility::VisibilitySubsystem,
};
use crate::visibility::cache_state::{bump_active_snapshot_ref, VisibilityCacheInputs};
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
            bump_active_snapshot_ref(self.cache, binding.basis.key(), -1);
            return Ok(RelationalSnapshotReleaseReceipt::new(handle.snapshot_id()));
        }
        self.visibility
            .remove_published_handle(handle.snapshot_id())
            .map(|_| RelationalSnapshotReleaseReceipt::new(handle.snapshot_id()))
            .ok_or(RelationalSnapshotReleaseDenial::UnknownSnapshot)
    }
}
