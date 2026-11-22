//! Bounded durable approval evidence extracted from the retained capability decision.

use super::*;
use crate::domain_computation::authorization::{
    WorkflowApprovalRequestEncodingDenial, MAXIMUM_PORTABLE_BYTES,
};
use crate::domain_computation::primary_graph::workflow::actor::encoding::{self, FundedActorText};

impl WorthQueryAdmittedOperation {
    /// Original live native admission identity, across ordinary and capability lanes.
    pub fn signed_creation_principal(
        &self,
    ) -> Option<
        &std::sync::Arc<
            worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity,
        >,
    > {
        self.data
            .authorization
            .as_ref()
            .map(WorthQueryRetainedAuthorizationDecisionFacts::signing_principal)
    }
    /// Live admission provenance is distinct from the serializable snapshots below.
    pub fn workflow_approval_signing_principal(
        &self,
    ) -> Option<
        &std::sync::Arc<
            worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity,
        >,
    > {
        self.data
            .authorization
            .as_ref()
            .and_then(WorthQueryRetainedAuthorizationDecisionFacts::capability_authorization)
            .map(|authorization| authorization.principal().external_identity())
    }

    pub fn workflow_approval_request_snapshot(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<FundedActorText>, WorkflowApprovalRequestEncodingDenial> {
        self.data
            .authorization
            .as_ref()
            .and_then(WorthQueryRetainedAuthorizationDecisionFacts::capability_authorization)
            .map(|authorization| {
                authorization
                    .request()
                    .encode_workflow_approval_request(execution)
            })
            .transpose()
    }

    pub fn workflow_approval_principal_snapshot(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<FundedActorText>, WorkflowApprovalRequestEncodingDenial> {
        self.data
            .authorization
            .as_ref()
            .and_then(WorthQueryRetainedAuthorizationDecisionFacts::capability_authorization)
            .map(|authorization| {
                let principal = authorization.principal().durable_view();
                if !authorization.principal().durable_is_portable() {
                    return Err(WorkflowApprovalRequestEncodingDenial::NonPortableValue);
                }
                encoding::encode(execution, &principal, MAXIMUM_PORTABLE_BYTES)
            })
            .transpose()
    }

    pub fn workflow_approval_lineage_snapshot(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<FundedActorText>, WorkflowApprovalRequestEncodingDenial> {
        self.data
            .authorization
            .as_ref()
            .and_then(WorthQueryRetainedAuthorizationDecisionFacts::capability_authorization)
            .map(|authorization| {
                encoding::encode(
                    execution,
                    &authorization.decision().durable_lineage_view(),
                    MAXIMUM_PORTABLE_BYTES,
                )
            })
            .transpose()
    }

    pub fn workflow_approval_support_snapshot(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<FundedActorText>, WorkflowApprovalRequestEncodingDenial> {
        self.data
            .authorization
            .as_ref()
            .and_then(WorthQueryRetainedAuthorizationDecisionFacts::capability_authorization)
            .map(|authorization| {
                crate::domain_computation::authorization::encode_workflow_approval_support(
                    execution,
                    authorization,
                )
            })
            .transpose()
    }

    pub fn workflow_approval_dependencies_snapshot(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<FundedActorText>, WorkflowApprovalRequestEncodingDenial> {
        self.data
            .authorization
            .as_ref()
            .and_then(WorthQueryRetainedAuthorizationDecisionFacts::capability_authorization)
            .map(|authorization| {
                crate::domain_computation::authorization::encode_workflow_approval_dependencies(
                    execution,
                    authorization,
                )
            })
            .transpose()
    }
}
