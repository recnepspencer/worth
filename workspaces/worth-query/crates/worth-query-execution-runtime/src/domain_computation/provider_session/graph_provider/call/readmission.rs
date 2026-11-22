use std::sync::Arc;

use worth_query_installation::facade::WorthQueryExecutionResourceEnvelope;

use super::{WorthQueryGraphProviderCall, WorthQueryGraphProviderCallSpec};
use crate::domain_computation::operation_binding::WorthQueryExecutionBoundOperationAuthority;
use crate::domain_computation::provider_session::{
    WorthQueryExecutionProviderSession, WorthQueryExecutionResourceAttemptEvidence,
};
use crate::domain_computation::WorthQueryGraphCallBindingDenial;

pub struct WorthQueryGraphProviderCallReadmissionPlan {
    spec: WorthQueryGraphProviderCallSpec,
    resource_envelope: Arc<WorthQueryExecutionResourceEnvelope>,
}

impl WorthQueryGraphProviderCall {
    pub fn preflight_readmission(
        &self,
        binding: &WorthQueryExecutionBoundOperationAuthority,
        execution_resources: &WorthQueryExecutionResourceAttemptEvidence,
    ) -> Result<WorthQueryGraphProviderCallReadmissionPlan, WorthQueryGraphCallBindingDenial> {
        if self.spec.scope.binding_identity.as_ref() != binding.binding_identity() {
            return Err(WorthQueryGraphCallBindingDenial::BoundOperationAuthorityMismatch);
        }
        if !same_execution_basis(execution_resources, &self.execution_resources) {
            return Err(WorthQueryGraphCallBindingDenial::ExecutionBasisMismatch);
        }
        Ok(WorthQueryGraphProviderCallReadmissionPlan {
            spec: self.spec.clone(),
            resource_envelope: Arc::clone(&self.resource_envelope),
        })
    }
}

impl WorthQueryGraphProviderCallReadmissionPlan {
    pub fn mint(
        self,
        session: &WorthQueryExecutionProviderSession,
        execution_resources: &WorthQueryExecutionResourceAttemptEvidence,
    ) -> WorthQueryGraphProviderCall {
        WorthQueryGraphProviderCall::mint_validated(
            session,
            self.spec,
            execution_resources,
            self.resource_envelope,
        )
    }
}

fn same_execution_basis(
    candidate: &WorthQueryExecutionResourceAttemptEvidence,
    retained: &WorthQueryExecutionResourceAttemptEvidence,
) -> bool {
    candidate.admission_identity() == retained.admission_identity()
        && candidate.request_identity() == retained.request_identity()
        && candidate.strategy() == retained.strategy()
        && candidate.envelope_identity() == retained.envelope_identity()
        && candidate.support_snapshot_identity() == retained.support_snapshot_identity()
}
