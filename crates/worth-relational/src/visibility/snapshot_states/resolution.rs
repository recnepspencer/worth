use crate::capabilities::{SnapshotSource, VisibilityPolicySource};
use crate::runtime::RelationalRuntime;
use crate::snapshots::data::{SnapshotHandle, SnapshotInspectionSummary};
use crate::visibility::cache_state::materialize_exact_state_for_basis;

use super::{build_visibility_state, SnapshotState, VisibilitySnapshotBasis};

pub(crate) struct ResolvedVisibilitySnapshot {
    pub(crate) handle: SnapshotHandle,
    pub(crate) state: SnapshotState,
}

pub(crate) fn resolve_snapshot_handle(
    runtime: &RelationalRuntime,
    handle: &SnapshotHandle,
) -> Option<SnapshotHandle> {
    if handle.runtime_instance_id != runtime.runtime_instance_id() {
        return None;
    }
    if let Some((branch_id, version_id, read_policy)) =
        runtime.active_snapshot_binding(handle.snapshot_id)
    {
        return Some(SnapshotHandle {
            runtime_instance_id: runtime.runtime_instance_id(),
            branch_id,
            snapshot_id: handle.snapshot_id,
            version_id,
            read_policy,
        });
    }

    let binding = runtime
        .visibility
        .published_snapshot_binding(handle.snapshot_id)?;
    Some(SnapshotHandle {
        runtime_instance_id: runtime.runtime_instance_id(),
        branch_id: binding.branch_id,
        snapshot_id: handle.snapshot_id,
        version_id: binding.version_id,
        read_policy: binding.read_policy,
    })
}

pub(crate) fn resolve_snapshot_basis(
    runtime: &RelationalRuntime,
    handle: &SnapshotHandle,
) -> Option<VisibilitySnapshotBasis> {
    if handle.runtime_instance_id != runtime.runtime_instance_id() {
        return None;
    }
    if let Some(binding) = runtime.visibility.active_handle_binding(handle.snapshot_id) {
        return Some(binding.basis.clone());
    }
    runtime
        .visibility
        .published_snapshot_binding(handle.snapshot_id)
        .map(|binding| binding.basis)
}

pub(crate) fn resolve_snapshot_state(
    runtime: &RelationalRuntime,
    handle: &SnapshotHandle,
) -> Option<ResolvedVisibilitySnapshot> {
    if handle.runtime_instance_id != runtime.runtime_instance_id() {
        return None;
    }
    if let Some(binding) = runtime.visibility.active_handle_binding(handle.snapshot_id) {
        let resolved_handle = SnapshotHandle {
            runtime_instance_id: runtime.runtime_instance_id(),
            branch_id: binding.branch_id,
            snapshot_id: handle.snapshot_id,
            version_id: binding.version_id,
            read_policy: binding.read_policy,
        };
        let state = materialize_exact_state_for_basis(
            runtime,
            binding.basis,
            !runtime.protect_active_snapshots(),
        )?;
        return Some(ResolvedVisibilitySnapshot {
            handle: resolved_handle,
            state,
        });
    }

    resolve_published_snapshot_state(runtime, handle)
}

fn resolve_published_snapshot_state(
    runtime: &RelationalRuntime,
    handle: &SnapshotHandle,
) -> Option<ResolvedVisibilitySnapshot> {
    let binding = runtime
        .visibility
        .published_snapshot_binding(handle.snapshot_id)?;
    let resolved_handle = SnapshotHandle {
        runtime_instance_id: runtime.runtime_instance_id(),
        branch_id: binding.branch_id,
        snapshot_id: handle.snapshot_id,
        version_id: binding.version_id,
        read_policy: binding.read_policy,
    };
    let state = materialize_exact_state_for_basis(runtime, binding.basis.clone(), true)
        .unwrap_or_else(|| {
            build_visibility_state(
                runtime,
                binding.basis,
                handle.snapshot_id,
                binding.read_policy,
            )
        });
    Some(ResolvedVisibilitySnapshot {
        handle: resolved_handle,
        state,
    })
}

pub(crate) fn resolve_snapshot_inspection(
    runtime: &RelationalRuntime,
    handle: &SnapshotHandle,
) -> Option<SnapshotInspectionSummary> {
    let handle = resolve_snapshot_handle(runtime, handle)?;
    let basis = resolve_snapshot_basis(runtime, &handle)?;
    let counts = basis.root().live_counts();
    Some(SnapshotInspectionSummary {
        branch_id: handle.branch_id,
        version_id: handle.version_id,
        root_id: Some(basis.root().id()),
        entity_count: counts.entities,
        relation_count: counts.relations,
        pinned_entity_count: 0,
        pinned_relation_count: 0,
    })
}
