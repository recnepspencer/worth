//! One actual external cut, with no name copies or hidden DashMap iterator Arc.
use super::super::{
    RelationalBranchRetentionBinding, RelationalRetentionRequestAcquisitionDenial as Denial,
};
use super::*;
use crate::branch::AdmittedRelationalBranchBasis;
use worth_execution::{
    ExecutionMemoryReservation, ExecutionOrderedMap, ExecutionOrderedMapDenial, ExecutionRequest,
};

#[derive(Debug)]
pub(crate) struct CapturedExternalCheckpointComponents {
    held: Vec<AdmittedRelationalBranchBasis>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl CapturedExternalCheckpointComponents {
    pub(crate) fn iter(&self) -> std::slice::Iter<'_, AdmittedRelationalBranchBasis> {
        self.held.iter()
    }
}
impl RelationalBranchRetentionOwner {
    pub(crate) fn capture_checkpoint_components_for_request<'a>(
        &self,
        request: &ExecutionRequest<'_, '_>,
        current: impl IntoIterator<Item = &'a RelationalBranchRetentionBinding>,
    ) -> Result<CapturedExternalCheckpointComponents, Denial> {
        // Scratch precedes the cut, so error unwinding releases every guard
        // before destroying the actual admitted transient map owners.
        let output_memory: Option<ExecutionMemoryReservation>;
        let mut output: Vec<AdmittedRelationalBranchBasis>;
        let mut lanes = ExecutionOrderedMap::<usize, RelationalBranchRetentionBinding>::default();
        let mut held = ExecutionOrderedMap::default();
        let cut = self
            .inner
            .external_cut
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut failure = None;
        // DashMap::iter allocates private guard Arcs. retain's literal-true
        // visitor changes no entry and allocates no traversal storage. It takes
        // shard WRITE guards; after a stop its bounded remaining cold traversal
        // only returns true. No lane/head/state lock is entered in this visitor.
        self.inner.retired_roots.retain(|_, entry| {
            if failure.is_none() {
                for binding in entry.branch_bindings.values() {
                    if let Err(cause) = insert_lane(&mut lanes, binding, &self.inner, request) {
                        failure = Some(cause);
                        break;
                    }
                }
            }
            true
        });
        if let Some(cause) = failure {
            return Err(cause);
        }
        #[cfg(test)]
        {
            let pause = self
                .inner
                .component_capture_pause
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .clone();
            if let Some(pause) = pause {
                pause.wait();
            }
        }
        for binding in current {
            insert_lane(&mut lanes, binding, &self.inner, request)?;
        }
        // Head transfer preserves the actual lane; owner reset replaces both
        // lanes and the retired inventory. Therefore allocation-identity dedup
        // selects exactly the same per-branch lane as the ordinary name map.
        for (lane_id, binding) in lanes.iter() {
            binding.capture_checkpoint_components_into(*lane_id, &mut held, request)?;
        }
        let layout = std::alloc::Layout::array::<AdmittedRelationalBranchBasis>(held.len())
            .map_err(|_| Denial::Storage(ExecutionOrderedMapDenial::CapacityOverflow))?;
        let bytes = u64::try_from(layout.size())
            .map_err(|_| Denial::Storage(ExecutionOrderedMapDenial::CapacityOverflow))?;
        output_memory = request
            .reserve_retained_memory(bytes)
            .map_err(|cause| Denial::Request(cause.into()))?;
        output = Vec::with_capacity(held.len());
        for basis in held.values() {
            request
                .consult()
                .map_err(|cause| Denial::Request(cause.into()))?;
            output.push(basis.clone());
        }
        drop(cut);
        drop(held);
        drop(lanes);
        Ok(CapturedExternalCheckpointComponents {
            held: output,
            _memory: output_memory,
        })
    }
}
fn insert_lane(
    lanes: &mut ExecutionOrderedMap<usize, RelationalBranchRetentionBinding>,
    binding: &RelationalBranchRetentionBinding,
    owner: &Arc<RelationalBranchRetentionOwnerInner>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    if !std::sync::Weak::ptr_eq(&binding.owner, &Arc::downgrade(owner)) {
        return Err(Denial::Retention(
            super::super::RelationalRetentionAcquisitionDenial::BindingMismatch,
        ));
    }
    request
        .consult()
        .map_err(|cause| Denial::Request(cause.into()))?;
    let id = binding.checkpoint_lane_identity();
    if !lanes.contains_key(&id) {
        lanes
            .insert_cloned_native_for_request(&id, binding, *request)
            .map_err(Denial::Storage)?;
    }
    Ok(())
}

// Preserve the ordinary capacity-before-identity refusal priority without
// publishing a provisional obligation before original-request admission.
pub(in crate::history::retention) fn has_live_obligation_room(
    owner: &RelationalBranchRetentionOwnerInner,
) -> bool {
    owner
        .live_retention_count
        .load(std::sync::atomic::Ordering::Relaxed)
        < owner.max_live_root_obligations
}
