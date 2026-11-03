use std::sync::Arc;

use super::graph_provider::{
    WorthQueryGraphCallBindingDenial, WorthQueryGraphCommitCall, WorthQueryGraphCommitCallRequest,
    WorthQueryGraphProviderCall, WorthQueryGraphProviderCallRequest,
};
use super::{
    WorthQueryClosedExecutionAttemptIdentity, WorthQueryExecutionAttemptIdentity,
    WorthQueryExecutionResourceAttemptEvidence,
};
use crate::domain_computation::operation_binding::WorthQueryExecutionBoundOperationAuthority;

#[derive(Debug)]
pub struct WorthQueryExecutionProviderSession {
    identity: WorthQueryExecutionProviderSessionIdentity,
    attempt_identity: WorthQueryClosedExecutionAttemptIdentity,
    binding_authority: Arc<WorthQueryExecutionBoundOperationAuthority>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct WorthQueryExecutionProviderSessionIdentity(Arc<str>);

impl WorthQueryExecutionProviderSession {
    pub(super) fn mint(
        attempt_identity: &WorthQueryExecutionAttemptIdentity,
        binding_authority: &WorthQueryExecutionBoundOperationAuthority,
    ) -> Self {
        Self {
            identity: WorthQueryExecutionProviderSessionIdentity(Arc::from(
                attempt_identity.as_str(),
            )),
            attempt_identity: attempt_identity.closed_identity(),
            binding_authority: Arc::new(binding_authority.clone()),
        }
    }

    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    pub fn attempt_identity(&self) -> &str {
        self.attempt_identity.as_str()
    }

    pub(in crate::domain_computation) fn closed_identity(
        &self,
    ) -> WorthQueryExecutionProviderSessionIdentity {
        self.identity.clone()
    }

    pub(in crate::domain_computation) fn closed_attempt_identity(
        &self,
    ) -> WorthQueryClosedExecutionAttemptIdentity {
        self.attempt_identity.clone()
    }

    pub(super) fn retain_binding_authority(
        &self,
    ) -> Arc<WorthQueryExecutionBoundOperationAuthority> {
        Arc::clone(&self.binding_authority)
    }

    pub(crate) fn binding_authority(&self) -> &WorthQueryExecutionBoundOperationAuthority {
        &self.binding_authority
    }

    pub fn bind_graph_provider_call(
        &self,
        graph_authority: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
        request: WorthQueryGraphProviderCallRequest,
        execution_resources: &WorthQueryExecutionResourceAttemptEvidence,
        resource_envelope: Arc<
            worth_query_installation::facade::WorthQueryExecutionResourceEnvelope,
        >,
    ) -> Result<WorthQueryGraphProviderCall, WorthQueryGraphCallBindingDenial> {
        if request.execution_snapshot_identity().is_none() {
            return Err(WorthQueryGraphCallBindingDenial::ExecutionBasisMismatch);
        }
        if !self.binding_authority.admits_graph_call(
            request.stage_identity(),
            graph_authority,
            request.kind(),
        ) {
            return Err(WorthQueryGraphCallBindingDenial::BoundOperationAuthorityMismatch);
        }
        let spec = request.into_spec(&self.binding_authority, graph_authority);
        WorthQueryGraphProviderCall::mint(self, spec, execution_resources, resource_envelope)
    }

    pub fn bind_graph_commit_call(
        &self,
        graph_authorities: &[&worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority],
        request: WorthQueryGraphCommitCallRequest,
        execution_resources: &WorthQueryExecutionResourceAttemptEvidence,
        resource_envelope: Arc<
            worth_query_installation::facade::WorthQueryExecutionResourceEnvelope,
        >,
    ) -> Result<WorthQueryGraphCommitCall, WorthQueryGraphCallBindingDenial> {
        if !self
            .binding_authority
            .admits_commit_call(request.stage_identity(), graph_authorities)
        {
            return Err(WorthQueryGraphCallBindingDenial::BoundOperationAuthorityMismatch);
        }
        let spec = request.into_spec(&self.binding_authority, graph_authorities);
        WorthQueryGraphCommitCall::mint(self, spec, execution_resources, resource_envelope)
    }
}

impl WorthQueryExecutionProviderSessionIdentity {
    pub(in crate::domain_computation) fn as_str(&self) -> &str {
        &self.0
    }

    pub(in crate::domain_computation) fn description(&self) -> Arc<str> {
        Arc::clone(&self.0)
    }
}
