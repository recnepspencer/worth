//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::vocabulary::primary_graph::decision_reuse::PreparedDecisionReuseContext;

impl RetainedInputCutoffCandidate {
    #[doc(hidden)]
    pub fn eligible_for_reuse_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        fresh_key: &PreparedInputReuseKey,
        fresh_context: &PreparedDecisionReuseContext,
        matched_predecessors: Option<MatchedRequiredPredecessors<'_>>,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        owner: &SourceInvalidationOwner,
    ) -> Result<Option<VerifiedDependencies>, InputCutoffVerificationStop> {
        Self::eligible_for_reuse(
            self,
            execution,
            fresh_key,
            fresh_context,
            matched_predecessors,
            runtime,
            snapshot,
            selected,
            owner,
        )
    }
}

impl RetainedInputCutoffCandidate {
    #[doc(hidden)]
    pub fn verify_for_reuse_for_test<'selected>(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        fresh_key: PreparedInputReuseKey,
        fresh_context: PreparedDecisionReuseContext,
        matched_predecessors: Option<MatchedRequiredPredecessors<'_>>,
        runtime: &RelationalRuntime,
        basis: &'selected PreparedInputCutoffBasis<'_>,
        owner: &SourceInvalidationOwner,
    ) -> Result<InputCutoffDecision<'selected>, InputCutoffVerificationStop> {
        Self::verify_for_reuse(
            self,
            execution,
            fresh_key,
            fresh_context,
            matched_predecessors,
            runtime,
            basis,
            owner,
        )
    }
}
