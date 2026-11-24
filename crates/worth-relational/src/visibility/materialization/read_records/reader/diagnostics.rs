use super::context::VisibilityReadContext;
use crate::diagnostics::data::{
    DeterminismExpectation, DiagnosticCode, DiagnosticsArtifactKind, DiagnosticsScope,
    RelationalDiagnosticArtifact, RelationalDiagnosticFields, RelationalDiagnosticValue,
    RelationalDiagnosticsEntry,
};
use crate::runtime::state::subsystems::visibility::cache::VisibilityResidency;
use crate::snapshots::data::{SnapshotHandle, SnapshotId, SnapshotReadPolicy};
use crate::visibility::{
    cache_state::{
        cached_historical_state_for_version, cached_state, residency, residency_for_version,
        VisibilityCacheInputs,
    },
    snapshot_states::resolution::{
        resolve_snapshot_basis, resolve_snapshot_handle, SnapshotReadDirectory,
    },
};

impl<'runtime> VisibilityReadContext<'runtime> {
    pub fn inspect_version_read_path(
        &self,
        version_id: crate::identity::data::VersionId,
    ) -> Option<RelationalDiagnosticArtifact> {
        let inputs = VisibilityCacheInputs::from(self.runtime);
        if version_id.is_zero()
            || version_id.as_u64() > inputs.history.history.branch.current_version_id().as_u64()
        {
            return None;
        }

        let residency = residency_for_version(inputs, version_id);
        let cached = cached_historical_state_for_version(inputs, version_id).is_some();
        let recent_window = inputs.policy.recent_version_window;
        let protected = inputs.protects(&residency);
        let recent_candidate = inputs.policy.enabled && recent_window > 0 && !protected;

        let mut entries = Vec::new();
        if !cached {
            entries.push(snapshot_miss_entry(false));
        }
        entries.push(snapshot_decision_entry(
            cached,
            protected,
            recent_candidate,
            false,
        ));
        Some(snapshot_read_path_artifact(
            version_id,
            cached,
            recent_candidate,
            recent_window,
            residency,
            entries,
        ))
    }

    pub fn inspect_snapshot_read_path(
        &self,
        handle: &SnapshotHandle,
    ) -> Option<RelationalDiagnosticArtifact> {
        let directory = SnapshotReadDirectory::from(self.runtime);
        let inputs = VisibilityCacheInputs::from(self.runtime);
        let snapshot = resolve_snapshot_handle(directory, handle)?;
        let basis = resolve_snapshot_basis(directory, handle)?;
        let version_id = snapshot.version_id;
        let residency = residency(inputs, basis.key());
        let cached = cached_state(inputs, basis.key()).is_some();
        let recent_window = inputs.policy.recent_version_window;
        let published_handle = !directory.is_active(handle.snapshot_id);
        let protected = inputs.protects(&residency);
        let recent_candidate = (!inputs.policy.protect_active_snapshots || published_handle)
            && inputs.policy.enabled
            && recent_window > 0
            && !protected;
        let mut entries = Vec::new();
        if published_handle {
            entries.push(published_snapshot_handle_read_entry(
                handle.snapshot_id,
                snapshot.version_id,
                snapshot.read_policy,
            ));
        }
        if !cached {
            entries.push(snapshot_miss_entry(published_handle));
        }
        entries.push(snapshot_decision_entry(
            cached,
            protected,
            recent_candidate,
            published_handle,
        ));
        Some(snapshot_read_path_artifact(
            version_id,
            cached,
            recent_candidate,
            recent_window,
            residency,
            entries,
        ))
    }
}

fn snapshot_read_path_artifact(
    version_id: crate::identity::data::VersionId,
    cached: bool,
    recent_candidate: bool,
    recent_window: usize,
    residency: VisibilityResidency,
    mut extra_entries: Vec<RelationalDiagnosticsEntry>,
) -> RelationalDiagnosticArtifact {
    let mut entries = vec![RelationalDiagnosticsEntry::new(
        DiagnosticCode::SnapshotReadPathInspected,
        "snapshot/version read path inspected",
        snapshot_read_path_inspection_fields(
            version_id,
            cached,
            recent_candidate,
            recent_window,
            &residency,
        ),
    )];
    entries.append(&mut extra_entries);
    RelationalDiagnosticArtifact::new(
        DiagnosticsScope::Snapshot,
        DiagnosticsArtifactKind::DetailedTrace,
        DeterminismExpectation::Required,
        entries,
    )
}

fn snapshot_decision_entry(
    cached: bool,
    protected: bool,
    recent_candidate: bool,
    published_handle: bool,
) -> RelationalDiagnosticsEntry {
    let (code, message) = if cached {
        (
            DiagnosticCode::VisibilityCacheHit,
            "read will reuse cached visibility state",
        )
    } else if protected {
        (
            DiagnosticCode::VisibilityCacheProtectedRead,
            "read will reconstruct and keep a protected visibility state",
        )
    } else if recent_candidate {
        (
            DiagnosticCode::VisibilityCacheRecentAdmissionCandidate,
            "read will reconstruct and may admit visibility state into the recent cache",
        )
    } else {
        (
            DiagnosticCode::VisibilityCacheTransientRead,
            "read will reconstruct transient visibility state without cache residency",
        )
    };
    RelationalDiagnosticsEntry::new(
        code,
        message,
        snapshot_visibility_cache_decision_fields(
            cached,
            protected,
            recent_candidate,
            published_handle,
        ),
    )
}

fn snapshot_miss_entry(published_handle: bool) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::VisibilityCacheMissReconstructed,
        "read will reconstruct visibility state from committed history",
        visibility_cache_miss_fields(published_handle),
    )
}

fn published_snapshot_handle_read_entry(
    snapshot_id: SnapshotId,
    version_id: crate::identity::data::VersionId,
    read_policy: SnapshotReadPolicy,
) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::PublishedSnapshotHandleRead,
        "snapshot read will resolve through a published handle",
        published_snapshot_handle_read_fields(snapshot_id, version_id, read_policy),
    )
}

fn published_snapshot_handle_read_fields(
    snapshot_id: SnapshotId,
    version_id: crate::identity::data::VersionId,
    read_policy: SnapshotReadPolicy,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        (
            "snapshot_id",
            RelationalDiagnosticValue::SnapshotId(snapshot_id),
        ),
        (
            "version_id",
            RelationalDiagnosticValue::VersionId(version_id),
        ),
        ("read_policy", snapshot_read_policy_value(read_policy)),
    ])
    .into()
}

fn snapshot_read_path_inspection_fields(
    version_id: crate::identity::data::VersionId,
    cached_visibility_state: bool,
    recent_candidate: bool,
    recent_window: usize,
    residency: &VisibilityResidency,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        (
            "version_id",
            RelationalDiagnosticValue::VersionId(version_id),
        ),
        (
            "cached_visibility_state",
            RelationalDiagnosticValue::Bool(cached_visibility_state),
        ),
        (
            "recent_candidate",
            RelationalDiagnosticValue::Bool(recent_candidate),
        ),
        (
            "recent_window",
            RelationalDiagnosticValue::unsigned(recent_window),
        ),
        (
            "recent_resident",
            RelationalDiagnosticValue::Bool(residency.recent_resident),
        ),
        (
            "branch_head_refs",
            RelationalDiagnosticValue::unsigned(residency.branch_head_refs as usize),
        ),
        (
            "replay_refs",
            RelationalDiagnosticValue::unsigned(residency.replay_refs as usize),
        ),
        (
            "active_snapshot_refs",
            RelationalDiagnosticValue::unsigned(residency.active_snapshot_refs as usize),
        ),
    ])
    .into()
}

fn snapshot_visibility_cache_decision_fields(
    cached_visibility_state: bool,
    protected_visibility_state: bool,
    recent_admission_candidate: bool,
    published_handle: bool,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        (
            "cached_visibility_state",
            RelationalDiagnosticValue::Bool(cached_visibility_state),
        ),
        (
            "protected_visibility_state",
            RelationalDiagnosticValue::Bool(protected_visibility_state),
        ),
        (
            "recent_admission_candidate",
            RelationalDiagnosticValue::Bool(recent_admission_candidate),
        ),
        (
            "published_handle",
            RelationalDiagnosticValue::Bool(published_handle),
        ),
    ])
    .into()
}

fn visibility_cache_miss_fields(published_handle: bool) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([(
        "published_handle",
        RelationalDiagnosticValue::Bool(published_handle),
    )])
    .into()
}

fn snapshot_read_policy_value(read_policy: SnapshotReadPolicy) -> RelationalDiagnosticValue {
    match read_policy {
        SnapshotReadPolicy::ImmutablePinned => RelationalDiagnosticValue::string("ImmutablePinned"),
        SnapshotReadPolicy::ImmutablePinnedNoLazyMutation => {
            RelationalDiagnosticValue::string("ImmutablePinnedNoLazyMutation")
        }
    }
}
