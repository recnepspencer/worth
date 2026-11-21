//! Thin forwarding shims for `authorization_revalidation` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_authorization_authorization_revalidation_workflow_approval {
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
    use crate::domain_computation::authorization::WorthQueryWorkflowApprovalAuthorityBasis;
    use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema, {
        pub fn readmit_workflow_approval_authority<Operation, Input, Scope>(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, authority: &WorthQueryWorkflowApprovalAuthorityBasis, admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>) -> Result<(), WorthQueryOperationAuthorizationDenial> {
            self.inner.readmit_workflow_approval_authority::<Operation, Input, Scope>(execution, authority, admission)
        }
    }
}
