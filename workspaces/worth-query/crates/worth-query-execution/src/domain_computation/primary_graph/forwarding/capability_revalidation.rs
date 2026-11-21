//! Thin forwarding shims for `capability_revalidation` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_authorization_capability_revalidation {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
    use worth_query_execution_query::facade::domain_computation::authorization::WorthQueryRetainedCapabilityAuthorization;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema, {
        pub(crate) fn refresh_capability_authorization_for_graph_work(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, authorization: &mut WorthQueryRetainedCapabilityAuthorization, graph_work: &crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession) -> Result<(), WorthQueryOperationAuthorizationDenial> {
            self.inner.refresh_capability_authorization_for_graph_work(execution, authorization, graph_work)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_authorization_capability_revalidation_prepared_read {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
    use worth_query_execution_query::facade::domain_computation::authorization::WorthQueryRetainedCapabilityAuthorization;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn refresh_capability_authorization_for_prepared_read(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, authorization: &mut WorthQueryRetainedCapabilityAuthorization, reserved: &crate::domain_computation::provider_session::WorthQueryPreparedProductReadSessionIdentity, product: &crate::basis::WorthQueryProductObservationLease) -> Result<(), WorthQueryOperationAuthorizationDenial> {
            self.inner.refresh_capability_authorization_for_prepared_read(execution, authorization, reserved, product)
        }
    }
}
