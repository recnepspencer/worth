use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
        ApplicationMutationIntent, ApplicationMutationScopeBinding,
        ApplicationMutationScopeResolution,
    },
    application_program::ApplicationWorkflowSpec,
};
use worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_execution::facade::workflow_definition_publication::{
    PreparedWorkflowDefinitionPublication, WorkflowDefinitionExpectedPredecessor,
    WorkflowDefinitionPreparationDenial, WorkflowDefinitionPublicationOutcome,
    WorthQueryWorkflowDefinitionPublicationAdapter,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledWorkflowDefinitionContract,
};

use crate::application_entry::{
    mutation::{authorization, WorthQueryApplicationMutationRequestWithIdempotency},
    WorthQueryApplicationRequestMutationDenial,
};

type IntentBinding<Schema, Intent> = <Intent as ApplicationMutationIntent<Schema>>::Binding;
type MutationScope<Schema, Binding> =
    <<Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<
        Schema,
    >>::Scope;
type MutationOperation<Schema, Intent> =
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Operation;
type MutationInput<Schema, Intent> =
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Input;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowDefinitionPublicationPreparationDenialKind {
    RequestAdmission,
    DefinitionPreparation,
}

#[derive(Debug)]
pub enum WorthQueryWorkflowDefinitionPublicationPreparationDenial {
    RequestAdmission(WorthQueryApplicationRequestMutationDenial),
    DefinitionPreparation(WorkflowDefinitionPreparationDenial),
}

impl WorthQueryWorkflowDefinitionPublicationPreparationDenial {
    pub const fn kind(&self) -> WorthQueryWorkflowDefinitionPublicationPreparationDenialKind {
        match self {
            Self::RequestAdmission(_) => {
                WorthQueryWorkflowDefinitionPublicationPreparationDenialKind::RequestAdmission
            }
            Self::DefinitionPreparation(_) => {
                WorthQueryWorkflowDefinitionPublicationPreparationDenialKind::DefinitionPreparation
            }
        }
    }
}

impl std::fmt::Display for WorthQueryWorkflowDefinitionPublicationPreparationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "workflow definition publication preparation denied: {:?}",
            self.kind()
        )
    }
}

impl std::error::Error for WorthQueryWorkflowDefinitionPublicationPreparationDenial {}

impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        SourcePreparation,
    >
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema> + Clone,
    IntentBinding<Schema, Intent>: ApplicationCapabilityMutationBinding<Schema>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
    MutationInput<Schema, Intent>: Clone
        + ApplicationCapabilityRequest<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
            Scope = MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
{
    pub fn prepare_workflow_publication<Spec, Program>(
        mut self,
        contract: WorthQueryInstalledWorkflowDefinitionContract<Schema, Spec, Program>,
        expected_predecessor: WorkflowDefinitionExpectedPredecessor,
    ) -> Result<
        WorthQueryWorkflowDefinitionPublicationRequest<
            'application,
            Schema,
            MutationOperation<Schema, Intent>,
            MutationInput<Schema, Intent>,
            MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
        WorthQueryWorkflowDefinitionPublicationPreparationDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let application = self.application_runtime();
        let selected = application
            .on_branch(self.product_branch())
            .select()
            .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)
            .map_err(
                WorthQueryWorkflowDefinitionPublicationPreparationDenial::RequestAdmission,
            )?;
        let mutation = authorization::prepare_capability_selected(&mut self, &selected).map_err(
            WorthQueryWorkflowDefinitionPublicationPreparationDenial::RequestAdmission,
        )?;
        let prepared = WorthQueryWorkflowDefinitionPublicationAdapter::prepare::<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
            MutationOperation<Schema, Intent>,
            MutationInput<Schema, Intent>,
            MutationScope<Schema, IntentBinding<Schema, Intent>>,
            Spec,
            Program,
        >(
            &selected,
            contract,
            expected_predecessor,
            mutation.admission,
        )
        .map_err(
            WorthQueryWorkflowDefinitionPublicationPreparationDenial::DefinitionPreparation,
        )?;
        Ok(WorthQueryWorkflowDefinitionPublicationRequest {
            application,
            prepared,
            idempotency: mutation.idempotency,
        })
    }
}

pub struct WorthQueryWorkflowDefinitionPublicationRequest<
    'application,
    Schema,
    Operation,
    Input,
    Scope,
> where
    Schema: ApplicationSchema,
{
    application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    prepared: PreparedWorkflowDefinitionPublication<Schema, Operation, Input, Scope>,
    idempotency:
        worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyBinding,
}

impl<Schema, Operation, Input, Scope>
    WorthQueryWorkflowDefinitionPublicationRequest<'_, Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
{
    pub fn execute(self) -> WorkflowDefinitionPublicationOutcome {
        WorthQueryWorkflowDefinitionPublicationAdapter::compare_and_commit(
            self.application,
            self.prepared,
            self.idempotency,
        )
    }
}
