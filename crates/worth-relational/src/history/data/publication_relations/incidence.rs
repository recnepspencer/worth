//! Reached incidence distinguishes ownership from independently committed references.
use super::{
    charge, creation, endpoints, CanonicalCommitEnvelope, CanonicalRequiredOutputOrigin, Stop,
};
use crate::branch::RelationalBranchRoot;
use crate::identity::data::VersionId;
use crate::storage::overlay::{current_relation_partition_source_for_slot, CurrentPartitionAccess};
use crate::storage::partition::AdjacencyDirection;
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionResourceLease, MapKernelContext};

#[allow(clippy::too_many_arguments)]
pub(super) fn validate(
    incidence: &dyn CurrentPartitionAccess,
    state: &dyn CurrentPartitionAccess,
    root: &RelationalBranchRoot,
    origin: &CanonicalRequiredOutputOrigin,
    current: Option<&CanonicalCommitEnvelope>,
    lookup: &mut impl FnMut(VersionId) -> Option<Arc<CanonicalCommitEnvelope>>,
    lease: Option<&ExecutionResourceLease<'_>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Stop> {
    for output in &origin.outputs {
        charge(context, 1)?;
        for direction in [AdjacencyDirection::Outgoing, AdjacencyDirection::Incoming] {
            let ids = incidence
                .get_current_partition(output.partition_id)
                .map(|partition| match direction {
                    AdjacencyDirection::Outgoing => partition.outgoing_candidates(*output),
                    AdjacencyDirection::Incoming => partition.incoming_candidates(*output),
                })
                .unwrap_or_default();
            let mut memory = None;
            let mut iterator = ids.try_iter(|work, bytes| {
                context
                    .checkpoint(work)
                    .map_err(|cause| Stop::Request(cause.into()))?;
                memory = Some(
                    ExecutionMemoryReservation::reserve_in_scope(lease, bytes).map_err(|stop| {
                        Stop::Request(crate::execution::denial::lease_denial(stop))
                    })?,
                );
                Ok::<_, Stop>(())
            })?;
            while let Some(id) = iterator.try_next(|work, _| {
                context
                    .checkpoint(work)
                    .map_err(|cause| Stop::Request(cause.into()))
            })? {
                charge(
                    context,
                    4 + (usize::BITS - origin.relations.len().leading_zeros()) as usize
                        + 2 * (usize::BITS - origin.outputs.len().leading_zeros()) as usize,
                )?;
                let (_, partition) = current_relation_partition_source_for_slot(
                    state,
                    id.partition_id,
                    id.slot_index(),
                )
                .ok_or("incident relation partition absent")?;
                let row = partition
                    .relation_arena()
                    .get(id)
                    .ok_or("incident relation generation absent")?;
                if !row.is_live() {
                    continue;
                }
                if origin
                    .relations
                    .binary_search_by_key(id, |entry| entry.id)
                    .is_ok()
                {
                    continue;
                }
                let ends = row
                    .extra()
                    .endpoints
                    .as_ref()
                    .ok_or("live incident relation endpoints absent")?;
                let source = origin.outputs.binary_search(&ends.source).is_ok();
                let target = origin.outputs.binary_search(&ends.target).is_ok();
                if source == target {
                    return Err("unclaimed internal or misindexed incident relation lacks producer provenance".into());
                }
                creation::validate(state, *id, origin, current, lookup, context)?;
                let kind = row.kind_id().ok_or("incident relation kind absent")?;
                if root.schema_authority().relation_aspect_plan(kind).is_none() {
                    return Err("incident relation installed plan absent".into());
                }
                endpoints::validate(
                    state,
                    root.schema_authority().registry(),
                    kind,
                    ends.source,
                    ends.target,
                    context,
                )?;
            }
        }
    }
    Ok(())
}
