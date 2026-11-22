//! Crate-boundary internal port for `WorthQueryInvariantApprovedProposedState`.
//! Never exported on `worth-query-execution::facade` or Host.

use super::commit_progression::{
    WorthQueryInvariantApprovedProposedState, WorthQueryPreparedProposedCommit,
    WorthQueryProviderCompareAndCommitOutcome,
};
use crate::domain_computation::provider_session::{
    WorthQueryDecisionReadSetFailure, WorthQueryDecisionReadSetFreshnessOutcome,
    WorthQueryFreshDecisionReadSet, WorthQuerySessionCommitOutcome,
    WorthQuerySessionPrepareOutcome, WorthQuerySessionReadAuthority,
};

pub trait InvariantApprovedProposedStateInternalPort<'run>: Sized {
    fn compare_and_commit_with(
        self,
        commit: impl FnOnce(WorthQuerySessionPrepareOutcome<'run>) -> WorthQuerySessionCommitOutcome,
        compare: impl FnOnce(
            &WorthQuerySessionReadAuthority<'_>,
            WorthQueryFreshDecisionReadSet,
        ) -> Result<
            WorthQueryDecisionReadSetFreshnessOutcome,
            WorthQueryDecisionReadSetFailure,
        >,
    ) -> WorthQueryProviderCompareAndCommitOutcome;

    fn compare_and_prepare(
        self,
        compare: impl FnOnce(
            &WorthQuerySessionReadAuthority<'_>,
            WorthQueryFreshDecisionReadSet,
        ) -> Result<
            WorthQueryDecisionReadSetFreshnessOutcome,
            WorthQueryDecisionReadSetFailure,
        >,
    ) -> Result<WorthQueryPreparedProposedCommit<'run>, WorthQueryProviderCompareAndCommitOutcome>;
}

impl<'run> InvariantApprovedProposedStateInternalPort<'run>
    for WorthQueryInvariantApprovedProposedState<'run>
{
    fn compare_and_commit_with(
        self,
        commit: impl FnOnce(WorthQuerySessionPrepareOutcome<'run>) -> WorthQuerySessionCommitOutcome,
        compare: impl FnOnce(
            &WorthQuerySessionReadAuthority<'_>,
            WorthQueryFreshDecisionReadSet,
        ) -> Result<
            WorthQueryDecisionReadSetFreshnessOutcome,
            WorthQueryDecisionReadSetFailure,
        >,
    ) -> WorthQueryProviderCompareAndCommitOutcome {
        Self::compare_and_commit_with(self, commit, compare)
    }

    fn compare_and_prepare(
        self,
        compare: impl FnOnce(
            &WorthQuerySessionReadAuthority<'_>,
            WorthQueryFreshDecisionReadSet,
        ) -> Result<
            WorthQueryDecisionReadSetFreshnessOutcome,
            WorthQueryDecisionReadSetFailure,
        >,
    ) -> Result<WorthQueryPreparedProposedCommit<'run>, WorthQueryProviderCompareAndCommitOutcome>
    {
        Self::compare_and_prepare(self, compare)
    }
}
