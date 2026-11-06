//! A succession request holds the selection; the attempt seals the succession authority.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::publication::PublishedWorkflowDefinitionRef;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentities, ApplicationMutationScopeBinding,
    WorthQueryApplicationOutputRole, WorthQueryCreateOutput, WorthQueryPreserveOutput,
};
use crate::domain_computation::primary_graph::application_attempt::{
    RequiredWorkflowSuccession, WorkflowSuccessionTarget,
};

impl WorthQueryWorkflowSuccessionAdapter {

    pub fn prepare<Schema, Binding, Role, Spec>(
        phase: &WorthQueryAdvancementPhase<'_>,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        predecessor: super::WorkflowSuccessionTarget<Schema>,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        admission: WorthQueryAdmittedApplicationOperation<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
    ) -> Result<RequiredWorkflowSuccession, WorkflowInstancePreparationDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Binding::Operation: ApplicationOperationMarkerIdentity<Schema>,
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Binding::Output,
            Action = WorthQueryPreserveOutput,
        >,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        crate::domain_computation::primary_graph::application_attempt::workflow_instance_program::prepare_succession::<Schema, Binding, Role, Spec>(
            phase,
            &selected.workflow_selection_basis(),
            installed,
            predecessor,
            identities,
            admission,
        )
    }

    pub const fn bind_idempotency(
        idempotency: WorthQueryApplicationIdempotencyBinding,
        identity: &[u8; 32],
    ) -> WorthQueryApplicationIdempotencyBinding {
        idempotency.bind_workflow_instance(identity)
    }
}
