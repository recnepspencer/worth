use worth_runtime_bridge::facade::BridgeBoundExecutionBasis;

use super::WorthQueryRunningDirectRun;
use crate::domain_computation::{
    WorthQueryAdmittedProviderExecutionPlan, WorthQueryExecutionBoundOperationAuthority,
    WorthQueryExecutionProviderSession, WorthQueryExecutionResourceAttemptEvidence,
    WorthQueryProviderSessionFailure,
};

impl WorthQueryRunningDirectRun {
    pub fn admit_provider_execution_plan(
        &mut self,
        graph_authority: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    ) -> Result<WorthQueryAdmittedProviderExecutionPlan<'_>, WorthQueryProviderSessionFailure> {
        WorthQueryAdmittedProviderExecutionPlan::direct(self, graph_authority)
    }

    pub(crate) fn provider_plan_operation(&self) -> &WorthQueryExecutionBoundOperationAuthority {
        self.affinity.provider_plan_operation()
    }

    pub(crate) fn provider_plan_session(&self) -> &WorthQueryExecutionProviderSession {
        self.affinity.provider_plan_session()
    }

    pub(crate) fn provider_plan_resources(
        &self,
    ) -> (
        &worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan,
        &WorthQueryExecutionResourceAttemptEvidence,
    ) {
        self.affinity.provider_plan_resources()
    }

    pub(crate) fn provider_plan_bridge_basis(&self) -> &BridgeBoundExecutionBasis {
        &self.bridge_basis
    }
}
