use crate::runtime::state::subsystems::visibility::VisibilitySubsystem;
use crate::runtime::RelationalRuntime;
use crate::snapshots::data::{SnapshotHandle, SnapshotInspectionSummary};
use crate::visibility::cache_state::materialize_exact_state_for_basis;

use super::{build_visibility_state, SnapshotState, VisibilitySnapshotBasis};

pub(crate) struct ResolvedVisibilitySnapshot {
    pub(crate) handle: SnapshotHandle,
    pub(crate) state: SnapshotState,
}

#[derive(Clone, Copy)]
pub(crate) struct SnapshotReadDirectory<'a> {
    visibility: &'a VisibilitySubsystem,
    runtime_identity: u64,
}

impl<'a> SnapshotReadDirectory<'a> {
    pub(crate) fn new(visibility: &'a VisibilitySubsystem, runtime_identity: u64) -> Self {
        Self {
            visibility,
            runtime_identity,
        }
    }
}

pub(crate) fn resolve_snapshot_handle<'a>(
    directory: impl Into<SnapshotReadDirectory<'a>>,
    handle: &SnapshotHandle,
) -> Option<SnapshotHandle> {
    let directory = directory.into();
    if handle.runtime_instance_id != directory.runtime_identity {
        return None;
    }
    if let Some(binding) = directory
        .visibility
        .active_handle_binding(handle.snapshot_id)
    {
        return Some(SnapshotHandle {
            runtime_instance_id: directory.runtime_identity,
            branch_id: binding.branch_id.clone(),
            snapshot_id: handle.snapshot_id,
            version_id: binding.version_id,
            read_policy: binding.read_policy,
        });
    }
    let binding = directory
        .visibility
        .published_snapshot_binding(handle.snapshot_id)?;
    Some(SnapshotHandle {
        runtime_instance_id: directory.runtime_identity,
        branch_id: binding.branch_id,
        snapshot_id: handle.snapshot_id,
        version_id: binding.version_id,
        read_policy: binding.read_policy,
    })
}

pub(crate) fn resolve_snapshot_basis<'a>(
    directory: impl Into<SnapshotReadDirectory<'a>>,
    handle: &SnapshotHandle,
) -> Option<VisibilitySnapshotBasis> {
    let directory = directory.into();
    if handle.runtime_instance_id != directory.runtime_identity {
        return None;
    }
    if let Some(binding) = directory
        .visibility
        .active_handle_binding(handle.snapshot_id)
    {
        return Some(binding.basis.clone());
    }
    directory
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
            !runtime
                .config
                .visibility
                .cache_policy
                .protect_active_snapshots,
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
                runtime.runtime_instance_id(),
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

pub(crate) fn resolve_snapshot_inspection<'a>(
    directory: impl Into<SnapshotReadDirectory<'a>>,
    handle: &SnapshotHandle,
) -> Option<SnapshotInspectionSummary> {
    let directory = directory.into();
    let handle = resolve_snapshot_handle(directory, handle)?;
    let basis = resolve_snapshot_basis(directory, &handle)?;
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
