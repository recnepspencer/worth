use worth_signal::facade::history::{RuntimeBranchId, RuntimeSnapshot};

use crate::boundary::errors::WorthSignalJsError;
use crate::runtime::summaries::{ReplaySummary, RuntimeSnapshotEnvelope};

use super::RuntimeCore;

impl RuntimeCore {
    #[cfg(test)]
    pub(crate) fn snapshot_owner_registry_counts(&self) -> (usize, usize, usize) {
        (
            self.runtime_snapshots.len(),
            self.admitted_runtime_snapshots.len(),
            self.snapshot_states.len(),
        )
    }

    pub fn snapshot(&mut self) -> Result<RuntimeSnapshotEnvelope, WorthSignalJsError> {
        let branch = self.runtime.current_branch();
        let basis = self.native_branch_basis(branch)?;
        let (admitted_snapshot, _) = self
            .runtime
            .capture_signal_branch_snapshot(&basis)
            .map_err(|error| {
                WorthSignalJsError::invalid_input(format!(
                    "Signal snapshot capture denied: {error:?}"
                ))
            })?
            .into_parts();
        let snapshot = admitted_snapshot.snapshot().clone();
        let snapshot_key = runtime_snapshot_key(&snapshot);
        self.admitted_runtime_snapshots
            .insert(snapshot_key, admitted_snapshot);
        self.runtime_snapshots
            .insert(snapshot_key, snapshot.clone());
        self.snapshot_states
            .insert(snapshot_key, self.snapshot_branch_state());
        Ok(RuntimeSnapshotEnvelope {
            snapshot,
            state: self.lock_store()?.snapshot(&self.catalog),
        })
    }

    pub fn restore_snapshot(
        &mut self,
        envelope: RuntimeSnapshotEnvelope,
    ) -> Result<(), WorthSignalJsError> {
        self.ensure_callback_snapshot_availability(&envelope.state)?;
        let snapshot_key = runtime_snapshot_key(&envelope.snapshot);
        // Every denial is decided before the branch switch below, so a
        // refused restore leaves the active branch where it was.
        let admitted_snapshot = self
            .admitted_runtime_snapshots
            .get(&snapshot_key)
            .ok_or_else(|| unavailable_snapshot(snapshot_key))?;
        require_exact_snapshot_payload(&envelope.snapshot, admitted_snapshot.snapshot())?;
        let target_branch_id = envelope.snapshot.meta.branch_id.0;
        if target_branch_id != self.runtime.current_branch().id.0 {
            // A runtime snapshot belongs to the branch it was captured on.
            // Restoring it reactivates that branch first, so a snapshot taken
            // before a branch was created, switched to, or merged remains
            // restorable afterwards.
            self.switch_branch(target_branch_id).map_err(|error| {
                WorthSignalJsError::invalid_input(format!(
                    "runtime snapshot restore targets branch `{target_branch_id}` which cannot be reactivated: {}",
                    error.message
                ))
            })?;
        }
        let admitted_snapshot = self
            .admitted_runtime_snapshots
            .get(&snapshot_key)
            .ok_or_else(|| unavailable_snapshot(snapshot_key))?;
        let basis = self.native_branch_basis_by_id(target_branch_id)?;
        self.runtime
            .restore_signal_branch(&basis, admitted_snapshot)
            .map_err(|error| {
                WorthSignalJsError::invalid_input(format!(
                    "Signal snapshot restore denied: {error:?}"
                ))
            })?;
        self.restore_runtime_store_snapshot(envelope.state)?;
        Ok(())
    }

    pub fn replay_for_branch(
        &mut self,
        branch_id: u64,
    ) -> Result<ReplaySummary, WorthSignalJsError> {
        let replay = self.runtime.replay_for_branch(RuntimeBranchId(branch_id));
        self.replay_summary_with_callbacks(replay)
    }

    pub fn branch_snapshot(
        &mut self,
        branch_id: u64,
    ) -> Result<RuntimeSnapshot, WorthSignalJsError> {
        let branch = self
            .runtime
            .branch_handle(RuntimeBranchId(branch_id))
            .ok_or_else(|| {
                WorthSignalJsError::invalid_input(format!("unknown branch `{branch_id}`"))
            })?;
        let basis = self.native_branch_basis(branch)?;
        let (admitted_snapshot, _) = self
            .runtime
            .capture_signal_branch_snapshot(&basis)
            .map_err(|error| {
                WorthSignalJsError::invalid_input(format!(
                    "Signal branch snapshot capture denied: {error:?}"
                ))
            })?
            .into_parts();
        let snapshot = admitted_snapshot.snapshot().clone();
        let snapshot_key = runtime_snapshot_key(&snapshot);
        self.admitted_runtime_snapshots
            .insert(snapshot_key, admitted_snapshot);
        self.runtime_snapshots
            .insert(snapshot_key, snapshot.clone());
        self.snapshot_states
            .insert(snapshot_key, self.state_for_branch(branch_id));
        Ok(snapshot)
    }

    pub fn branch_snapshot_id(&mut self, branch_id: u64) -> Result<u64, WorthSignalJsError> {
        Ok(self.branch_snapshot(branch_id)?.meta.snapshot_id.0)
    }

    pub fn branch_snapshot_envelope(
        &mut self,
        branch_id: u64,
    ) -> Result<RuntimeSnapshotEnvelope, WorthSignalJsError> {
        let snapshot = self.branch_snapshot(branch_id)?;
        let snapshot_key = runtime_snapshot_key(&snapshot);
        let state = self
            .snapshot_states
            .get(&snapshot_key)
            .map(|state| state.store.clone())
            .ok_or_else(|| {
                WorthSignalJsError::internal(format!(
                    "snapshot `{}:{}` missing runtime-local branch state",
                    snapshot.meta.branch_id.0, snapshot.meta.snapshot_id.0
                ))
            })?;
        Ok(RuntimeSnapshotEnvelope { snapshot, state })
    }

    pub fn restore_branch_snapshot(
        &mut self,
        branch_id: u64,
        snapshot: RuntimeSnapshot,
    ) -> Result<(), WorthSignalJsError> {
        let snapshot_key = runtime_snapshot_key(&snapshot);
        let state = self
            .snapshot_states
            .get(&snapshot_key)
            .cloned()
            .ok_or_else(|| {
                WorthSignalJsError::internal(format!(
                    "snapshot `{}:{}` is missing runtime-local branch semantic state",
                    snapshot.meta.branch_id.0, snapshot.meta.snapshot_id.0
                ))
            })?;
        self.ensure_callback_snapshot_availability(&state.store)?;
        let admitted_snapshot = self
            .admitted_runtime_snapshots
            .get(&snapshot_key)
            .ok_or_else(|| unavailable_snapshot(snapshot_key))?;
        require_exact_snapshot_payload(&snapshot, admitted_snapshot.snapshot())?;
        let basis = self.native_branch_basis_by_id(branch_id)?;
        self.runtime
            .restore_signal_branch(&basis, admitted_snapshot)
            .map_err(|error| {
                WorthSignalJsError::invalid_input(format!(
                    "Signal branch snapshot restore denied: {error:?}"
                ))
            })?;
        self.branch_states.insert(branch_id, state.clone());
        if self.runtime.current_branch().id.0 == branch_id {
            self.restore_branch_state(state)?;
        }
        Ok(())
    }

    pub fn restore_branch_snapshot_by_id(
        &mut self,
        branch_id: u64,
        snapshot_id: u64,
    ) -> Result<(), WorthSignalJsError> {
        let snapshot = self
            .runtime_snapshots
            .get(&(branch_id, snapshot_id))
            .cloned()
            .ok_or_else(|| {
                WorthSignalJsError::invalid_input(format!(
                    "unknown runtime snapshot `{branch_id}:{snapshot_id}`"
                ))
            })?;
        self.restore_branch_snapshot(branch_id, snapshot)
    }
}

fn runtime_snapshot_key(snapshot: &RuntimeSnapshot) -> (u64, u64) {
    (snapshot.meta.branch_id.0, snapshot.meta.snapshot_id.0)
}

fn require_exact_snapshot_payload(
    supplied: &RuntimeSnapshot,
    admitted: &RuntimeSnapshot,
) -> Result<(), WorthSignalJsError> {
    let supplied = serde_json::to_vec(supplied).map_err(|error| {
        WorthSignalJsError::invalid_input(format!("snapshot payload encoding failed: {error}"))
    })?;
    let admitted = serde_json::to_vec(admitted).map_err(|error| {
        WorthSignalJsError::internal(format!("admitted snapshot encoding failed: {error}"))
    })?;
    if supplied == admitted {
        return Ok(());
    }
    Err(WorthSignalJsError::invalid_input(
        "snapshot payload does not match the owner-admitted snapshot",
    ))
}

fn unavailable_snapshot((branch_id, snapshot_id): (u64, u64)) -> WorthSignalJsError {
    WorthSignalJsError::invalid_input(format!(
        "snapshot `{branch_id}:{snapshot_id}` is not admitted by this runtime"
    ))
}
