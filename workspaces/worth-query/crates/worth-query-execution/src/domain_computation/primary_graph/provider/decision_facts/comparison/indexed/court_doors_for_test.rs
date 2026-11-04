//! Sealed test doors over this module's private members.
use super::*;

impl PreparedSelections {
    #[doc(hidden)]
    pub fn selectors_for_test(
        &self,
    ) -> &BTreeMap<(DerivedIndexId, KindId, AspectFieldLocator), PreparedEntityFieldLookup> {
        &self.selectors
    }
}

impl PreparedSelections {
    #[doc(hidden)]
    pub fn new_for_test() -> Self {
        Self::new()
    }
}

impl PreparedSelections {
    #[doc(hidden)]
    pub fn remains_equal_for_test(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        fact: &WorthQueryApplicationObservedFact,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        applied: Option<
            &worth_relational::facade::runtime::RelationalPinnedRead<
                worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
            >,
        >,
        control: StorageControl<'_, '_>,
    ) -> Result<bool, WorthQueryDecisionReadSetFailure> {
        Self::remains_equal(self, execution, fact, runtime, snapshot, applied, control)
    }
}
