//! A public fixture mutation inserts the native authored applicability relation.

use worth_query_decl::facade::{worth_query_operation_links, worth_query_operation_reads};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
    WorthQueryApplicationRequestMutationDenial,
};
use worth_query_host::facade::declaration::application_operation::{
    ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
    ApplicationCandidateResourceCeiling, ApplicationMutationBinding, ApplicationMutationFieldScope,
    ApplicationMutationIntent, NoApplicationMutationOutputs, NoApplicationMutationSource,
};
use worth_query_host::facade::declaration::application_schema::{
    ApplicationFieldRef, ApplicationPrincipalBindingRef, ApplicationRelationCardinality,
    ApplicationRelationIntegrity, ApplicationSchemaDeclarationBuilder, EqualityPredicate,
    NoApplicationUnit, ReadOnly, U64ApplicationValueBinding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryInvariantMutationTarget,
};
use worth_query_host::facade::product::WorthQueryProductBranch;
use worth_query_host::facade::{
    worth_query_operation, worth_query_relation, worth_query_structured_value_binding,
};

use super::super::host::DocumentWorkflowRuntime;
use super::super::operator_identity::{authenticate_operator, request_scope};
use super::super::retention_entry::{DOCUMENT_IDENTITY, RELATED_DOCUMENT_IDENTITY};
use super::super::schema::{
    Document, DocumentFacts, DocumentIdentityField, DocumentPrincipalBinding,
    DocumentRetentionSchema, ExternalMapping, Principal,
};

#[path = "review_requirement/unlink.rs"]
mod unlink;
pub use unlink::{
    unlink_review_requirement, unlink_review_requirement_on, UnlinkReviewRequirementBinding,
    UnlinkReviewRequirementHandler,
};

worth_query_relation!(pub ReviewRequired in DocumentRetentionSchema, Document => Document; integrity = ApplicationRelationIntegrity {
    cardinality: ApplicationRelationCardinality::new(None, Some(1), None, None, None, Some(1)),
    ..ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
});

#[derive(Clone, Debug, serde::Serialize)]
pub struct ReviewRequirementInput {
    pub resource: String,
    pub related: String,
}
worth_query_structured_value_binding!(pub ReviewRequirementInputBinding for ReviewRequirementInput {
    identity: "worth.query.certification.review-requirement-input.v1"
});
worth_query_operation!(pub LinkReviewRequirement for DocumentRetentionSchema, input ReviewRequirementInputBinding);
worth_query_operation_reads!(LinkReviewRequirement => [DocumentIdentityField]);
worth_query_operation_links!(LinkReviewRequirement => [ReviewRequired]);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewRequirementLinked;
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReviewRequirementDenial {
    RelationMismatch,
}
worth_query_structured_value_binding!(pub ReviewRequirementLinkedBinding for ReviewRequirementLinked {
    identity: "worth.query.certification.review-requirement-linked.v1"
});
worth_query_structured_value_binding!(pub ReviewRequirementDenialBinding for ReviewRequirementDenial {
    identity: "worth.query.certification.review-requirement-denial.v1"
});

pub struct ReviewRequirementBinding;
type ReviewRequirementScope = ApplicationMutationFieldScope<
    DocumentRetentionSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
pub struct ReviewRequirementTargets {
    resource: WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>,
    related: WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>,
}

impl ApplicationMutationBinding<DocumentRetentionSchema> for ReviewRequirementBinding {
    type Input = ReviewRequirementInput;
    type InputBinding = ReviewRequirementInputBinding;
    type Result = ReviewRequirementLinked;
    type ResultBinding = ReviewRequirementLinkedBinding;
    type IdempotencyKey = u64;
    type Operation = LinkReviewRequirement;
    type Decision = ReviewRequirementTargets;
    type Denial = ReviewRequirementDenial;
    type DenialBinding = ReviewRequirementDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ReviewRequirementScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;

    const IDENTITY: &'static str = "worth.query.certification.review-requirement-binding.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.certification.review-requirement-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.review-requirement-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 1, 0, 0, 0),
            ApplicationCandidateResourceCeiling::bounded(1024, 1024),
        );

    fn scope_field() -> ApplicationFieldRef<
        DocumentRetentionSchema,
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

#[derive(Clone)]
pub struct ReviewRequirementIntent {
    pub input: ReviewRequirementInput,
}
impl ApplicationMutationIntent<DocumentRetentionSchema> for ReviewRequirementIntent {
    type Binding = ReviewRequirementBinding;
    fn input(&self) -> &ReviewRequirementInput {
        &self.input
    }
    fn scope_binding(&self) -> ReviewRequirementScope {
        ReviewRequirementScope::new(
            DocumentIdentityField::reference(),
            self.input.resource.clone(),
        )
    }
}

pub struct ReviewRequirementHandler;
impl OperationHandler<DocumentRetentionSchema, ReviewRequirementBinding>
    for ReviewRequirementHandler
{
    fn decide(
        &self,
        input: &ReviewRequirementInput,
        reader: &mut DecisionReader<'_, '_, '_, DocumentRetentionSchema, ReviewRequirementBinding>,
    ) -> HandlerResult<ReviewRequirementTargets, ReviewRequirementDenial> {
        let resource = match reader
            .resolve_entity(DocumentIdentityField::reference(), input.resource.clone())
        {
            Ok(value) => value,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        let related = match reader
            .resolve_entity(DocumentIdentityField::reference(), input.related.clone())
        {
            Ok(value) => value,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        let resource = match reader.mutation_target(&resource) {
            Ok(value) => value,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        let related = match reader.mutation_target(&related) {
            Ok(value) => value,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        HandlerResult::Completed(ReviewRequirementTargets { resource, related })
    }
    fn candidate_requirements(
        &self,
        _: &ReviewRequirementInput,
        _: &ReviewRequirementTargets,
    ) -> ApplicationCandidateRequirements {
        ReviewRequirementBinding::CANDIDATES
    }
    fn build_candidate(
        &self,
        _: &ReviewRequirementInput,
        decision: ReviewRequirementTargets,
        writer: &mut CandidateWriter<'_, DocumentRetentionSchema, ReviewRequirementBinding>,
    ) -> HandlerResult<ReviewRequirementLinked, ReviewRequirementDenial> {
        let resource = match writer.projected_entity(&decision.resource) {
            Ok(value) => value,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        let related = match writer.projected_entity(&decision.related) {
            Ok(value) => value,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        match writer.link(
            ReviewRequired::reference(),
            "review-required",
            &resource,
            &related,
        ) {
            Ok(()) => HandlerResult::Completed(ReviewRequirementLinked),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    let schema = schema
        .relation(
            ReviewRequired::reference(),
            Document::reference(),
            Document::reference(),
        )
        .operation(
            LinkReviewRequirement::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(LinkReviewRequirement::reference(), 16)
        .operation_projection_work_budget(LinkReviewRequirement::reference(), 32)
        .operation_read_field(
            LinkReviewRequirement::reference(),
            DocumentIdentityField::reference(),
        )
        .operation_link(
            LinkReviewRequirement::reference(),
            ReviewRequired::reference(),
        )
        .application_mutation_binding::<ReviewRequirementBinding>();
    unlink::declare(schema)
}

pub fn link_review_requirement(
    application: &DocumentWorkflowRuntime,
    idempotency: u64,
) -> Result<
    WorthQueryApplicationMutationOutcome<ReviewRequirementDenial, ReviewRequirementLinked>,
    WorthQueryApplicationRequestMutationDenial,
> {
    link_review_requirement_on(
        application,
        application.runtime().current_world(),
        idempotency,
    )
}

/// Issues the same mutation on one exact World product occurrence.
pub fn link_review_requirement_on(
    application: &DocumentWorkflowRuntime,
    branch: WorthQueryProductBranch,
    idempotency: u64,
) -> Result<
    WorthQueryApplicationMutationOutcome<ReviewRequirementDenial, ReviewRequirementLinked>,
    WorthQueryApplicationRequestMutationDenial,
> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(ReviewRequirementIntent {
            input: ReviewRequirementInput {
                resource: DOCUMENT_IDENTITY.to_owned(),
                related: RELATED_DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .execute_in_program(
            application.program_runtime(),
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
}
