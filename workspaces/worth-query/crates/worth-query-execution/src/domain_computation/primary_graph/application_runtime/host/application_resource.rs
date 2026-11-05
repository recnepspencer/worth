//! The runtime composes `application_resource`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::application_resource::host::ApplicationResourceHost;

impl<Schema: ApplicationSchema> ApplicationResourceHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn application_resource_host(&self) -> &dyn ApplicationResourceHost<Schema> {
        self
    }
    fn bridge(&self) -> &crate::domain_computation::primary_graph::managed_bridge::WorthQueryInstalledApplicationBridge{
        &self.runtime.bridge
    }
    fn installed_resources(&self) -> &crate::domain_computation::primary_graph::application_resource::installation::InstalledResourceRegistry<Schema>{
        &self.installed_resources
    }
    fn on_branch(
        &self,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> crate::domain_computation::primary_graph::product_operation::WorthQueryProductEntry<
        '_,
        Schema,
    > {
        Self::on_branch(self, branch)
    }
    fn select_application_read_observation_with_request(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, observation: &crate::domain_computation::primary_graph::application_query::WorthQueryApplicationReadObservation) -> Result<crate::domain_computation::primary_graph::product_operation::WorthQuerySelectedProductOperation<'_, Schema>, crate::basis::WorthQueryProductBranchAdmissionDenial>{
        Self::select_application_read_observation_with_request(self, execution, observation)
    }
    fn refuse_completed_external_effect(&self, record: &crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord) -> Result<(), crate::domain_computation::application_aftermath::external_effect::WorthQueryTerminalEffectRefusal>{
        Self::refuse_completed_external_effect(self, record)
    }
    fn committed_dispatch_outbox_for_binding(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, binding: &crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleBinding) -> Result<crate::domain_computation::primary_graph::provider::WorthQueryCommittedDispatchOutboxObservation, crate::domain_computation::primary_graph::provider::WorthQueryCommittedDispatchOutboxReadDenial>{
        self.runtime
            .primary_provider
            .committed_dispatch_outbox_for_binding(execution, binding)
    }
}
