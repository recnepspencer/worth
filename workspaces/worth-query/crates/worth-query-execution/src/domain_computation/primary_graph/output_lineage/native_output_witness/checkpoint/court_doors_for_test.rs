//! Sealed test doors over this module's private members.
use super::*;

impl SealedNativeOutputWitness {
    #[doc(hidden)]
    pub fn checkpoint_facts_current_in_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relational: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        facts: &(impl FactSequence + ?Sized),
    ) -> Result<bool, super::super::super::invalidation::SettlementVerificationStop> {
        Self::checkpoint_facts_current_in(self, execution, relational, snapshot, facts)
    }
}

impl SealedNativeOutputWitness {
    #[doc(hidden)]
    pub fn from_checkpoint_facts_for_test(
        correspondence: &WorthQueryApplicationOutputCorrespondence,
        layout: &WorthQueryPrimaryGraphLayout,
        facts: &(impl FactSequence + ?Sized),
    ) -> Result<Option<Arc<OnceLock<Self>>>, CompanionPreflightStop> {
        Self::from_checkpoint_facts(correspondence, layout, facts)
    }
}
