use worth_execution::ExecutionRequest;

/// Upper-layer runtime port for `WorthQueryProviderCommitAuthorization::authorize_application_commit`.
pub trait CommitAuthorizationRuntimePort<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    fn authorize_with_commit_basis<'serialization, 'admission, Operation, Input, Scope>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        approval_authority: Option<
            &crate::domain_computation::authorization::WorthQueryWorkflowApprovalAuthorityBasis,
        >,
        commit_basis: &crate::domain_computation::authorization::WorthQueryCommitAuthorizationBasis,
        admission: &'admission crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            Scope,
        >,
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
    >;
}

pub trait WorkflowDefinitionRetirementPort<Schema, Operation, Input, Scope>: Sized {
    fn into_retirement_commit_parts(
        self,
    ) -> (
        crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationEffectProgram<
            Schema,
            Operation,
            Input,
            Scope,
        >,
        worth_query_declaration::facade::application_program::ApplicationProgramRevision,
        crate::domain_computation::primary_graph::application_attempt::workflow_definition_program::PublishedWorkflowDefinitionRef,
        [u8; 32],
    );

    fn complete_retirement(
        definition: crate::domain_computation::primary_graph::application_attempt::workflow_definition_program::PublishedWorkflowDefinitionRef,
        receipt: crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitReceipt,
    ) -> crate::domain_computation::primary_graph::application_attempt::workflow_definition_program::retirement::PerformedWorkflowDefinitionRetirement;
}
