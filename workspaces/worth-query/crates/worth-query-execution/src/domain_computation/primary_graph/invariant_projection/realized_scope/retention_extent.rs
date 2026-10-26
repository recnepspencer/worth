//! Fund only new physical keys; the snapshot, rather than the walk, owns them.
use super::WorthQueryRealizedProjectionScope;
use worth_execution::{ExecutionRequest, MapKernelContext, WorkCeilingDenial};
use worth_relational::facade::{
    runtime::PinnedStructuralWalkVisit, transactions::RelationalExecutionDenialCause as Cause,
};

impl WorthQueryRealizedProjectionScope {
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn admit_walk(
        &mut self,
        visits: &[PinnedStructuralWalkVisit],
        execution: &ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), Cause> {
        let mut main_new = 0_usize;
        let mut call_new = 0_usize;
        for visit in visits {
            checkpoint(context)?;
            if !self.entity_ids.contains(&visit.entity.entity_id) {
                main_new = main_new.checked_add(1).ok_or(Cause::ChargedBytesOverflow)?;
            }
            if self
                .call
                .as_ref()
                .is_some_and(|call| !call.contains(&visit.entity.entity_id))
            {
                call_new = call_new.checked_add(1).ok_or(Cause::ChargedBytesOverflow)?;
            }
        }
        let bytes = growth(self.entity_ids.len(), main_new)?
            .checked_add(growth(
                self.call.as_ref().map_or(0, |call| call.len()),
                call_new,
            )?)
            .and_then(|n| self.walk_scope_bytes.checked_add(n))
            .ok_or(Cause::ChargedBytesOverflow)?;
        if bytes != self.walk_scope_bytes {
            if self.walk_scope_memory.is_none() {
                self.walk_scope_memory = super::super::walk::memory::reserve(
                    execution,
                    context.has_accounting(),
                    bytes,
                )?;
            } else {
                super::super::walk::memory::resize(&mut self.walk_scope_memory, bytes)?;
            }
            self.walk_scope_bytes = bytes;
        }
        for visit in visits {
            checkpoint(context)?;
            self.record(visit.entity.entity_id);
        }
        Ok(())
    }
}

fn checkpoint(context: &mut MapKernelContext<'_, '_>) -> Result<(), Cause> {
    let admitted = if context.has_accounting() {
        context.checkpoint(1)
    } else {
        context.poll()
    };
    admitted.map_err(|stop| Cause::from(WorkCeilingDenial::Stopped(stop)))
}

fn growth(old: usize, new: usize) -> Result<u64, Cause> {
    if new == 0 {
        return Ok(0);
    }
    let after = old
        .checked_add(new)
        .and_then(|n| n.checked_add(1))
        .ok_or(Cause::ChargedBytesOverflow)?;
    let levels = usize::BITS - 1 - after.leading_zeros();
    let node = 11 * std::mem::size_of::<worth_relational::facade::identity::EntityId>()
        + 14 * std::mem::size_of::<usize>();
    new.checked_mul(levels as usize + 2)
        .and_then(|n| n.checked_mul(node))
        .and_then(|n| u64::try_from(n).ok())
        .ok_or(Cause::ChargedBytesOverflow)
}
