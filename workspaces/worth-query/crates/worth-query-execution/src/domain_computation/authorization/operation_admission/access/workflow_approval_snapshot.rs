//! Bounded durable approval evidence extracted from the retained capability decision.

use super::*;
use crate::domain_computation::authorization::{
    WorkflowApprovalRequestEncodingDenial, MAXIMUM_PORTABLE_BYTES,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation) fn workflow_approval_request_snapshot(
        &self,
    ) -> Result<Option<String>, WorkflowApprovalRequestEncodingDenial> {
        self.authorization
            .as_ref()
            .and_then(WorthQueryRetainedAuthorizationDecisionFacts::capability_authorization)
            .map(|authorization| authorization.request().encode_workflow_approval_request())
            .transpose()
    }

    pub(in crate::domain_computation) fn workflow_approval_principal_snapshot(
        &self,
    ) -> Result<Option<String>, WorkflowApprovalRequestEncodingDenial> {
        self.authorization
            .as_ref()
            .and_then(WorthQueryRetainedAuthorizationDecisionFacts::capability_authorization)
            .map(|authorization| {
                let principal = authorization.principal().durable();
                if !principal.is_portable() {
                    return Err(WorkflowApprovalRequestEncodingDenial::NonPortableValue);
                }
                encode(&principal)
            })
            .transpose()
    }

    pub(in crate::domain_computation) fn workflow_approval_lineage_snapshot(
        &self,
    ) -> Result<Option<String>, WorkflowApprovalRequestEncodingDenial> {
        self.authorization
            .as_ref()
            .and_then(WorthQueryRetainedAuthorizationDecisionFacts::capability_authorization)
            .map(|authorization| encode(&authorization.decision().durable_lineage()))
            .transpose()
    }

    pub(in crate::domain_computation) fn workflow_approval_support_snapshot(
        &self,
    ) -> Result<Option<String>, WorkflowApprovalRequestEncodingDenial> {
        self.authorization
            .as_ref()
            .and_then(WorthQueryRetainedAuthorizationDecisionFacts::capability_authorization)
            .map(crate::domain_computation::authorization::encode_workflow_approval_support)
            .transpose()
    }

    pub(in crate::domain_computation) fn workflow_approval_dependencies_snapshot(
        &self,
    ) -> Result<Option<String>, WorkflowApprovalRequestEncodingDenial> {
        self.authorization
            .as_ref()
            .and_then(WorthQueryRetainedAuthorizationDecisionFacts::capability_authorization)
            .map(|authorization| {
                crate::domain_computation::authorization::encode_workflow_approval_dependencies(
                    authorization,
                )
            })
            .transpose()
    }
}

fn encode(value: &impl serde::Serialize) -> Result<String, WorkflowApprovalRequestEncodingDenial> {
    let encoded = serde_json::to_string(value)
        .map_err(|_| WorkflowApprovalRequestEncodingDenial::InvalidRequest)?;
    (encoded.len() <= MAXIMUM_PORTABLE_BYTES)
        .then_some(encoded)
        .ok_or(WorkflowApprovalRequestEncodingDenial::TooLarge)
}
