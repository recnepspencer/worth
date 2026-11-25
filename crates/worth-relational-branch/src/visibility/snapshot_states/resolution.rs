use crate::history::retention::VisibilitySnapshotBasis;
use crate::runtime::state::subsystems::visibility::VisibilitySubsystem;
use crate::snapshots::data::{SnapshotHandle, SnapshotInspectionSummary};
use crate::visibility::cache_state::{materialize_exact_state, VisibilityCacheInputs};

use super::SnapshotState;

pub struct ResolvedVisibilitySnapshot {
    pub handle: SnapshotHandle,
    pub state: SnapshotState,
}

#[derive(Clone, Copy)]
pub struct SnapshotReadDirectory<'a> {
    visibility: &'a VisibilitySubsystem,
    runtime_identity: u64,
}

impl<'a> SnapshotReadDirectory<'a> {
    pub fn new(visibility: &'a VisibilitySubsystem, runtime_identity: u64) -> Self {
        Self {
            visibility,
            runtime_identity,
        }
    }
    pub fn is_active(self, id: crate::snapshots::data::SnapshotId) -> bool {
        self.visibility.active_handle_binding(id).is_some()
    }
}

pub fn resolve_snapshot_handle<'a>(
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

pub fn resolve_snapshot_basis<'a>(
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

#[derive(Clone, Copy)]
pub struct SnapshotResolutionInputs<'a> {
    pub directory: SnapshotReadDirectory<'a>,
    pub cache: VisibilityCacheInputs<'a>,
}

pub fn resolve_snapshot_state<'a>(
    inputs: impl Into<SnapshotResolutionInputs<'a>>,
    handle: &SnapshotHandle,
) -> Option<ResolvedVisibilitySnapshot> {
    let inputs = inputs.into();
    if handle.runtime_instance_id != inputs.directory.runtime_identity {
        return None;
    }
    if let Some(binding) = inputs
        .directory
        .visibility
        .active_handle_binding(handle.snapshot_id)
    {
        let resolved_handle = SnapshotHandle {
            runtime_instance_id: inputs.directory.runtime_identity,
            branch_id: binding.branch_id,
            snapshot_id: handle.snapshot_id,
            version_id: binding.version_id,
            read_policy: binding.read_policy,
        };
        let state = materialize_selected_state(
            inputs.cache,
            &binding.basis,
            !inputs.cache.policy.protect_active_snapshots,
        )?;
        return Some(ResolvedVisibilitySnapshot {
            handle: resolved_handle,
            state,
        });
    }

    resolve_published_snapshot_state(inputs, handle)
}

fn resolve_published_snapshot_state(
    inputs: SnapshotResolutionInputs<'_>,
    handle: &SnapshotHandle,
) -> Option<ResolvedVisibilitySnapshot> {
    let binding = inputs
        .directory
        .visibility
        .published_snapshot_binding(handle.snapshot_id)?;
    let resolved_handle = SnapshotHandle {
        runtime_instance_id: inputs.directory.runtime_identity,
        branch_id: binding.branch_id,
        snapshot_id: handle.snapshot_id,
        version_id: binding.version_id,
        read_policy: binding.read_policy,
    };
    let state =
        materialize_selected_state(inputs.cache, &binding.basis, true).unwrap_or_else(|| {
            binding.basis.build_visibility_state(
                inputs.directory.runtime_identity,
                handle.snapshot_id,
                binding.read_policy,
            )
        });
    Some(ResolvedVisibilitySnapshot {
        handle: resolved_handle,
        state,
    })
}

pub fn resolve_snapshot_inspection<'a>(
    directory: impl Into<SnapshotReadDirectory<'a>>,
    handle: &SnapshotHandle,
) -> Option<SnapshotInspectionSummary> {
    let directory = directory.into();
    let handle = resolve_snapshot_handle(directory, handle)?;
    let basis = resolve_snapshot_basis(directory, &handle)?;
    let counts = basis.projection_root().live_counts();
    Some(SnapshotInspectionSummary {
        branch_id: handle.branch_id,
        version_id: handle.version_id,
        root_id: Some(basis.projection_root().id()),
        entity_count: counts.entities,
        relation_count: counts.relations,
        pinned_entity_count: 0,
        pinned_relation_count: 0,
    })
}

fn materialize_selected_state(
    inputs: VisibilityCacheInputs<'_>,
    basis: &VisibilitySnapshotBasis,
    allow_recent_admission: bool,
) -> Option<SnapshotState> {
    let key = basis.key();
    materialize_exact_state(inputs, key, allow_recent_admission, || {
        basis.clone().build_visibility_state(
            inputs.runtime_identity,
            crate::snapshots::data::SnapshotId(0),
            crate::snapshots::data::SnapshotReadPolicy::ImmutablePinnedNoLazyMutation,
        )
    })
}
