use super::{
    WorthQueryInvariantProgressionAuthority, WorthQueryProposedPostState,
    WorthQueryProposedStateInspection,
};
use crate::domain_computation::provider_session::{
    WorthQueryDecisionReadSetFailure, WorthQueryDecisionReadSetFreshnessOutcome,
    WorthQueryProviderSessionFailure, WorthQuerySessionCommitOrAbortOutcome,
    WorthQueryStaleDecisionReadSet,
};

pub struct WorthQueryInvariantApprovedProposedState<'run> {
    proposed: WorthQueryProposedPostState<'run>,
    progression: WorthQueryInvariantProgressionAuthority,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProviderCommitAdmissionDenial {
    ForeignInvariantProgression,
}

#[derive(Debug)]
pub enum WorthQueryProviderCompareAndCommitOutcome {
    ProductStale(crate::domain_computation::WorthQueryProductStaleApplication),
    ProductUnpublished(crate::domain_computation::WorthQueryProductUnpublishedApplication),
    NoEffect(worth_runtime_world::facade::NoEffectCompositePublication),
    Committed(WorthQueryCommittedProviderSession),
    Stale(WorthQueryStaleDecisionReadSet),
    Denied(WorthQueryProviderCompareAndCommitDenial),
    Deferred(crate::domain_computation::WorthQueryProviderSessionCommitDeferred),
    ControlStopped(crate::domain_computation::WorthQueryProviderSessionCommitControlStopped),
    SettlementDeferred(crate::domain_computation::WorthQueryProviderSessionSettlementDeferred),
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

    pub(in crate::domain_computation) const fn terminal_binding(
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

impl<'run> WorthQueryProposedStateInspection<'run> {
    pub fn bind_invariant_progression(
        self,
        progression: WorthQueryInvariantProgressionAuthority,
    ) -> Result<
        WorthQueryInvariantApprovedProposedState<'run>,
        (
            WorthQueryProviderCommitAdmissionDenial,
            WorthQueryProposedStateInspection<'run>,
        ),
    > {
        let attempt = &self.proposed.attempt;
        let plan = attempt.staged.plan();
        if !progression.belongs_to(
            plan.provider_identity(),
            plan.provider_generation(),
            attempt.staged.provisional_binding_identity(),
            plan.basis_identity(),
            self.proposed.identity(),
            self.proposed.generation(),
        ) {
            return Err((
                WorthQueryProviderCommitAdmissionDenial::ForeignInvariantProgression,
                self,
            ));
        }
        Ok(WorthQueryInvariantApprovedProposedState {
            proposed: self.proposed,
            progression,
        })
    }
}

impl WorthQueryInvariantApprovedProposedState<'_> {
    pub(in crate::domain_computation) fn provider_session_terminal_binding(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding {
        self.proposed
            .attempt
            .staged
            .provider_session_terminal_binding()
    }

    pub fn invariant_receipt_identities(&self) -> &[std::sync::Arc<str>] {
        self.progression.receipt_identities()
    }

    pub fn compare_and_commit(mut self) -> WorthQueryProviderCompareAndCommitOutcome {
        let fresh = self.proposed.attempt.read_set;
        let compared = self
            .proposed
            .attempt
            .staged
            .read_authority()
            .recompare_fresh_decision_read_set(fresh);
        match compared {
            Ok(WorthQueryDecisionReadSetFreshnessOutcome::Stale(stale)) => {
                let _ = self.proposed.attempt.overlay.discard();
                let _ = self.proposed.attempt.staged.abort();
                WorthQueryProviderCompareAndCommitOutcome::Stale(stale)
            }
            Err(failure) => {
                let _ = self.proposed.attempt.overlay.discard();
                let _ = self.proposed.attempt.staged.abort();
                WorthQueryProviderCompareAndCommitOutcome::Denied(
                    WorthQueryProviderCompareAndCommitDenial::DecisionReadSet(failure),
                )
            }
            Ok(WorthQueryDecisionReadSetFreshnessOutcome::Fresh(fresh)) => {
                self.proposed.attempt.read_set = fresh;
                self.commit_fresh()
            }
        }
    }

    pub fn discard(mut self) {
        let _ = self.proposed.attempt.overlay.discard();
        let _ = self.proposed.attempt.staged.abort();
    }

    fn commit_fresh(mut self) -> WorthQueryProviderCompareAndCommitOutcome {
        let prepared = match self.proposed.attempt.staged.prepare_for_commit() {
            Ok(prepared) => prepared,
            Err(failure) => {
                let _ = self.proposed.attempt.overlay.discard();
                return WorthQueryProviderCompareAndCommitOutcome::Denied(
                    WorthQueryProviderCompareAndCommitDenial::ProviderSession(failure),
                );
            }
        };
        match prepared.commit() {
            WorthQuerySessionCommitOrAbortOutcome::ProductStale(stale) => {
                let _ = self.proposed.attempt.overlay.discard();
                WorthQueryProviderCompareAndCommitOutcome::ProductStale(stale)
            }
            WorthQuerySessionCommitOrAbortOutcome::ProductUnpublished(unpublished) => {
                self.proposed
                    .attempt
                    .overlay
                    .release_to_provider_resolution();
                WorthQueryProviderCompareAndCommitOutcome::ProductUnpublished(unpublished)
            }
            WorthQuerySessionCommitOrAbortOutcome::NoEffect(no_effect) => {
                let _ = self.proposed.attempt.overlay.discard();
                WorthQueryProviderCompareAndCommitOutcome::NoEffect(no_effect)
            }
            WorthQuerySessionCommitOrAbortOutcome::Committed(disposition) => {
                self.proposed
                    .attempt
                    .overlay
                    .release_to_provider_resolution();
                WorthQueryProviderCompareAndCommitOutcome::Committed(
                    WorthQueryCommittedProviderSession::from_disposition(disposition),
                )
            }
            WorthQuerySessionCommitOrAbortOutcome::CommitRecoveryRequired(failure) => {
                self.proposed
                    .attempt
                    .overlay
                    .release_to_provider_resolution();
                WorthQueryProviderCompareAndCommitOutcome::Indeterminate(failure)
            }
            WorthQuerySessionCommitOrAbortOutcome::CommitDeferred(deferred) => {
                let _ = self.proposed.attempt.overlay.discard();
                WorthQueryProviderCompareAndCommitOutcome::Deferred(deferred)
            }
            WorthQuerySessionCommitOrAbortOutcome::CommitControlStopped(stopped) => {
                let _ = self.proposed.attempt.overlay.discard();
                WorthQueryProviderCompareAndCommitOutcome::ControlStopped(stopped)
            }
            WorthQuerySessionCommitOrAbortOutcome::CommitSettlementDeferred(deferred) => {
                self.proposed
                    .attempt
                    .overlay
                    .release_to_provider_resolution();
                WorthQueryProviderCompareAndCommitOutcome::SettlementDeferred(deferred)
            }
            WorthQuerySessionCommitOrAbortOutcome::Aborted(_)
            | WorthQuerySessionCommitOrAbortOutcome::AbortRecoveryRequired(_) => {
                unreachable!("commit transition cannot produce an abort outcome")
            }
        }
    }
}
