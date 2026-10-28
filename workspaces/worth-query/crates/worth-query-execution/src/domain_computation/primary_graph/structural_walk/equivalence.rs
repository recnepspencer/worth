//! Full immutable body equality, shared only within one row/set interning pass.
use super::*;
use std::collections::BTreeMap;
use worth_execution::ExecutionRequest;

type Pair = (
    Arc<StructuralWalkObservation>,
    Arc<StructuralWalkObservation>,
    bool,
);
#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct StructuralWalkEquivalence {
    completed: BTreeMap<(usize, usize), Pair>,
    bytes: u64,
    _memory: Option<ExecutionMemoryReservation>,
}
impl StructuralWalkEquivalence {
    pub(in crate::domain_computation::primary_graph) fn equal(
        &mut self,
        left: &Arc<StructuralWalkObservation>,
        right: &Arc<StructuralWalkObservation>,
        execution: &ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
        control: &dyn Fn() -> Result<(), Cause>,
    ) -> Result<bool, Cause> {
        StructuralWalkObservation::checkpoint(context)?;
        control()?;
        if Arc::ptr_eq(left, right) {
            return Ok(true);
        }
        // A negative descriptor comparison cannot certify equality.
        if left.commitment() != right.commitment() {
            return Ok(false);
        }
        let key = (Arc::as_ptr(left) as usize, Arc::as_ptr(right) as usize);
        // Membership only: addresses never establish semantic order or iteration.
        let equal = if let Some((a, b, equal)) = self
            .completed
            .get(&key)
            .or_else(|| self.completed.get(&(key.1, key.0)))
        {
            if !(Arc::ptr_eq(a, left) && Arc::ptr_eq(b, right)
                || Arc::ptr_eq(a, right) && Arc::ptr_eq(b, left))
            {
                return Err(Cause::ExpectedIdentitiesNotCanonical);
            }
            *equal
        } else {
            let equal = left.same_truth_with_poll(right, || {
                StructuralWalkObservation::checkpoint(context)?;
                control()
            })?;
            memory::grow_tree::<((usize, usize), Pair)>(
                &mut self._memory,
                &mut self.bytes,
                self.completed.len(),
                execution,
                context.has_accounting(),
            )?;
            self.completed
                .insert(key, (Arc::clone(left), Arc::clone(right), equal));
            equal
        };
        StructuralWalkObservation::checkpoint(context)?;
        control()?;
        Ok(equal)
    }
}
