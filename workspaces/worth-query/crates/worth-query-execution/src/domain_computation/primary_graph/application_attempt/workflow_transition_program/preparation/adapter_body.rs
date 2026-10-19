//! Publication enters composed signing only with the actual prepared native approval.
use super::*;
use crate::domain_computation::primary_graph::operation_body::{
    WorthQueryOperationBodyDecision, WorthQueryOperationBodyDenial,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentities, ApplicationMutationScopeBinding,
};

impl WorthQueryWorkflowAdvanceAdapter {
    pub fn prepare_body<Schema, Binding>(
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: PreparedWorkflowAdvance<Schema, Binding::Operation, Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope>,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        principal_identity: &Binding::PrincipalIdentity,
        idempotency: crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
    ) -> Result<super::super::publication::WorkflowApprovalBodyPreparation<PreparedWorkflowAdvance<Schema, Binding::Operation, Binding::Input,
        <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope>>, super::super::publication::WorkflowApprovalBodyDenial>
    where Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema, Result = (), Denial = WorthQueryOperationBodyDenial,
            Decision = WorthQueryOperationBodyDecision<Schema, <<Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope>>,
        Binding::Input: Clone + Send + Sync + 'static,
    {
        prepared.prepare_body::<Binding>(
            access,
            phase,
            runtime,
            identities,
            principal_identity,
            idempotency,
        )
    }
}
