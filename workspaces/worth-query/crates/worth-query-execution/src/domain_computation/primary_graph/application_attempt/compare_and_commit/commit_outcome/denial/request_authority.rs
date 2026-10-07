//! The typed owner evidence a denial carries beside its kind.

use super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage,
};
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

use super::denial_cause::DenialCause;

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
            Some(DenialCause::CustomInvariant(_)) | Some(DenialCause::Execution(_)) | None => None,
        }
    }

    pub(super) const fn custom_invariant(
        &self,
    ) -> Option<&crate::domain_computation::WorthQueryCustomInvariantDenial> {
        match &self.cause {
            Some(DenialCause::CustomInvariant(denial)) => Some(denial),
            Some(DenialCause::RequestAuthority(_)) | Some(DenialCause::Execution(_)) | None => None,
        }
    }
}
