use crate::diagnostics::data::{
    DiagnosticCode, DiagnosticsScope, RelationalDiagnosticFields, RelationalDiagnosticValue,
    RelationalDiagnosticsEntry,
};
use crate::identity::data::VersionId;
use crate::visibility::cache_state::{
    bump_replay_ref, ensure_historical_state, evict_cache_if_needed,
    historical_basis_for_retained_version,
};
use crate::visibility::snapshot_states::SnapshotState;

use super::HistoryAuthority;

impl<'runtime> HistoryAuthority<'runtime> {
    pub fn retain_version_for_replay(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        version_id: crate::identity::data::VersionId,
    ) -> Result<bool, crate::history::data::RelationalReplayRetentionDenial> {
        crate::execution::check_request(execution)?;
        if self
            .runtime
            .visibility
            .increment_replay_retention(version_id)
            .is_some()
        {
            if self
                .runtime
                .config
                .visibility
                .cache_policy
                .protect_replay_retained
            {
                bump_replay_ref(self.runtime, version_id, 1);
            }
            return Ok(true);
        }
        if version_id.is_zero() || version_id.as_u64() > self.runtime.current_version_id().as_u64()
        {
            return Ok(false);
        }
        let Ok(basis) = historical_basis_for_retained_version(self.runtime, version_id) else {
            return Ok(false);
        };
        let state = ensure_historical_state(execution, self.runtime, basis, false)?;
        let pins = self
            .runtime
            .visibility_pins()
            .demand_replay_pins(execution, state)?;
        let diagnostic = replay_retention_diagnostic(
            DiagnosticCode::ReplayRetentionPinned,
            "replay retention pinned historical visibility state",
            version_id,
            pins.state(),
        );
        let retention_fence = self
            .runtime
            .visibility
            .retention_fence_version(self.runtime.current_version_id());
        let installed = self
            .runtime
            .visibility
            .retain_resident_replay(version_id, pins, |pins| {
                self.runtime
                    .visibility_pins()
                    .pin_replay_state(pins, retention_fence);
            });
        if self
            .runtime
            .config
            .visibility
            .cache_policy
            .protect_replay_retained
        {
            bump_replay_ref(self.runtime, version_id, 1);
        }
        if installed {
            self.runtime
                .publication_authority()
                .diagnostic(DiagnosticsScope::Retention)
                .minimal_summary()
                .entries([diagnostic])
                .emit();
        }
        Ok(true)
    }

    pub fn release_version_replay_retention(
        &self,
        version_id: crate::identity::data::VersionId,
    ) -> bool {
        let retention_fence = self
            .runtime
            .visibility
            .retention_fence_version(self.runtime.current_version_id());
        let mut diagnostic = None;
        let released = self
            .runtime
            .visibility
            .release_resident_replay(version_id, |pins| {
                self.runtime
                    .visibility_pins()
                    .unpin_replay_state(pins, retention_fence);
                diagnostic = Some(replay_retention_diagnostic(
                    DiagnosticCode::ReplayRetentionReleased,
                    "replay retention released historical visibility state",
                    version_id,
                    pins.state(),
                ));
            });
        if !released {
            return false;
        }
        if self
            .runtime
            .config
            .visibility
            .cache_policy
            .protect_replay_retained
        {
            bump_replay_ref(self.runtime, version_id, -1);
            if diagnostic.is_some() {
                evict_cache_if_needed(self.runtime);
            }
        }
        if let Some(diagnostic) = diagnostic {
            self.runtime
                .publication_authority()
                .diagnostic(DiagnosticsScope::Retention)
                .minimal_summary()
                .entries([diagnostic])
                .emit();
        }

        true
    }
}

fn replay_retention_diagnostic(
    code: DiagnosticCode,
    message: &str,
    version_id: VersionId,
    state: &SnapshotState,
) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(code, message, replay_retention_fields(version_id, state))
}

fn replay_retention_fields(
    version_id: VersionId,
    state: &SnapshotState,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        (
            "version_id",
            RelationalDiagnosticValue::VersionId(version_id),
        ),
        (
            "pinned_entity_count",
            RelationalDiagnosticValue::unsigned(state.pinned_entity_count),
        ),
        (
            "pinned_relation_count",
            RelationalDiagnosticValue::unsigned(state.pinned_relation_count),
        ),
        (
            "pinned_partition_count",
            RelationalDiagnosticValue::unsigned(state.pinned_partitions.len()),
        ),
    ])
    .into()
}
