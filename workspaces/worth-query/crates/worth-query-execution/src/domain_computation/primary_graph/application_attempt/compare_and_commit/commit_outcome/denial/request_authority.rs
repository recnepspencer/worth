//! The typed owner evidence a denial carries beside its kind.

use super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage,
};
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

#[derive(Debug)]
pub(super) enum DenialCause {
    SourceRebase(crate::domain_computation::primary_graph::provider::PreparedRebaseDenial),
    InvariantExecution(crate::domain_computation::WorthQueryInvariantExecutionFailure),
    ProviderSession(crate::domain_computation::WorthQueryProviderSessionFailure),
    DecisionReadSet(crate::domain_computation::WorthQueryDecisionReadSetFailure),
    /// The request's own authorization stopped the commit: its security
    /// basis on the branch, or what it may do there.
    RequestAuthority(WorthQueryOperationAuthorizationDenialKind),
}

impl WorthQueryApplicationCommitDenial {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn provider_session_denied(
        failure: crate::domain_computation::WorthQueryProviderSessionFailure,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProviderRejected,
            stage: WorthQueryApplicationCommitDenialStage::ProviderCommit,
            detail: Some(failure.detail().into()),
            cause: Some(DenialCause::ProviderSession(failure)),
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn with_provider_session_failure(
        mut self,
        failure: crate::domain_computation::WorthQueryProviderSessionFailure,
    ) -> Self {
        self.cause = Some(DenialCause::ProviderSession(failure));
        self
    }

    /// Complete native diagnostics for a refusal proven to precede publication.
    pub fn native_preparation_error(
        &self,
    ) -> Option<&worth_relational::facade::mvcc::TransactionCommitError> {
        match &self.cause {
            Some(DenialCause::ProviderSession(failure)) => failure.native_preparation_error(),
            _ => None,
        }
    }

    /// The request was refused authorization at `stage`. The refusal is the
    /// request's own, so whoever shares the attempt's subject is not refused.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn request_authority_denied(
        stage: WorthQueryApplicationCommitDenialStage,
        denial: &WorthQueryOperationAuthorizationDenial,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProviderRejected,
            stage,
            detail: Some(denial.to_string().into()),
            cause: Some(DenialCause::RequestAuthority(denial.kind())),
        }
    }

    /// Why the request was refused authorization, when that refused the commit.
    pub(in crate::domain_computation::primary_graph) const fn request_authority(
        &self,
    ) -> Option<WorthQueryOperationAuthorizationDenialKind> {
        match &self.cause {
            Some(DenialCause::RequestAuthority(kind)) => Some(*kind),
            _ => None,
        }
    }

    pub(super) fn custom_invariant(
        &self,
    ) -> Option<&crate::domain_computation::WorthQueryCustomInvariantDenial> {
        match &self.cause {
            Some(DenialCause::InvariantExecution(failure)) => failure.custom_invariant_denial(),
            _ => None,
        }
    }
}

impl WorthQueryApplicationCommitDenial {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn source_rebase_denied(
        denial: crate::domain_computation::primary_graph::provider::PreparedRebaseDenial,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProviderRejected,
            stage: WorthQueryApplicationCommitDenialStage::ProposalBinding,
            detail: None,
            cause: Some(DenialCause::SourceRebase(denial)),
        }
    }
    /// Exact physical owner refusal; not logical count policy or graph authority.
    pub fn allocation_denial(&self) -> Option<&worth_execution::ExecutionAllocationDenial> {
        match &self.cause {
            Some(DenialCause::ProviderSession(failure)) => failure.allocation_denial(),
            Some(DenialCause::SourceRebase(crate::domain_computation::primary_graph::provider::PreparedRebaseDenial::Retention(crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial::Allocation(denial)))) => Some(denial),
            _ => None,
        }
    }
    pub fn source_rebase_interruption(
        &self,
    ) -> Option<worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption>
    {
        match &self.cause { Some(DenialCause::SourceRebase(crate::domain_computation::primary_graph::provider::PreparedRebaseDenial::Retention(crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial::RequestInterruption(stop)))) => Some(*stop), _ => None }
    }
    pub fn source_rebase_temporary_reservation_error(
        &self,
    ) -> Option<&std::collections::TryReserveError> {
        match &self.cause {
            Some(DenialCause::SourceRebase(
                crate::domain_computation::primary_graph::provider::PreparedRebaseDenial::Temporary(
                    error,
                ),
            )) => Some(error),
            _ => None,
        }
    }
}
