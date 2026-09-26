use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
        ApplicationMutationIntent, ApplicationMutationScopeBinding,
        ApplicationMutationScopeResolution, NoApplicationMutationSource,
    },
    application_program::{
        ApplicationWorkflowSpec, ApplicationWorkflowValidationDenial, AuthoredWorkflowDefinition,
    },
};
use worth_query_execution::facade::{
    application_installation::WorthQueryWorkflowVocabulary,
    workflow_definition_publication::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    },
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryApplicationWorkflowInstallationDenial,
    WorthQueryInstalledWorkflowDefinitionContract,
};

use crate::application_entry::{
    mutation::{
        WorthQueryApplicationMutationRequest, WorthQueryApplicationMutationRequestWithIdempotency,
        WorthQueryMutationSourcePrepared,
    },
    WorthQueryWorkflowDefinitionPublicationPreparationDenial,
};

type IntentBinding<Schema, Intent> = <Intent as ApplicationMutationIntent<Schema>>::Binding;
type MutationScope<Schema, Binding> =
    <<Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<
        Schema,
    >>::Scope;
type MutationInput<Schema, Intent> =
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Input;
type MutationOperation<Schema, Intent> =
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Operation;

#[derive(Debug)]
pub enum WorthQueryOrdinaryWorkflowPublicationDenial {
    RuntimeMismatch,
    InvalidDefinition(ApplicationWorkflowValidationDenial),
    UnsupportedDefinition(WorthQueryApplicationWorkflowInstallationDenial),
    Preparation(WorthQueryWorkflowDefinitionPublicationPreparationDenial),
}

impl std::fmt::Display for WorthQueryOrdinaryWorkflowPublicationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RuntimeMismatch => {
                formatter.write_str("workflow runtime belongs to another application")
            }
            Self::InvalidDefinition(denial) => {
                write!(formatter, "invalid authored workflow: {denial}")
            }
            Self::UnsupportedDefinition(denial) => write!(
                formatter,
                "workflow installation rejected definition: {denial}"
            ),
            Self::Preparation(denial) => write!(
                formatter,
                "workflow publication preparation denied: {denial}"
            ),
        }
    }
}

impl std::error::Error for WorthQueryOrdinaryWorkflowPublicationDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RuntimeMismatch => None,
            Self::InvalidDefinition(denial) => Some(denial),
            Self::UnsupportedDefinition(denial) => Some(denial),
            Self::Preparation(denial) => Some(denial),
        }
    }
}

pub struct WorthQueryOrdinaryWorkflowDraft<
    'application,
    'principal,
    'scope,
    Schema,
    Intent,
    Spec,
    Program,
> where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    request: WorthQueryApplicationMutationRequest<'application, 'principal, 'scope, Schema, Intent>,
    contract: WorthQueryInstalledWorkflowDefinitionContract<Schema, Spec, Program>,
}

pub struct WorthQueryOrdinaryWorkflowPublication<
    'application,
    'principal,
    'scope,
    Schema,
    Intent,
    Spec,
    Program,
> where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    draft: WorthQueryOrdinaryWorkflowDraft<
        'application,
        'principal,
        'scope,
        Schema,
        Intent,
        Spec,
        Program,
    >,
    expected_predecessor: WorkflowDefinitionExpectedPredecessor,
}

pub struct WorthQueryOrdinaryWorkflowPublicationWithIdempotency<
    'application,
    'principal,
    'scope,
    'key,
    Schema,
    Intent,
    Spec,
    Program,
> where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    request: WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        WorthQueryMutationSourcePrepared,
    >,
    contract: WorthQueryInstalledWorkflowDefinitionContract<Schema, Spec, Program>,
    expected_predecessor: WorkflowDefinitionExpectedPredecessor,
}

impl<'application, 'principal, 'scope, Schema, Intent>
    WorthQueryApplicationMutationRequest<'application, 'principal, 'scope, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    /// Pure admission and installed-vocabulary binding; no product selection or permission is retained.
    pub fn workflow<'workflow, Spec, Program: 'workflow>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec, Program>>,
        authored: AuthoredWorkflowDefinition<Spec>,
    ) -> Result<
        WorthQueryOrdinaryWorkflowDraft<
            'application,
            'principal,
            'scope,
            Schema,
            Intent,
            Spec,
            Program,
        >,
        WorthQueryOrdinaryWorkflowPublicationDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let workflow = workflow.into();
        if !std::ptr::eq(self.application_runtime(), workflow.runtime()) {
            return Err(WorthQueryOrdinaryWorkflowPublicationDenial::RuntimeMismatch);
        }
        let validated = authored
            .validate()
            .map_err(WorthQueryOrdinaryWorkflowPublicationDenial::InvalidDefinition)?;
        let contract = workflow
            .workflow_spec()
            .bind_definition(validated)
            .map_err(WorthQueryOrdinaryWorkflowPublicationDenial::UnsupportedDefinition)?;
        Ok(WorthQueryOrdinaryWorkflowDraft {
            request: self,
            contract,
        })
    }
}

impl<'application, 'principal, 'scope, Schema, Intent, Spec, Program>
    WorthQueryOrdinaryWorkflowDraft<'application, 'principal, 'scope, Schema, Intent, Spec, Program>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub fn publish(
        self,
        expected_predecessor: WorkflowDefinitionExpectedPredecessor,
    ) -> WorthQueryOrdinaryWorkflowPublication<
        'application,
        'principal,
        'scope,
        Schema,
        Intent,
        Spec,
        Program,
    > {
        WorthQueryOrdinaryWorkflowPublication {
            draft: self,
            expected_predecessor,
        }
    }
}

impl<'application, 'principal, 'scope, Schema, Intent, Spec, Program>
    WorthQueryOrdinaryWorkflowPublication<
        'application,
        'principal,
        'scope,
        Schema,
        Intent,
        Spec,
        Program,
    >
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    IntentBinding<Schema, Intent>:
        ApplicationMutationBinding<Schema, SourceExpectation = NoApplicationMutationSource>,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub fn idempotency<'key>(
        self,
        key: &'key <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::IdempotencyKey,
    ) -> WorthQueryOrdinaryWorkflowPublicationWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        Spec,
        Program,
    > {
        WorthQueryOrdinaryWorkflowPublicationWithIdempotency {
            request: self.draft.request.without_source().idempotency(key),
            contract: self.draft.contract,
            expected_predecessor: self.expected_predecessor,
        }
    }
}

impl<Schema, Intent, Spec, Program>
    WorthQueryOrdinaryWorkflowPublicationWithIdempotency<'_, '_, '_, '_, Schema, Intent, Spec, Program>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema> + Clone,
    IntentBinding<Schema, Intent>: ApplicationCapabilityMutationBinding<Schema>,
    MutationOperation<Schema, Intent>: 'static,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
    MutationInput<Schema, Intent>: Clone + Send + Sync + 'static
        + ApplicationCapabilityRequest<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
            Scope = MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub fn execute(self) -> Result<WorkflowDefinitionPublicationOutcome, WorthQueryOrdinaryWorkflowPublicationDenial> {
        let prepared = self.request
            .prepare_workflow_publication(self.contract, self.expected_predecessor)
            .map_err(WorthQueryOrdinaryWorkflowPublicationDenial::Preparation)?;
        Ok(prepared.execute())
    }
}
