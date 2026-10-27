//! Pair memoization belongs to the same immutable-source interning extent as walks.
use super::*;
use crate::domain_computation::primary_graph::component_selection::ComponentSelection;
impl SourceReadEquivalence {
    pub(in crate::domain_computation::primary_graph) fn equal_component(
        &mut self,
        left: &Arc<ComponentSelection>,
        right: &Arc<ComponentSelection>,
        context: &mut MapKernelContext<'_, '_>,
        control: &dyn Fn() -> Result<(), Cause>,
    ) -> Result<bool, Cause> {
        AggregateObservation::checkpoint(context)?;
        control()?;
        if Arc::ptr_eq(left, right) {
            return Ok(true);
        }
        if left.commitment() != right.commitment() {
            return Ok(false);
        }
        let key = (Arc::as_ptr(left) as usize, Arc::as_ptr(right) as usize);
        if let Some((a, b, equal)) = self
            .components
            .get(&key)
            .or_else(|| self.components.get(&(key.1, key.0)))
        {
            if !(Arc::ptr_eq(a, left) && Arc::ptr_eq(b, right)
                || Arc::ptr_eq(a, right) && Arc::ptr_eq(b, left))
            {
                return Err(Cause::ExpectedIdentitiesNotCanonical);
            }
            return Ok(*equal);
        }
        let equal = left.same_truth_with_poll(right, || {
            AggregateObservation::checkpoint(context)?;
            control()
        })?;
        self.components
            .insert(key, (Arc::clone(left), Arc::clone(right), equal));
        AggregateObservation::checkpoint(context)?;
        control()?;
        Ok(equal)
    }
}
