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
    InvariantExecution(crate::domain_computation::WorthQueryInvariantExecutionFailure),
    /// The request's own authorization stopped the commit: its security
    /// basis on the branch, or what it may do there.
    RequestAuthority(WorthQueryOperationAuthorizationDenialKind),
}

impl WorthQueryApplicationCommitDenial {
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
