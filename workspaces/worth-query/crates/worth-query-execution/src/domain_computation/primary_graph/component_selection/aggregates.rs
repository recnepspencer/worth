//! Closed source forms share the original Native observation body owner.
use super::*;
use worth_relational::facade::schema::NativeMemberKey as Key;
#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum ComponentAggregateBodies {
    Physical(Vec<Arc<AggregateObservation>>),
    Derived(Vec<Arc<AggregateObservation<Key>>>),
}
impl ComponentAggregateBodies {
    pub(super) fn same_truth_with_poll<E>(
        &self,
        other: &Self,
        mut poll: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        poll()?;
        match (self, other) {
            (Self::Physical(a), Self::Physical(b)) if a.len() == b.len() => {
                for (a, b) in a.iter().zip(b) {
                    if !a.same_truth_with_poll(b, &mut poll)? {
                        return Ok(false);
                    }
                }
            }
            (Self::Derived(a), Self::Derived(b)) if a.len() == b.len() => {
                for (a, b) in a.iter().zip(b) {
                    if !a.same_truth_with_poll(b, &mut poll)? {
                        return Ok(false);
                    }
                }
            }
            _ => return Ok(false),
        }
        poll()?;
        Ok(true)
    }
}
