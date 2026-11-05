//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProviderCommitAuthorization {
    #[doc(hidden)]
    pub fn authorize_application_commit_for_test<
        'serialization,
        'admission,
        Schema,
        Operation,
        Input,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        application: &dyn crate::domain_computation::authorization::host::AuthorizationHost<Schema>,
        admission: &'admission crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        serialization: &'serialization crate::domain_computation::primary_graph::WorthQueryApplicationBranchCommitCoordination<'_>,
    ) -> Result<
        crate::domain_computation::authorization::WorthQueryApplicationCommitAuthorization<
            'serialization,
            'admission,
            Schema,
            Operation,
            Input,
            Scope,
        >,
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    >
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        Self::authorize_application_commit::<Schema, Operation, Input, Scope>(
            self,
            execution,
            application,
            admission,
            serialization,
        )
    }
}
