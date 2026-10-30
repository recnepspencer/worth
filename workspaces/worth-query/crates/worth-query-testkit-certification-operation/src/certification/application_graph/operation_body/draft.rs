//! Optional draft values exercise the same native body and candidate owners.
pub use super::{program, request_scope, schema::*, DOCUMENT_IDENTITY};
pub use serde::Serialize;
pub use std::marker::PhantomData;

pub use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome, WorthQueryApplicationRequestExt,
    },
    declaration::{application_operation::*, application_schema::*},
    primary_graph::{WorthQueryOperationBodyDecision, WorthQueryOperationBodyDenial},
    runtime::ExecutionAllocationPolicy,
};

#[derive(Clone, Debug, Serialize)]
pub struct DraftInput {
    pub identity: String,
    pub note: String,
}
pub struct DraftInputBinding;
impl ApplicationStructuredValueBinding for DraftInputBinding {
    type Value = DraftInput;
    const IDENTITY_NAME: &'static str = "worth.query.body-court.draft-input.v1";
    const RETAIN_CANONICAL_INPUT: bool = true;
    fn validate(_: &DraftInput) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}

pub trait DraftOperation<Schema: BodySchemaBinding>:
    ApplicationOperationMarkerIdentity<Schema, InputBinding = DraftInputBinding> + 'static
{
    const BINDING_IDENTITY: &'static str;
    const HANDLER_IDENTITY: &'static str;
    const IDEMPOTENCY_IDENTITY: &'static str;
}
impl<Schema: BodySchemaBinding> DraftOperation<Schema> for SetDraftNote {
    const BINDING_IDENTITY: &'static str = "worth.query.body-court.set-draft.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.body-court.set-draft-body.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.body-court.set-draft-command.v1";
}
impl<Schema: BodySchemaBinding> DraftOperation<Schema> for ClearDraftNote {
    const BINDING_IDENTITY: &'static str = "worth.query.body-court.clear-draft.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.body-court.clear-draft-body.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.body-court.clear-draft-command.v1";
}
impl<Schema: BodySchemaBinding> DraftOperation<Schema> for RequireDraftNote {
    const BINDING_IDENTITY: &'static str = "worth.query.body-court.require-draft.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.body-court.require-draft-body.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.body-court.require-draft-command.v1";
}

pub struct DraftBinding<const DESTINATION: u64, Operation>(pub PhantomData<Operation>);
impl<const DESTINATION: u64, Operation>
    ApplicationMutationBinding<DocumentRetentionSchema<DESTINATION>>
    for DraftBinding<DESTINATION, Operation>
where
    Operation: DraftOperation<DocumentRetentionSchema<DESTINATION>>,
{
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    type Input = DraftInput;
    type InputBinding = DraftInputBinding;
    type Result = ();
    type ResultBinding = super::binding::BodyResultBinding;
    type IdempotencyKey = u64;
    type Operation = Operation;
    type Decision = WorthQueryOperationBodyDecision<DocumentRetentionSchema<DESTINATION>, Document>;
    type Denial = WorthQueryOperationBodyDenial;
    type DenialBinding = super::binding::BodyDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ApplicationMutationFieldScope<
        DocumentRetentionSchema<DESTINATION>,
        Document,
        DocumentFacts,
        DocumentIdentityField,
        String,
        ReadOnly,
        NoApplicationUnit,
    >;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = Operation::BINDING_IDENTITY;
    const HANDLER_IDENTITY: &'static str = Operation::HANDLER_IDENTITY;
    const IDEMPOTENCY_IDENTITY: &'static str = Operation::IDEMPOTENCY_IDENTITY;
    const REQUIRES_APPLICATION_PROGRAM: bool = true;

    fn scope_field() -> ApplicationFieldRef<
        DocumentRetentionSchema<DESTINATION>,
        Document,
        DocumentFacts,
        DocumentIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        DocumentIdentityField::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DocumentRetentionSchema<DESTINATION>,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}

pub struct DraftIntent<const DESTINATION: u64, Operation>(
    pub DraftInput,
    pub PhantomData<fn() -> Operation>,
);
impl<const DESTINATION: u64, Operation> Clone for DraftIntent<DESTINATION, Operation> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}
impl<const DESTINATION: u64, Operation> DraftIntent<DESTINATION, Operation> {
    pub fn new(note: &str) -> Self {
        Self(
            DraftInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                note: note.to_owned(),
            },
            PhantomData,
        )
    }
}
impl<const DESTINATION: u64, Operation>
    ApplicationMutationIntent<DocumentRetentionSchema<DESTINATION>>
    for DraftIntent<DESTINATION, Operation>
where
    Operation: DraftOperation<DocumentRetentionSchema<DESTINATION>>,
{
    type Binding = DraftBinding<DESTINATION, Operation>;
    fn input(&self) -> &DraftInput {
        &self.0
    }
    fn scope_binding(&self) -> <Self::Binding as ApplicationMutationBinding<DocumentRetentionSchema<DESTINATION>>>::ScopeBinding{
        ApplicationMutationFieldScope::new(
            DocumentIdentityField::reference(),
            self.0.identity.clone(),
        )
    }
}

pub fn invoke<Operation: DraftOperation<DocumentRetentionSchema<20>>>(
    app: &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<20>,
        program::BodyProgram<20>,
    >,
    key: u64,
    note: &str,
) -> Outcome<WorthQueryOperationBodyDenial, ()> {
    let scope = request_scope();
    let principal = super::authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .advanced()
        .mutate(DraftIntent::<20, Operation>::new(note))
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
}
