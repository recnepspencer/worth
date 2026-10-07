//! Custody of postconditions is distinct from comparable computation inputs.
use super::{ComputationSourceEvidence, CurrentComputation};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;
use std::{ops::Deref, sync::Arc};
type Fact = WorthQueryApplicationObservedFact;

/// Retained postconditions have no slice projection or Deref. Only the one
/// computation-current door can expose them for comparison.
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct RetainedSourceFacts {
    source: ComputationSourceEvidence,
    facts: Arc<[Fact]>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct ComparableSourceFacts {
    computation: CurrentComputation,
    facts: Arc<[Fact]>,
}
impl RetainedSourceFacts {
    pub(super) fn retain(source: ComputationSourceEvidence, facts: Arc<[Fact]>) -> Self {
        Self { source, facts }
    }
    pub(in crate::domain_computation::primary_graph) fn for_comparison(
        &self,
    ) -> Option<ComparableSourceFacts> {
        Some(ComparableSourceFacts {
            computation: self.source.certify_current()?,
            facts: Arc::clone(&self.facts),
        })
    }
    #[cfg(feature = "certification-invalidation-equivalence")]
    pub(super) fn same_binding(&self, other: &Self) -> bool {
        self.source == other.source && Arc::ptr_eq(&self.facts, &other.facts)
    }
    pub(super) fn source(&self) -> ComputationSourceEvidence {
        self.source
    }
    /// Owner-only custody/index posting. This is never a comparison proof.
    pub(super) fn postconditions(&self) -> &Arc<[Fact]> {
        &self.facts
    }
    pub(in crate::domain_computation::primary_graph) fn len(&self) -> usize {
        self.facts.len()
    }
    pub(in crate::domain_computation::primary_graph) fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn for_test(
        stale: bool,
        facts: Arc<[Fact]>,
    ) -> Self {
        Self::retain(ComputationSourceEvidence::for_test(stale), facts)
    }
}
impl ComparableSourceFacts {
    pub(in crate::domain_computation::primary_graph) fn computation(&self) -> CurrentComputation {
        self.computation
    }
    pub(in crate::domain_computation::primary_graph) fn facts(&self) -> &Arc<[Fact]> {
        &self.facts
    }
    pub(in crate::domain_computation::primary_graph) fn retained(&self) -> RetainedSourceFacts {
        RetainedSourceFacts::retain(self.computation.source(), Arc::clone(&self.facts))
    }
}
impl Deref for ComparableSourceFacts {
    type Target = [Fact];
    fn deref(&self) -> &Self::Target {
        &self.facts
    }
}
