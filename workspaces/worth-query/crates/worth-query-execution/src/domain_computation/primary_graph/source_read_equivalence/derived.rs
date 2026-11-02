//! One pass memo retains exact full derived comparisons and original custody.
use super::*;
use crate::domain_computation::primary_graph::derived_source::DerivedSourceObservation;
pub(super) type Pair = (
    Arc<DerivedSourceObservation>,
    Arc<DerivedSourceObservation>,
    bool,
);
impl SourceReadEquivalence {
    #[expect(
        clippy::disallowed_methods,
        reason = "Addresses serve live allocation lookup and custody; canonical order uses declared identity"
    )]
    pub(in crate::domain_computation::primary_graph) fn equal_derived(
        &mut self,
        left: &Arc<DerivedSourceObservation>,
        right: &Arc<DerivedSourceObservation>,
        execution: &ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
        control: &dyn Fn() -> Result<(), Cause>,
    ) -> Result<bool, Cause> {
        DerivedSourceObservation::checkpoint(context)?;
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
            .derived
            .get(&key)
            .or_else(|| self.derived.get(&(key.1, key.0)))
        {
            if !(Arc::ptr_eq(a, left) && Arc::ptr_eq(b, right)
                || Arc::ptr_eq(a, right) && Arc::ptr_eq(b, left))
            {
                return Err(Cause::ExpectedIdentitiesNotCanonical);
            }
            *equal
        } else {
            let equal = left.same_truth_with_poll(right, || {
                DerivedSourceObservation::checkpoint(context)?;
                control()
            })?;
            memory::grow_tree::<((usize, usize), Pair)>(
                &mut self._memory,
                &mut self.bytes,
                self.derived.len(),
                execution,
                context.has_accounting(),
            )?;
            self.derived
                .insert(key, (Arc::clone(left), Arc::clone(right), equal));
            equal
        };
        DerivedSourceObservation::checkpoint(context)?;
        control()?;
        Ok(equal)
    }
}
