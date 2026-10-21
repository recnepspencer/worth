//! Paid exact-prefix pages over an already issued immutable ordered-index pin.
use super::bounded_related_entity_ordered_lookup::{prepare_related_lookup, PreparedRelatedLookup};
use crate::indexes::data::*;
use crate::runtime::RelationalPinnedRead;
use worth_execution::{
    ExecutionAllocationPolicy, ExecutionArrayBuilder, ExecutionRequest, ExecutionWorkCeiling,
    MapKernelContext,
};
use BoundedRelatedEntityPrefixDenial as Denial;
mod range;
mod validation;

impl RelationalPinnedRead {
    /// Seeks only the selected leading-value range and publishes one complete funded page.
    /// The original request supplies work, interruption and payload allocation authority.
    /// No cold maintenance, population scan or current registry read occurs here.
    pub fn execute_bounded_related_entity_prefix_lookup(
        &self,
        request: BoundedRelatedEntityPrefixLookupRequest<'_>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BoundedRelatedEntityPrefixPage, Denial> {
        execution
            .in_scope(|lease| {
                ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
                    lease,
                    None,
                    |meter| {
                        let allocation = lease
                            .map(ExecutionAllocationPolicy::Execution)
                            .unwrap_or(ExecutionAllocationPolicy::SystemAllocation);
                        self.lookup_prefix(&request, execution, allocation, meter)
                    },
                )
            })
            .map_err(Denial::Work)?
            .map_err(Denial::Work)?
            .0
    }
    fn lookup_prefix(
        &self,
        request: &BoundedRelatedEntityPrefixLookupRequest<'_>,
        execution: &ExecutionRequest<'_, '_>,
        allocation: ExecutionAllocationPolicy<'_, '_>,
        meter: &mut MapKernelContext<'_, '_>,
    ) -> Result<BoundedRelatedEntityPrefixPage, Denial> {
        charge(meter, 1)?;
        if request.ordered.after().is_some() {
            return Err(Denial::ForeignBoundary);
        }

        let prepared = prepare_related_lookup(self, &request.ordered).map_err(Denial::Lookup)?;
        let _projection_memory = reserve_basis_copy(self, allocation, meter)?;
        let projection = self
            .project_snapshot(request.ordered.snapshot())
            .ok_or_else(|| {
                Denial::Lookup(BoundedRelatedEntityOrderedLookupDenial::new(
                    BoundedRelatedEntityOrderedLookupDenialKind::SnapshotUnavailable,
                    request.ordered.index_id(),
                ))
            })?;
        validation::prefix(&projection, request, &prepared, meter)?;
        let DerivedIndexEntries::RelatedEntityOrdering(all) = &prepared.generation.entries else {
            unreachable!("prepared ordering kind")
        };

        let rows = all.get(&request.ordered.parent_entity_id());
        let (start, end, comparisons) = range::select(self, request, &prepared, rows, meter)?;
        let count = (end - start).min(request.ordered.page_width());
        // Exactly the native ID allocation, retained in the returned page until last Drop.
        charge(meter, count as u64)?;
        let mut ids =
            ExecutionArrayBuilder::allocate(count, allocation).map_err(Denial::Allocation)?;
        for offset in start..start + count {
            let entry = range::entry(rows.expect("nonempty selected range"), offset, meter)?;
            validation::row(
                self,
                &projection,
                request,
                &prepared,
                entry,
                execution,
                meter,
            )?;
            ids.push(entry.child_entity_id())
                .map_err(Denial::Allocation)?;
        }
        let ids = ids.seal().map_err(Denial::Allocation)?;
        charge(meter, 1)?;
        let boundary = if start + count < end {
            let memory = reserve_basis_copy(self, allocation, meter)?;
            Some(BoundedRelatedEntityPrefixBoundary {
                basis: self.pinned_basis().clone(),
                _memory: memory,
                runtime: self.query_context().runtime_instance_id,
                generation: prepared.generation.clone(),
                parent: request.ordered.parent_entity_id(),
                child_kind: request.ordered.child_kind(),
                prefix_len: request.prefix.len(),
                last: start + count - 1,
            })
        } else {
            None
        };
        Ok(BoundedRelatedEntityPrefixPage {
            ids,
            generation: prepared.generation_id,
            boundary,
            comparisons,
        })
    }
}
fn charge(meter: &mut MapKernelContext<'_, '_>, work: u64) -> Result<(), Denial> {
    meter
        .checkpoint(work)
        .map_err(|stop| Denial::Work(worth_execution::WorkCeilingDenial::Stopped(stop)))
}
fn reserve_basis_copy(
    read: &RelationalPinnedRead,
    allocation: ExecutionAllocationPolicy<'_, '_>,
    meter: &mut MapKernelContext<'_, '_>,
) -> Result<Option<worth_execution::ExecutionMemoryReservation>, Denial> {
    let bytes = read.pinned_basis().branch_id().0.len() as u64;
    charge(meter, bytes + 1)?;
    match allocation {
        worth_execution::ExecutionAllocationPolicy::Execution(lease) => lease
            .reserve_memory(bytes)
            .map(Some)
            .map_err(Denial::Memory),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation => Ok(None),
    }
}
