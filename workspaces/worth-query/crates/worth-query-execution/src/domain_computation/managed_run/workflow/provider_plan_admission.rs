use std::sync::Arc;

use worth_runtime_bridge::facade::BridgeBoundExecutionBasis;

use super::WorthQueryRunningWorkflowRun;
use crate::domain_computation::{
    WorthQueryAdmittedProviderExecutionPlan, WorthQueryExecutionBoundOperationAuthority,
    WorthQueryExecutionProviderSession, WorthQueryExecutionResourceAttemptEvidence,
    WorthQueryProviderSessionFailure,
};
use crate::domain_computation::vocabulary::managed_run::provider_plan_admission::WorthQueryWorkflowProviderPlanPermit;



impl WorthQueryRunningWorkflowRun {
    pub fn admit_stage_provider_execution_plan(
        &mut self,
        stage_identity: &str,
        graph_authority: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    ) -> Result<WorthQueryAdmittedProviderExecutionPlan<'_>, WorthQueryProviderSessionFailure> {
        WorthQueryAdmittedProviderExecutionPlan::workflow_stage(
            self,
            stage_identity,
            graph_authority,
            &WorthQueryWorkflowProviderPlanPermit::mint(),
        )
    }

    pub(in crate::domain_computation) fn provider_plan_operation(
        &self,
        _owner: &WorthQueryWorkflowProviderPlanPermit,
    ) -> &WorthQueryExecutionBoundOperationAuthority {
        self.affinity.provider_plan_operation(_owner)
    }

    pub(in crate::domain_computation) fn provider_plan_session(
        &self,
        _owner: &WorthQueryWorkflowProviderPlanPermit,
    ) -> &WorthQueryExecutionProviderSession {
        self.affinity.provider_plan_session(_owner)
    }

    pub(in crate::domain_computation) fn provider_plan_stage_resources(
        &self,
        stage_identity: &str,
        _owner: &WorthQueryWorkflowProviderPlanPermit,
    ) -> Option<(
        Arc<worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan>,
        WorthQueryExecutionResourceAttemptEvidence,
    )>{
        self.affinity
            .managed_stage_resources_and_evidence(stage_identity)
    }

    pub(in crate::domain_computation) fn provider_plan_bridge_basis(
        &self,
        _owner: &WorthQueryWorkflowProviderPlanPermit,
    ) -> &BridgeBoundExecutionBasis {
        &self.bridge_basis
    }
}
