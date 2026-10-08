//! An ordinary, owner-published grant change for workflow wait-currentness courts.

use worth_query_host::facade::{
    declaration::{
        application_operation::{
            ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
            ApplicationCandidateResourceCeiling, ApplicationMutationBinding,
            ApplicationMutationFieldScope, ApplicationMutationIntent, NoApplicationMutationOutputs,
            NoApplicationMutationSource,
        },
        application_schema::{
            ApplicationFieldRef, ApplicationPrincipalBindingRef,
            ApplicationSchemaDeclarationBuilder, EqualityPredicate, NoApplicationUnit, ReadOnly,
            U64ApplicationValueBinding,
        },
    },
    primary_graph::{
        CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
        WorthQueryInvariantMutationTarget,
    },
    worth_query_operation, worth_query_operation_reads, worth_query_operation_writes,
    worth_query_portable_type, worth_query_structured_value_binding,
};

use super::super::schema::{
    DocumentPrincipalBinding, DocumentRetentionSchema, ExternalMapping, Principal,
};
use super::declaration::{
    WorkflowAuthoringGrant, WorkflowAuthoringGrantFacts, WorkflowGrantIdentityField,
    WorkflowGrantStatusField,
};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct WorkflowGrantStatusInput {
    pub grant_identity: String,
    pub status: String,
}

worth_query_portable_type!(WorkflowGrantStatusInput => "worth.query.certification.workflow-grant-status-input.v1");
worth_query_structured_value_binding!(pub WorkflowGrantStatusInputBinding for WorkflowGrantStatusInput {
    identity: "worth.query.certification.workflow-grant-status-input.v1"
});
worth_query_operation!(pub WorkflowGrantStatusOperation for DocumentRetentionSchema, input WorkflowGrantStatusInputBinding);
worth_query_operation_reads!(WorkflowGrantStatusOperation => [WorkflowGrantIdentityField, WorkflowGrantStatusField]);
worth_query_operation_writes!(WorkflowGrantStatusOperation => [WorkflowGrantStatusField]);

#[derive(Clone, Debug)]
pub struct WorkflowGrantStatusIntent {
    pub input: WorkflowGrantStatusInput,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowGrantStatusChanged;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowGrantStatusDenial {
    InvalidStatus,
    Unchanged,
}

worth_query_structured_value_binding!(pub WorkflowGrantStatusChangedBinding for WorkflowGrantStatusChanged {
    identity: "worth.query.certification.workflow-grant-status-changed.v1"
});
worth_query_structured_value_binding!(pub WorkflowGrantStatusDenialBinding for WorkflowGrantStatusDenial {
    identity: "worth.query.certification.workflow-grant-status-denial.v1"
});

pub struct WorkflowGrantStatusBinding;

type GrantScope = ApplicationMutationFieldScope<
    DocumentRetentionSchema,
    WorkflowAuthoringGrant,
    WorkflowAuthoringGrantFacts,
    WorkflowGrantIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;

impl ApplicationMutationBinding<DocumentRetentionSchema> for WorkflowGrantStatusBinding {
    type Input = WorkflowGrantStatusInput;
    type InputBinding = WorkflowGrantStatusInputBinding;
    type Result = WorkflowGrantStatusChanged;
    type ResultBinding = WorkflowGrantStatusChangedBinding;
    type IdempotencyKey = u64;
    type Operation = WorkflowGrantStatusOperation;
    type Decision =
        WorthQueryInvariantMutationTarget<DocumentRetentionSchema, WorkflowAuthoringGrant>;
    type Denial = WorkflowGrantStatusDenial;
    type DenialBinding = WorkflowGrantStatusDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = GrantScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;

    const IDENTITY: &'static str = "worth.query.certification.workflow-grant-status-binding.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.certification.workflow-grant-status-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.workflow-grant-status-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 2, 0),
            ApplicationCandidateResourceCeiling::representation_bytes(1_024),
        );

    fn scope_field() -> ApplicationFieldRef<
        DocumentRetentionSchema,
        WorkflowAuthoringGrant,
        WorkflowAuthoringGrantFacts,
        WorkflowGrantIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        WorkflowGrantIdentityField::reference()
    }

    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DocumentRetentionSchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}

impl ApplicationMutationIntent<DocumentRetentionSchema> for WorkflowGrantStatusIntent {
    type Binding = WorkflowGrantStatusBinding;

    fn input(&self) -> &WorkflowGrantStatusInput {
        &self.input
    }

    fn scope_binding(&self) -> GrantScope {
        GrantScope::new(
            WorkflowGrantIdentityField::reference(),
            self.input.grant_identity.clone(),
        )
    }
}

pub struct WorkflowGrantStatusHandler;

impl OperationHandler<DocumentRetentionSchema, WorkflowGrantStatusBinding>
    for WorkflowGrantStatusHandler
{
    fn decide(
        &self,
        input: &WorkflowGrantStatusInput,
        reader: &mut DecisionReader<
            '_,
            '_,
            '_,
            DocumentRetentionSchema,
            WorkflowGrantStatusBinding,
        >,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<DocumentRetentionSchema, WorkflowAuthoringGrant>,
        WorkflowGrantStatusDenial,
    > {
        if input.status != "active" && input.status != "revoked" {
            return HandlerResult::DomainDenied(WorkflowGrantStatusDenial::InvalidStatus);
        }
        let grant = match reader.resolve_entity(
            WorkflowGrantIdentityField::reference(),
            input.grant_identity.clone(),
        ) {
            Ok(grant) => grant,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        match reader.field(&grant, WorkflowGrantStatusField::reference()) {
            Ok(Some(current)) if current == input.status => {
                return HandlerResult::DomainDenied(WorkflowGrantStatusDenial::Unchanged)
            }
            Ok(_) => {}
            Err(error) => return HandlerResult::ExecutionDenied(error),
        }
        match reader.mutation_target(&grant) {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn candidate_requirements(
        &self,
        _: &WorkflowGrantStatusInput,
        _: &WorthQueryInvariantMutationTarget<DocumentRetentionSchema, WorkflowAuthoringGrant>,
    ) -> ApplicationCandidateRequirements {
        WorkflowGrantStatusBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        input: &WorkflowGrantStatusInput,
        target: WorthQueryInvariantMutationTarget<DocumentRetentionSchema, WorkflowAuthoringGrant>,
        writer: &mut CandidateWriter<'_, DocumentRetentionSchema, WorkflowGrantStatusBinding>,
    ) -> HandlerResult<WorkflowGrantStatusChanged, WorkflowGrantStatusDenial> {
        let grant = match writer.projected_entity(&target) {
            Ok(grant) => grant,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        if let Err(error) = writer.write_field(
            &grant,
            WorkflowGrantStatusField::reference(),
            input.status.clone(),
        ) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
        }
        HandlerResult::Completed(WorkflowGrantStatusChanged)
    }
}

pub(super) fn install_members(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    schema
        .operation(
            WorkflowGrantStatusOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(WorkflowGrantStatusOperation::reference(), 64)
        .operation_read_field(
            WorkflowGrantStatusOperation::reference(),
            WorkflowGrantIdentityField::reference(),
        )
        .operation_read_field(
            WorkflowGrantStatusOperation::reference(),
            WorkflowGrantStatusField::reference(),
        )
        .operation_write(
            WorkflowGrantStatusOperation::reference(),
            WorkflowGrantStatusField::reference(),
        )
        .application_mutation_binding::<WorkflowGrantStatusBinding>()
}
