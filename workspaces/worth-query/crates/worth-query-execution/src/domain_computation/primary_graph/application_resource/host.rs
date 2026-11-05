//! What `application_resource` needs from the runtime that composes it.

/// The runtime's identity plus what `application_resource` reads or calls above itself.
pub trait ApplicationResourceHost<Schema>:
    crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost<Schema>
    + crate::domain_computation::authorization::host::AuthorizationHost<Schema>
{
    /// This host as the trait object `application_resource` is handed.
    fn application_resource_host(&self) -> &dyn ApplicationResourceHost<Schema>;
    fn installed_resources(&self) -> &crate::domain_computation::primary_graph::application_resource::installation::InstalledResourceRegistry<Schema>;
    fn on_branch(
        &self,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> crate::domain_computation::primary_graph::product_operation::WorthQueryProductEntry<
        '_,
        Schema,
    >;
    fn select_application_read_observation_with_request(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, observation: &crate::domain_computation::primary_graph::application_query::WorthQueryApplicationReadObservation) -> Result<crate::domain_computation::primary_graph::product_operation::WorthQuerySelectedProductOperation<'_, Schema>, crate::basis::WorthQueryProductBranchAdmissionDenial>;
    fn refuse_completed_external_effect(&self, record: &crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord) -> Result<(), crate::domain_computation::application_aftermath::external_effect::WorthQueryTerminalEffectRefusal>;
    fn committed_dispatch_outbox_for_binding(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, binding: &crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleBinding) -> Result<crate::domain_computation::primary_graph::provider::WorthQueryCommittedDispatchOutboxObservation, crate::domain_computation::primary_graph::provider::WorthQueryCommittedDispatchOutboxReadDenial>;
}
