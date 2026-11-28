use super::{CandidatePhaseLease, WorthQueryStagedProviderCandidate};
use worth_query_execution_runtime::facade::ports::SessionBoundReadsCommitInternalPort as _;

use crate::domain_computation::provider_session::{
    WorthQueryDecisionReadSetFailure, WorthQueryDecisionReadSetFreshnessOutcome,
    WorthQueryProviderSessionFailure, WorthQuerySessionCommitOutcome,
    WorthQueryStaleDecisionReadSet,
};

pub struct WorthQueryValidatedProviderCandidate<'run> {
    candidate: WorthQueryStagedProviderCandidate<'run>,
}

/// The same staged preparation used by ordinary commits, retained while a group
/// prepares its other writers. It has no independent publication authority.
pub struct WorthQueryPreparedProposedCommit<'run> {
    prepared: crate::domain_computation::provider_session::WorthQuerySessionPrepareOutcome<'run>,
    lease: CandidatePhaseLease,
}

#[derive(Debug)]
pub enum WorthQueryProviderCompareAndCommitOutcome {
    ProductStale(crate::domain_computation::WorthQueryProductStaleApplication),
    ProductUnpublished(
        worth_query_execution_runtime::facade::ports::OpaqueProductPublicationOutcome,
    ),
    ProductPreparationPending(
        worth_query_execution_runtime::facade::ports::OpaqueProductPublicationOutcome,
    ),
    ProductPreparationRefused(
        worth_query_execution_runtime::facade::ports::OpaqueProductPublicationOutcome,
    ),

    NoEffect(worth_runtime_world::facade::NoEffectCompositePublication),
    OutputAdoptionPrepared(
        worth_query_execution_runtime::facade::ports::PreparedOutputAdoptionPayload,
    ),
    Committed(WorthQueryCommittedProviderSession),
    Stale(WorthQueryStaleDecisionReadSet),
    Denied(WorthQueryProviderCompareAndCommitDenial),
    Deferred(crate::domain_computation::WorthQueryProviderSessionCommitDeferred),
    ControlStopped(crate::domain_computation::WorthQueryProviderSessionCommitControlStopped),
    Indeterminate(WorthQueryProviderSessionFailure),
}

/// Closed provider answer paired with the exact session that produced it.
///
/// Provider text remains available only as a descriptive projection. The
/// terminal binding, not text, selects the exact committed owner evidence.
#[derive(Debug)]
pub struct WorthQueryCommittedProviderSession {
    disposition:
        crate::domain_computation::provider_session::WorthQueryClosedProviderSessionDisposition,
}

impl WorthQueryCommittedProviderSession {
    fn from_disposition(
        disposition: crate::domain_computation::provider_session::WorthQueryClosedProviderSessionDisposition,
    ) -> Self {
        Self { disposition }
    }

    pub fn provider_description(
        &self,
    ) -> &crate::domain_computation::provider_session::WorthQueryProviderTerminalDescription {
        self.disposition.provider_description()
    }

    pub fn counters(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolCounters
    {
        self.disposition.counters()
    }

    pub const fn terminal_binding(
        &self,
    ) -> &crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding
    {
        self.disposition.terminal_binding()
    }
}

#[derive(Debug)]
pub enum WorthQueryProviderCompareAndCommitDenial {
    DecisionReadSet(WorthQueryDecisionReadSetFailure),
    ProviderSession(WorthQueryProviderSessionFailure),
}

impl<'run> WorthQueryValidatedProviderCandidate<'run> {
    pub(super) fn new(candidate: WorthQueryStagedProviderCandidate<'run>) -> Self {
        Self { candidate }
    }

    pub fn provider_session_terminal_binding(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding {
        self.candidate.staged.provider_session_terminal_binding()
    }

    pub fn compare_and_commit_with(
        self,
        commit: impl FnOnce(
            crate::domain_computation::provider_session::WorthQuerySessionPrepareOutcome<'run>,
        ) -> WorthQuerySessionCommitOutcome,
        compare: impl FnOnce(
            &crate::domain_computation::provider_session::WorthQuerySessionReadAuthority<'_>,
            crate::domain_computation::WorthQueryFreshDecisionReadSet,
        ) -> Result<
            WorthQueryDecisionReadSetFreshnessOutcome,
            crate::domain_computation::WorthQueryDecisionReadSetFailure,
        >,
    ) -> WorthQueryProviderCompareAndCommitOutcome {
        match self.compare_and_prepare(compare) {
            Ok(prepared) => prepared.commit_with(commit),
            Err(outcome) => outcome,
        }
    }

    pub fn compare_and_prepare(
        mut self,
        compare: impl FnOnce(
            &crate::domain_computation::provider_session::WorthQuerySessionReadAuthority<'_>,
            crate::domain_computation::WorthQueryFreshDecisionReadSet,
        ) -> Result<
            WorthQueryDecisionReadSetFreshnessOutcome,
            crate::domain_computation::WorthQueryDecisionReadSetFailure,
        >,
    ) -> Result<WorthQueryPreparedProposedCommit<'run>, WorthQueryProviderCompareAndCommitOutcome>
    {
        let fresh = self.candidate.read_set;
        let compared = compare(&self.candidate.staged.read_authority(), fresh);
        match compared {
            Ok(WorthQueryDecisionReadSetFreshnessOutcome::Stale(stale)) => {
                self.candidate.lease.discard();
                let _ = self.candidate.staged.abort();
                Err(WorthQueryProviderCompareAndCommitOutcome::Stale(stale))
            }
            Err(failure) => {
                self.candidate.lease.discard();
                let _ = self.candidate.staged.abort();
                Err(WorthQueryProviderCompareAndCommitOutcome::Denied(
                    WorthQueryProviderCompareAndCommitDenial::DecisionReadSet(failure),
                ))
            }
            Ok(WorthQueryDecisionReadSetFreshnessOutcome::Fresh(fresh)) => {
                self.candidate.read_set = fresh;
                self.prepare_fresh()
            }
        }
    }

    pub fn discard(mut self) {
        self.candidate.lease.discard();
        let _ = self.candidate.staged.abort();
    }

    fn prepare_fresh(
        mut self,
    ) -> Result<WorthQueryPreparedProposedCommit<'run>, WorthQueryProviderCompareAndCommitOutcome>
    {
        let prepared = match self.candidate.staged.prepare_for_commit() {
            Ok(prepared) => prepared,
            Err(failure) => {
                self.candidate.lease.discard();
                return Err(WorthQueryProviderCompareAndCommitOutcome::Denied(
                    WorthQueryProviderCompareAndCommitDenial::ProviderSession(failure),
                ));
            }
        };
        Ok(WorthQueryPreparedProposedCommit {
            prepared,
            lease: self.candidate.lease,
        })
    }
}

impl<'run> WorthQueryPreparedProposedCommit<'run> {
    pub fn terminal_binding(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding {
        self.prepared.terminal_binding()
    }

    pub fn discard(mut self) {
        self.lease.discard();
        // Dropping the actual prepared protocol aborts its live provider lease.
        drop(self.prepared);
    }
    pub fn commit_with(
        self,
        commit: impl FnOnce(
            crate::domain_computation::provider_session::WorthQuerySessionPrepareOutcome<'run>,
        ) -> WorthQuerySessionCommitOutcome,
    ) -> WorthQueryProviderCompareAndCommitOutcome {
        let Self {
            prepared,
            mut lease,
        } = self;
        match commit(prepared) {
            WorthQuerySessionCommitOutcome::ProductStale(stale) => {
                lease.discard();
                WorthQueryProviderCompareAndCommitOutcome::ProductStale(stale)
            }
            WorthQuerySessionCommitOutcome::ProductUnpublished(unpublished) => {
                lease.release_to_provider_resolution();
                WorthQueryProviderCompareAndCommitOutcome::ProductUnpublished(unpublished)
            }
            WorthQuerySessionCommitOutcome::ProductPreparationPending(owner) => {
                lease.release_to_provider_resolution();
                WorthQueryProviderCompareAndCommitOutcome::ProductPreparationPending(owner)
            }
            WorthQuerySessionCommitOutcome::ProductPreparationRefused(owner) => {
                lease.release_to_provider_resolution();
                WorthQueryProviderCompareAndCommitOutcome::ProductPreparationRefused(owner)
            }
            WorthQuerySessionCommitOutcome::NoEffect(no_effect) => {
                lease.discard();
                WorthQueryProviderCompareAndCommitOutcome::NoEffect(no_effect)
            }
            WorthQuerySessionCommitOutcome::OutputAdoptionPrepared(adoption) => {
                lease.release_to_provider_resolution();
                WorthQueryProviderCompareAndCommitOutcome::OutputAdoptionPrepared(adoption)
            }
            WorthQuerySessionCommitOutcome::Committed(disposition) => {
                lease.release_to_provider_resolution();
                WorthQueryProviderCompareAndCommitOutcome::Committed(
                    WorthQueryCommittedProviderSession::from_disposition(disposition),
                )
            }
            WorthQuerySessionCommitOutcome::CommitDenied(failure) => {
                lease.discard();
                WorthQueryProviderCompareAndCommitOutcome::Denied(
                    WorthQueryProviderCompareAndCommitDenial::ProviderSession(failure),
                )
            }
            WorthQuerySessionCommitOutcome::CommitRecoveryRequired(failure) => {
                lease.release_to_provider_resolution();
                WorthQueryProviderCompareAndCommitOutcome::Indeterminate(failure)
            }
            WorthQuerySessionCommitOutcome::CommitDeferred(deferred) => {
                lease.discard();
                WorthQueryProviderCompareAndCommitOutcome::Deferred(deferred)
            }
            WorthQuerySessionCommitOutcome::CommitControlStopped(stopped) => {
                lease.discard();
                WorthQueryProviderCompareAndCommitOutcome::ControlStopped(stopped)
            }
        }
    }
}
