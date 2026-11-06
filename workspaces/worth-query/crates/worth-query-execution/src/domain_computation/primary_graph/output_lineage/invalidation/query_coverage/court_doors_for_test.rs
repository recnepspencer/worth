//! Sealed test doors over this module's private members.
use super::*;

impl QueryReadBinding {
    #[doc(hidden)]
    pub fn dirty_for_test(&self) -> &Option<usize> {
        &self.dirty
    }
}

impl QueryReadBinding {
    #[doc(hidden)]
    pub fn identity_for_test(&self) -> &Weak<QueryReadIdentity> {
        &self.identity
    }
}

impl QueryReadBinding {
    #[doc(hidden)]
    pub fn projection_for_test(&self) -> &QueryReadBacking {
        &self.projection
    }
}

impl SourceInvalidationOwner {
    #[doc(hidden)]
    pub fn query_facts_clean_at_for_test<'selected>(
        &self,
        coverage: &'selected QueryFactCoverage,
        selected: &'selected PositionedRelationalSnapshot,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Option<CleanQueryFacts<'selected>>, QueryCoverageStop> {
        Self::query_facts_clean_at(self, coverage, selected, policy)
    }
}

impl SourceInvalidationOwner {
    #[doc(hidden)]
    pub fn retain_query_fact_coverage_for_test(
        self: &Arc<Self>,
        basis: PositionedRelationalSnapshot,
        facts: &[WorthQueryApplicationObservedFact],
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Option<Arc<QueryFactCoverage>>, QueryCoverageStop> {
        Self::retain_query_fact_coverage(self, basis, facts, policy)
    }
}

impl QueryFactCoverage {
    #[doc(hidden)]
    pub fn projection_for_test(&self) -> &Arc<QueryReadProjection> {
        &self.projection
    }
}

impl QueryFactCoverage {
    #[doc(hidden)]
    pub fn slot_for_test(&self) -> &Mutex<Option<u64>> {
        &self.slot
    }
}
