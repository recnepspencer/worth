use crate::capabilities::{SnapshotSource, VisibilityPolicySource};
use crate::runtime::RelationalRuntime;
use crate::snapshots::data::{SnapshotHandle, SnapshotInspectionSummary};
use crate::visibility::cache_state::materialize_exact_state_for_basis;

use super::{build_visibility_state, SnapshotState, VisibilitySnapshotBasis};

pub(crate) struct ResolvedVisibilitySnapshot {
    pub(crate) handle: SnapshotHandle,
    pub(crate) state: SnapshotState,
    pub(crate) keeps_storage_pins: bool,
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
            keeps_storage_pins: false,
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
        keeps_storage_pins: false,
    })
}

pub(crate) fn resolve_snapshot_inspection(
    runtime: &RelationalRuntime,
    handle: &SnapshotHandle,
) -> Option<SnapshotInspectionSummary> {
    let resolved = resolve_snapshot_state(runtime, handle)?;
    if resolved.keeps_storage_pins {
        return Some(SnapshotInspectionSummary {
            branch_id: resolved.handle.branch_id,
            version_id: resolved.handle.version_id,
            root_id: resolved.state.basis.root_id(),
            entity_count: resolved.state.pinned_entity_count,
            relation_count: resolved.state.pinned_relation_count,
            pinned_entity_count: resolved.state.pinned_entity_count,
            pinned_relation_count: resolved.state.pinned_relation_count,
        });
    }
    Some(SnapshotInspectionSummary {
        branch_id: resolved.handle.branch_id,
        version_id: resolved.handle.version_id,
        root_id: resolved.state.basis.root_id(),
        entity_count: resolved.state.pinned_entity_count,
        relation_count: resolved.state.pinned_relation_count,
        pinned_entity_count: 0,
        pinned_relation_count: 0,
    })
}
