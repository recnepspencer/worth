//! Optional draft values exercise the same native body and candidate owners.
use super::{program, request_scope, schema::*, DOCUMENT_IDENTITY};
use serde::Serialize;
use std::marker::PhantomData;
use worth_query_decl::facade::application_operation::ApplicationCandidatePolicy;
use worth_query_host::facade::{
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

pub struct DraftBinding<const DESTINATION: u64, Operation>(PhantomData<Operation>);
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
    const CANDIDATES: ApplicationCandidatePolicy =
        <super::binding::BodyBinding<DESTINATION> as ApplicationMutationBinding<
            DocumentRetentionSchema<DESTINATION>,
        >>::CANDIDATES;
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
    DraftInput,
    PhantomData<fn() -> Operation>,
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

fn invoke<Operation: DraftOperation<DocumentRetentionSchema<20>>>(
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

#[test]
fn native_optional_draft_set_empty_text_clear_and_replay() {
    let app = super::install::<20>();
    assert_eq!(super::draft_query::read_note(&app), None);
    assert!(matches!(
        invoke::<SetDraftNote>(&app, 300, ""),
        Outcome::Committed { .. }
    ));
    assert_eq!(super::draft_query::read_note(&app), Some(String::new()));
    assert!(matches!(
        invoke::<SetDraftNote>(&app, 300, ""),
        Outcome::AlreadyCommitted(_)
    ));
    assert!(matches!(
        invoke::<SetDraftNote>(&app, 301, "typed"),
        Outcome::Committed { .. }
    ));
    assert_eq!(
        super::draft_query::read_note(&app),
        Some("typed".to_owned())
    );
    // No optional step means Unchanged, even while another body writes the scope.
    assert!(matches!(
        super::invoke(&app, 303),
        Outcome::Committed { .. }
    ));
    assert_eq!(
        super::draft_query::read_note(&app),
        Some("typed".to_owned())
    );
    assert!(matches!(
        invoke::<ClearDraftNote>(&app, 302, ""),
        Outcome::Committed { .. }
    ));
    assert_eq!(super::draft_query::read_note(&app), None);
    assert!(matches!(
        invoke::<ClearDraftNote>(&app, 302, ""),
        Outcome::AlreadyCommitted(_)
    ));
}

#[test]
fn concurrent_optional_set_from_absence_stales_without_overwriting_the_winner() {
    use worth_query_host::facade::application_entry::WorthQueryApplicationProgramMutationPreparation as Preparation;
    use worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted;
    let app = super::install::<20>();
    let scope = request_scope();
    let principal = super::authenticate_operator(app.runtime().installed_schema(), &scope);
    let key = 304_u64;
    let mut request = app
        .request(&principal, &scope)
        .advanced()
        .mutate(DraftIntent::<20, SetDraftNote>::new("loser"))
        .without_source()
        .idempotency(&key);
    let candidate = match request
        .prepare_in_program(&app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("fresh optional body must prepare"),
    };
    assert!(matches!(
        invoke::<SetDraftNote>(&app, 305, "winner"),
        Outcome::Committed { .. }
    ));
    match candidate.commit(ExecutionAllocationPolicy::SystemAllocation) {
        Outcome::Commit(WorthQueryApplicationUncommitted::Stale(stale)) => {
            assert!(stale.stale_fact_count() > 0)
        }
        other => panic!("absent optional source must stale: {other:?}"),
    }
    assert_eq!(
        super::draft_query::read_note(&app),
        Some("winner".to_owned())
    );
}
