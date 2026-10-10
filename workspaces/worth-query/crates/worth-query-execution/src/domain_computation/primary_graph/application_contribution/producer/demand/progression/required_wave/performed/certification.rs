//! A certification can be consumed only before another publication moves it.
use super::PerformedMembers;

/// No accessor exposes a stale decision. Reinspection obtains a new value.
pub(in super::super) struct CertifiedDecision<T> {
    generation: u64,
    value: T,
}
impl<T> CertifiedDecision<T> {
    pub(in super::super) fn into_current(self, owner: &PerformedMembers) -> Option<T> {
        (self.generation == owner.publication_generation).then_some(self.value)
    }
}
impl PerformedMembers {
    pub(in super::super) fn certify<T>(&self, value: T) -> CertifiedDecision<T> {
        CertifiedDecision {
            generation: self.publication_generation,
            value,
        }
    }
}
