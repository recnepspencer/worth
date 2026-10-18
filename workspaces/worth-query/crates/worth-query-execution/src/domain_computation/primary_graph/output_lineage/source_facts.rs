//! Custody of postconditions is distinct from comparable computation inputs.
use super::{ComputationSourceEvidence, CurrentComputation};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;
use std::ops::Deref;
#[cfg(test)]
use std::sync::Arc;
type Fact = WorthQueryApplicationObservedFact;
use crate::domain_computation::primary_graph::application_checkpoint::facts::{
    FactPayload, FactSequence,
};

/// Retained postconditions have no slice projection or Deref. Only the one
/// computation-current door can expose them for comparison.
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct RetainedSourceFacts {
    source: ComputationSourceEvidence,
    facts: FactPayload,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct ComparableSourceFacts {
    computation: CurrentComputation,
    facts: FactPayload,
}
impl RetainedSourceFacts {
    pub(super) fn retain(source: ComputationSourceEvidence, facts: impl Into<FactPayload>) -> Self {
        Self {
            source,
            facts: facts.into(),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn for_comparison(
        &self,
    ) -> Option<ComparableSourceFacts> {
        Some(ComparableSourceFacts {
            computation: self.source.certify_current()?,
            facts: self.facts.clone(),
        })
    }
    pub(super) fn same_binding(&self, other: &Self) -> bool {
        self.source == other.source && self.facts.same_backing(&other.facts)
    }
    pub(super) fn source(&self) -> ComputationSourceEvidence {
        self.source
    }
    /// Owner-only custody/index posting. This is never a comparison proof.
    pub(super) fn postconditions(&self) -> &FactPayload {
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
    pub(in crate::domain_computation::primary_graph) fn facts(&self) -> &FactPayload {
        &self.facts
    }
    pub(in crate::domain_computation::primary_graph) fn retained(&self) -> RetainedSourceFacts {
        RetainedSourceFacts::retain(self.computation.source(), self.facts.clone())
    }
}
impl Deref for ComparableSourceFacts {
    type Target = FactPayload;
    fn deref(&self) -> &Self::Target {
        &self.facts
    }
}

impl FactSequence for ComparableSourceFacts {
    fn get(&self, ordinal: usize) -> Option<&Fact> {
        self.facts().get(ordinal)
    }
    fn len(&self) -> usize {
        self.facts().len()
    }
    fn retained_storage_bytes(&self) -> Option<u64> {
        self.facts().retained_storage_bytes()
    }
    fn iter(&self) -> impl DoubleEndedIterator<Item = &Fact> + ExactSizeIterator + Clone {
        self.facts().iter()
    }
}
